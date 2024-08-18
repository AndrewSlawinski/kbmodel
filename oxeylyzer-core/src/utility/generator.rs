use crate::config::config::Config;
use crate::config::pins::Pins;
use crate::data_dir::DataFetch;
use crate::language::language_data::LanguageData;
use crate::layout::layout::FastLayout;
use crate::n_gram::bigram_type::BigramType;
use crate::n_gram::n_gram::NGram;
use crate::stats::layout_cache::StatsCache;
use crate::type_def::*;
use crate::utility::pair::Pair;
use crate::utility::scorer::Scorer;
use itertools::Itertools;
use rayon::iter::{
    IntoParallelIterator,
    ParallelIterator,
};
use std::collections::HashMap;

pub struct Generator<'a> {
    pub language_data: &'a LanguageData,
    pub config: &'a Config,
    pub pins: &'a Pins,

    pub scorer: &'a Scorer<'a>,

    language_chars_default: HashMap<String, Fixed<char>>,

    possible_swaps: [Pair; 435],
}

impl<'a> Generator<'a> {
    pub fn new(language_data: &'a LanguageData, config: &'a Config, scorer: &'a Scorer) -> Self {
        let language_chars_default = DataFetch::chars_in_languages_default();

        return Self {
            language_data: &language_data,
            config: &config,
            pins: &config.pins,
            scorer,
            language_chars_default,
            possible_swaps: Self::get_possible_swaps(),
        };
    }

    pub fn update_cache(
        &self,
        layout: &FastLayout,
        swap: &Pair,
        cache: &mut StatsCache,
        trigrams_start: f64,
    ) {
        let col0 = FINGER_TO_COLUMN[swap.0];
        let col1 = FINGER_TO_COLUMN[swap.1];

        cache.finger_speed_total = if col0 == col1
        {
            let fspeed = self.scorer.column_finger_speed(layout, col0);

            let total = cache.finger_speed_total - cache.effort_stats.finger_speeds[col0] + fspeed;

            cache.effort_stats.finger_speeds[col0] = fspeed;

            total
        } else {
            let fspeed0 = self.scorer.column_finger_speed(layout, col0);
            let fspeed1 = self.scorer.column_finger_speed(layout, col1);

            let total = cache.finger_speed_total - cache.effort_stats.finger_speeds[col0] - cache.effort_stats.finger_speeds[col1] + fspeed0 + fspeed1;

            cache.effort_stats.finger_speeds[col0] = fspeed0;
            cache.effort_stats.finger_speeds[col1] = fspeed1;

            total
        };

        cache.usage_total = if col0 == col1
        {
            let usage = self.scorer.column_usage(layout, col0);
            let total = cache.usage_total - cache.effort_stats.usage[col0] + usage;

            cache.effort_stats.usage[col0] = usage;

            total
        } else {
            let usage0 = self.scorer.column_usage(layout, col0);
            let usage1 = self.scorer.column_usage(layout, col1);

            let total = cache.usage_total - cache.effort_stats.usage[col0] - cache.effort_stats.usage[col1] + usage0 + usage1;

            cache.effort_stats.usage[col0] = usage0;
            cache.effort_stats.usage[col1] = usage1;

            total
        };

        let effort1 = self.scorer.char_effort(layout, swap.0);
        let effort2 = self.scorer.char_effort(layout, swap.1);

        cache.char_effort_total = cache.char_effort_total - cache.effort_stats.char_effort[swap.0] - cache.effort_stats.char_effort[swap.1] + effort1 + effort2;

        cache.effort_stats.char_effort[swap.0] = effort1;
        cache.effort_stats.char_effort[swap.1] = effort2;

        let trigrams_end = self.scorer.trigram_char_score(layout, swap);

        cache.trigrams_total = cache.trigrams_total - trigrams_start + trigrams_end;

        if swap.affects_scissor()
        {
            cache.bigram_stats[BigramType::Scissors] = self.scorer.scissor_score(layout);
        }

        if swap.affects_lsb()
        {
            cache.bigram_stats[BigramType::LateralStretchBigrams] = self.scorer.lateral_stretch_bigram_score(layout);
        }

        cache.total_score = cache.score();
    }

    fn optimize_cached(
        &self,
        layout: &mut FastLayout,
        cache: &mut StatsCache,
        possible_swaps: &[Pair],
    ) -> f64 {
        let mut current_best_score = f64::MIN / 2.0;

        while let (Some(best_swap), new_score) = self.scorer.best_swap_cached(layout, cache, Some(current_best_score), possible_swaps) {
            current_best_score = new_score;

            let trigrams_start = self.scorer.trigram_char_score(layout, &best_swap);

            layout.swap_xy(&best_swap);

            self.update_cache(layout, &best_swap, cache, trigrams_start);
        }

        return current_best_score;
    }

    fn optimize_columns(&self, layout: &mut FastLayout, cache: &mut StatsCache, score: Option<f64>) {
        let mut best_score = score.unwrap_or(cache.total_score);

        let mut best = layout.clone();

        self.column_permutations(layout, &mut best, cache, &mut best_score, 6);

        layout.swap_indexes();

        self.column_permutations(layout, &mut best, cache, &mut best_score, 6);

        *layout = best;
        // layout.score = best_score;
    }

    fn column_permutations(
        &self,
        layout: &mut FastLayout,
        best: &mut FastLayout,
        cache: &mut StatsCache,
        best_score: &mut f64,
        k: usize,
    ) {
        if k == 1
        {
            let new_score = cache.total_score;

            if new_score > *best_score
            {
                *best_score = new_score;
                *best = layout.clone();
            }
        }

        (0..k).for_each(|i| {
            self.column_permutations(layout, best, cache, best_score, k - 1);

            let swap = if k % 2 == 0
            {
                Pair(COLS[i], COLS[k - 1])
            } else {
                Pair(COLS[0], COLS[k - 1])
            };

            let trigrams_start = self.scorer.trigram_char_score(layout, &swap);

            layout.swap_xy(&swap);

            self.update_cache(layout, &swap, cache, trigrams_start);
        });
    }

    pub fn generate_layout(&self) -> FastLayout {
        let mut layout = FastLayout::from(
            self.language_chars_default.get(&self.language_data.language).unwrap(),
        );

        let mut cache = StatsCache::new(&self.language_data, &layout, &self.config.weights);

        self.optimize(&mut layout, &mut cache);

        // layout.score = score(language_data, &layout, (self.config).weights);

        return layout;
    }

    pub fn generate_layouts(&self, amount: usize) -> Vec<FastLayout> {
        return (0..amount).into_par_iter().map(|_| self.generate_layout()).collect();
    }

    pub fn modify_layout(&self, mut layout: &mut FastLayout, possible_swaps: Option<&[Pair]>) {
        let mut cache = StatsCache::new(&self.language_data, &layout, &self.config.weights);

        match possible_swaps {
            | Some(swaps) => self.optimize_cached(&mut layout, &mut cache, swaps),
            | None => self.optimize_cached(&mut layout, &mut cache, &self.pinned_swaps()),
        };

        // layout.score = score(language_data, &layout, (self.config).weights);
    }

    pub fn generate_n_with_pins_iter(
        &'a self,
        layout: &'a FastLayout,
        amount: usize,
    ) -> impl ParallelIterator<Item = FastLayout> + '_ {
        let possible_swaps = self.pinned_swaps();

        return (0..amount).into_par_iter().map(move |_| {
            let mut layout = FastLayout::based_on_layout(layout, Some(&self.config.pins));

            self.modify_layout(&mut layout, Some(&possible_swaps));

            layout
        });
    }

    pub fn optimize(&self, layout: &mut FastLayout, cache: &mut StatsCache) {
        let with_col_score = f64::MIN;
        let mut optimized_score = f64::MIN / 2.0;

        while with_col_score < optimized_score {
            optimized_score = self.optimize_cached(layout, cache, &self.possible_swaps);

            self.optimize_columns(layout, cache, Some(optimized_score));
            // with_col_score = layout.score;
        }

        // layout.score = optimized_score;
    }

    pub fn optimize_mut(
        &self,
        layout: &mut FastLayout,
        cache: &mut StatsCache,
        possible_swaps: &[Pair],
    ) {
        let with_col_score = f64::MIN;
        let mut optimized_score = f64::MIN / 2.0;

        while with_col_score < optimized_score {
            optimized_score = self.optimize_cached(layout, cache, possible_swaps);
            self.optimize_columns(layout, cache, Some(optimized_score));
            // with_col_score = layout.score;
        }

        // layout.score = optimized_score;
    }

    pub fn filter_trigrams(n_trigrams: &mut Vec<(NGram<u8, 3>, f64)>, c1: &u8) {
        n_trigrams.iter_mut().filter(|(t, _)| t.inner.contains(&c1)).collect_vec();
    }

    fn pinned_swaps(&self) -> Vec<Pair> {
        let mut map: Fixed<bool> = [true; 30];

        for (i, m) in map.iter_mut().enumerate() {
            if self.config.pins.pins.contains(&(i as u8))
            {
                *m = false;
            }
        }

        let mut res = Vec::new();

        for possible_swap in self.possible_swaps {
            if map[possible_swap.0] && map[possible_swap.1]
            {
                res.push(possible_swap);
            }
        }

        return res;
    }

    const fn get_possible_swaps() -> [Pair; 435] {
        let mut res = [Pair::default(); 435];
        let mut i = 0;
        let mut pos1 = 0;

        while pos1 < 30 {
            let mut pos2 = pos1 + 1;

            while pos2 < 30 {
                res[i] = Pair(pos1, pos2);
                i += 1;
                pos2 += 1;
            }

            pos1 += 1;
        }

        return res;
    }
}
