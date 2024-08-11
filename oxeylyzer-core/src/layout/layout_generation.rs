use crate::{
    config::pins::Pins,
    config::config::Config,
    generic::Bigram,
    generic::Fixed,
    language::language_data::BigramData,
    language::language_data::{
        LanguageData,
        TrigramData,
    },
    language::parse_language_cfg::chars_in_language_default,
    language::trigram_patterns::TrigramPattern::*,
    layout::keyboard_type::KeyboardType,
    layout::keyboard_type::KeyboardType::*,
    layout::layout::*,
    layout::layout_cache::LayoutCache,
    stat::layout_stats::NGramType::*,
    stat::layout_stats::*,
    stat::trigram::Trigram,
    stat::trigram_stats::TrigramStats,
    utility::converter::Converter,
    utility::pair::Pair,
};
use itertools::Itertools;
use rayon::iter::{
    IntoParallelIterator,
    ParallelIterator,
};
use std::collections::HashMap;
use std::hint::unreachable_unchecked;

pub type CharTrigramsData = HashMap<Bigram<u8>, TrigramData>;
pub type Layouts = Vec<FastLayout>;
pub type FingerSpeeds = [(Pair, f64); 48];
pub type ScissorIndices = [Pair; 17];
pub type LateralStretchBigramIndices = [Pair; 16];
pub type PinkyRingIndices = [Pair; 18];

pub struct LayoutGenerator<'a> {
    language_data: &'a LanguageData,
    config: &'a Config,
    pins: &'a Pins,

    pub u8_chars_for_generation: Fixed<u8>,
    pub finger_speed_values: FingerSpeeds,
    pub effort_map: Fixed<f64>,
    pub scissor_indices: ScissorIndices,
    pub lateral_stretch_bigram_indices: LateralStretchBigramIndices,
    pub pinky_ring_indices: PinkyRingIndices,
}

impl<'a> LayoutGenerator<'a> {
    pub fn new(language_data: &'a LanguageData, config: &'a Config) -> Self {
        let chars_fg = language_data.converter.clone().to(chars_for_generation(language_data.language.as_str()));

        let mut chars_for_generation: Fixed<u8> = chars_fg.try_into().unwrap();

        chars_for_generation.sort_by(|&a, &b| {
            let a = language_data.characters.get(a as usize).unwrap_or(&0.0);
            let b = language_data.characters.get(b as usize).unwrap_or(&0.0);

            return b.partial_cmp(a).unwrap();
        });

        Self {
            language_data: &language_data,
            config: &config,
            pins: &config.pins,
            u8_chars_for_generation: chars_for_generation,

            finger_speed_values: get_finger_speeds(config.clone().weights.fingers.lateral_penalty),
            effort_map: get_effort_map(config.weights.fingers.heatmap, &config.info.keyboard_type),

            scissor_indices: GET_SCISSOR_INDICES,
            lateral_stretch_bigram_indices: LATERAL_STRETCH_BIGRAM_INDICES,
            pinky_ring_indices: PINKY_RING_INDICES,
        }
    }

    pub unsafe fn score_swap_cached(
        &self,
        layout: &mut FastLayout,
        swap: &Pair,
        cache: &LayoutCache,
        finger_speeds: &FingerSpeeds,
    ) -> f64 {
        layout.swap_no_bounds(swap);

        let Pair(i0, i1) = *swap;

        let col0 = FINGER_TO_COLUMN[i0];
        let col1 = FINGER_TO_COLUMN[i1];

        let fspeed_score = if col0 == col1
        {
            let fspeed = self.column_finger_speed(layout, col0);

            cache.finger_speed_total - cache.finger_speeds[col0] + fspeed
        } else {
            let fspeed1 = self.column_finger_speed(layout, col0);
            let fspeed2 = self.column_finger_speed(layout, col1);

            cache.finger_speed_total - cache.finger_speeds[col0] - cache.finger_speeds[col1] + fspeed1 + fspeed2
        };

        let usage_score = if col0 == col1
        {
            let usage = self.column_usage(layout, col0);

            cache.usage_total - cache.usage[col0] + usage
        } else {
            let usage0 = self.column_usage(layout, col0);
            let usage1 = self.column_usage(layout, col1);

            cache.usage_total - cache.usage[col0] - cache.usage[col1] + usage0 + usage1
        };

        let effort0 = self.char_effort(layout, i0);
        let effort1 = self.char_effort(layout, i1);

        let effort_score = cache.effort_total - cache.effort[i0] - cache.effort[i1] + effort0 + effort1;

        let scissors_score = if swap.affects_scissor()
        {
            self.scissor_score(layout)
        } else {
            cache.scissors
        };

        let lsbs_score = if swap.affects_lsb()
        {
            self.lateral_stretch_bigram_score(layout)
        } else {
            cache.lsbs
        };

        let pinky_ring_score = if swap.affects_pinky_ring()
        {
            self.pinky_ring_score(layout)
        } else {
            cache.pinky_ring
        };

        // let _new_heur = cache.trigrams_total - scissors_score - effort_score - usage_score - fspeed_score;

        let trigrams_score = if cache.total_score < f64::MAX
        {
            //new_heur + new_heur.abs() * 0.0) {
            let trigrams_end = self.trigram_char_score(layout, swap);

            unsafe { layout.swap_no_bounds(swap) };

            let trigrams_start = self.trigram_char_score(layout, swap);

            #[cfg(test)]
            NOT_PRUNED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            cache.trigrams_total - trigrams_start + trigrams_end
        } else {
            #[cfg(test)]
            PRUNED_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            unsafe { layout.swap_no_bounds(swap) };

            return f64::MIN + 1000.0;
        };

        return trigrams_score - scissors_score - lsbs_score - pinky_ring_score - effort_score - usage_score - fspeed_score;
    }

    #[inline]
    pub unsafe fn column_finger_speed(&self, layout: &FastLayout, column: usize) -> f64 {
        let (start, len) = column_to_start_length(column);
        let mut res = 0.0;

        for i in start..(start + len) {
            let (pair, dist) = self.finger_speed_values.get_unchecked(i);

            res += self.pair_finger_speed(layout, pair, *dist);
        }

        return res;

        #[inline(always)]
        unsafe fn column_to_start_length(column: usize) -> (usize, usize) {
            *[
                (0, 3),
                (3, 3),
                (6, 3),
                (18, 15),
                (33, 15),
                (9, 3),
                (12, 3),
                (15, 3),
            ].get_unchecked(column)
        }
    }

    pub unsafe fn accept_swap(&self, layout: &mut FastLayout, swap: &Pair, cache: &mut LayoutCache) {
        let trigrams_start = self.trigram_char_score(layout, swap);

        layout.swap_no_bounds(swap);

        let Pair(i0, i1) = *swap;

        let col0 = FINGER_TO_COLUMN[i0];
        let col1 = FINGER_TO_COLUMN[i1];

        cache.finger_speed_total = if col0 == col1
        {
            let fspeed = self.column_finger_speed(layout, col0);
            let total = cache.finger_speed_total - cache.finger_speeds[col0] + fspeed;

            cache.finger_speeds[col0] = fspeed;

            total
        } else {
            let fspeed0 = self.column_finger_speed(layout, col0);
            let fspeed1 = self.column_finger_speed(layout, col1);

            let total = cache.finger_speed_total - cache.finger_speeds[col0] - cache.finger_speeds[col1] + fspeed0 + fspeed1;

            cache.finger_speeds[col0] = fspeed0;
            cache.finger_speeds[col1] = fspeed1;

            total
        };

        cache.usage_total = if col0 == col1
        {
            let usage = self.column_usage(layout, col0);
            let total = cache.usage_total - cache.usage[col0] + usage;

            cache.usage[col0] = usage;

            total
        } else {
            let usage1 = self.column_usage(layout, col0);
            let usage2 = self.column_usage(layout, col1);
            let total = cache.usage_total - cache.usage[col0] - cache.usage[col1] + usage1 + usage2;

            cache.usage[col0] = usage1;
            cache.usage[col1] = usage2;

            total
        };

        let effort1 = self.char_effort(layout, i0);
        let effort2 = self.char_effort(layout, i1);

        cache.effort_total = cache.effort_total - cache.effort[i0] - cache.effort[i1] + effort1 + effort2;

        cache.effort[i0] = effort1;
        cache.effort[i1] = effort2;

        let trigrams_end = self.trigram_char_score(layout, swap);

        cache.trigrams_total = cache.trigrams_total - trigrams_start + trigrams_end;

        if swap.affects_scissor()
        {
            cache.scissors = self.scissor_score(layout);
        }

        if swap.affects_lsb()
        {
            cache.lsbs = self.lateral_stretch_bigram_score(layout);
        }

        if swap.affects_pinky_ring()
        {
            cache.pinky_ring = self.pinky_ring_score(layout);
        }

        cache.total_score = cache.total_score();
    }

    pub fn best_swap_cached(
        &self,
        layout: &mut FastLayout,
        cache: &LayoutCache,
        current_best_score: Option<f64>,
        possible_swaps: &[Pair],
    ) -> (Option<Pair>, f64) {
        let mut best_score = current_best_score.unwrap_or(f64::MIN / 2.0);
        let mut best_swap: Option<Pair> = None;

        // for swap in possible_swaps
        // {
        //     // let score = self.score_swap_cached(layout, swap, cache);
        //
        //     // if score > best_score
        //     // {
        //     //     best_score = score;
        //     //     best_swap = Some(*swap);
        //     // }
        // }

        return (best_swap, best_score);
    }

    unsafe fn optimize_cached(
        &self,
        layout: &mut FastLayout,
        cache: &mut LayoutCache,
        possible_swaps: &[Pair],
    ) -> f64 {
        let mut current_best_score = f64::MIN / 2.0;

        while let (Some(best_swap), new_score) = self.best_swap_cached(layout, cache, Some(current_best_score), possible_swaps) {
            current_best_score = new_score;
            self.accept_swap(layout, &best_swap, cache);
        }

        return current_best_score;
    }

    fn optimize_columns(&self, layout: &mut FastLayout, cache: &mut LayoutCache, score: Option<f64>) {
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
        cache: &mut LayoutCache,
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

        (0..k).for_each(|i| unsafe {
            self.column_permutations(layout, best, cache, best_score, k - 1);
            if k % 2 == 0
            {
                self.accept_swap(layout, &Pair(COLS[i], COLS[k - 1]), cache);
            } else {
                self.accept_swap(layout, &Pair(COLS[0], COLS[k - 1]), cache);
            }
        });
    }

    pub unsafe fn generate_layout(&self) -> FastLayout {
        let mut layout = FastLayout::random(self.u8_chars_for_generation);
        let mut cache = LayoutCache::new(&self.language_data, &layout, &self.config.weights);

        self.optimize(&mut layout, &mut cache, &get_possible_swaps());

        // layout.score = score(language_data, &layout, (self.config).weights);

        return layout;
    }

    pub unsafe fn optimize(
        &self,
        layout: &mut FastLayout,
        cache: &mut LayoutCache,
        possible_swaps: &[Pair],
    ) {
        let mut with_col_score = f64::MIN;
        let mut optimized_score = f64::MIN / 2.0;

        while with_col_score < optimized_score {
            optimized_score = self.optimize_cached(layout, cache, possible_swaps);
            self.optimize_columns(layout, cache, Some(optimized_score));
            // with_col_score = layout.score;
        }

        // layout.score = optimized_score;
    }

    pub unsafe fn optimize_mut(
        &self,
        layout: &mut FastLayout,
        cache: &mut LayoutCache,
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

    pub unsafe fn generate_n_with_pins_iter(
        &self,
        based_on: &FastLayout,
        amount: usize,
    ) -> impl ParallelIterator<Item = FastLayout> + '_ {
        let possible_swaps = self.pinned_swaps();

        return (0..amount).into_par_iter().map(move |_| self.generate_with_pins(based_on, Some(&possible_swaps.clone())));
    }

    pub unsafe fn generate_with_pins(
        &self,
        based_on: &FastLayout,
        possible_swaps: Option<&[Pair]>,
    ) -> FastLayout {
        let mut layout = FastLayout::random_pins(based_on.matrix, &self.config.pins);
        let mut cache = LayoutCache::new(&self.language_data, &layout, &self.config.weights);

        match possible_swaps {
            | Some(ps) => self.optimize_cached(&mut layout, &mut cache, ps),
            | None => self.optimize_cached(&mut layout, &mut cache, &self.pinned_swaps()),
        };

        // layout.score = score(language_data, &layout, (self.config).weights);

        return layout;
    }

    pub unsafe fn pinky_ring_score(&self, layout: &FastLayout) -> f64 {
        let mut res = 0.0;
        let len = self.language_data.characters.len();

        for Pair(i0, i1) in self.pinky_ring_indices {
            let c0 = layout.cu(i0) as usize;
            let c1 = layout.cu(i1) as usize;

            res += self.language_data.bigrams.get(c0 * len + c1).unwrap_or(&0.0);

            res += self.language_data.bigrams.get(c1 * len + c0).unwrap_or(&0.0);
        }

        return res * self.config.weights.bigrams.pinky_ring;
    }

    unsafe fn iter_columns(&self, layout: &FastLayout, column: usize, res: &mut f64) {
        for c in [
            layout.cu(column),
            layout.cu(column + 10),
            layout.cu(column + 20),
        ] {
            if let Some(v) = self.language_data.characters.get(c as usize)
            {
                *res += v;
            }
        }
    }

    pub unsafe fn column_usage(&self, layout: &FastLayout, column: usize) -> f64 {
        let mut res = 0.0;

        match column {
            | 0..=2 => self.iter_columns(layout, column, &mut res),
            | 3 | 4 => {
                let col = (column - 3) * 2 + 3;

                let cu = [
                    layout.cu(col),
                    layout.cu(col + 10),
                    layout.cu(col + 20),
                    layout.cu(col + 1),
                    layout.cu(col + 11),
                    layout.cu(col + 21),
                ];

                for c in cu {
                    if let Some(v) = self.language_data.characters.get(c as usize)
                    {
                        res += v;
                    }
                }
            }
            | 5..=7 => {
                let col = column + 2;

                self.iter_columns(layout, col, &mut res);
            }
            | _ => unreachable_unchecked(),
        };

        self.config.weights.fingers.overuse_penalty * match column {
            | 0 | 7 => (res - self.config.weights.fingers.bias.pinky).max(0.0),
            | 1 | 6 => (res - self.config.weights.fingers.bias.ring).max(0.0),
            | 2 | 5 => (res - self.config.weights.fingers.bias.middle).max(0.0),
            | 3 | 4 => (res - self.config.weights.fingers.bias.index).max(0.0),
            | _ => unreachable_unchecked(),
        }
    }

    #[inline]
    pub unsafe fn pair_finger_speed(&self, layout: &FastLayout, pair: &Pair, distance: f64) -> f64 {
        let c1 = unsafe { layout.cu(pair.0) } as usize;
        let c2 = unsafe { layout.cu(pair.1) } as usize;

        let mut res = 0.0;

        let len = self.language_data.characters.len();

        res += self.weighted_bigrams().get(c1 * len + c2).unwrap_or(&0.0) * distance;
        res += self.weighted_bigrams().get(c2 * len + c1).unwrap_or(&0.0) * distance;

        return res;
    }

    #[inline]
    pub unsafe fn char_effort(&self, layout: &FastLayout, i: usize) -> f64 {
        let c = unsafe { layout.cu(i) };

        match self.language_data.characters.get(c as usize) {
            | Some(&v) => v * unsafe { self.effort_map.get_unchecked(i) },
            | None => 0.0,
        }
    }

    pub fn format_finger_speed(finger_speed: &[f64]) -> String {
        let mut finger_speed_str: Vec<String> = Vec::new();

        for v in finger_speed {
            finger_speed_str.push(format!("{:.3}", v * 10.0));
        }

        return finger_speed_str.join(", ");
    }

    pub unsafe fn bigram_percent(
        &self,
        layout: &FastLayout,
        finger_speeds: &FingerSpeeds,
        bigram_type: NGramType,
    ) -> f64 {
        let data: BigramData = match bigram_type {
            | SFB => self.language_data.bigrams.clone(),
            | Skipgram | DSFB => self.language_data.skipgrams.clone(),
            | Skipgram2 | DSFB2 => self.language_data.skipgrams2.clone(),
            | Skipgram3 | DSFB3 => self.language_data.skipgrams3.clone(),
        };

        let mut res = 0.0;
        let len = self.language_data.characters.len();

        for (Pair(i1, i2), _) in finger_speeds {
            let c1 = layout.cu(*i1) as usize;
            let c2 = layout.cu(*i2) as usize;

            res += data.get(c1 * len + c2).unwrap_or(&0.0);
            res += data.get(c2 * len + c1).unwrap_or(&0.0);
        }

        return res;
    }

    pub fn same_finger_bigrams(
        &self,
        layout: &FastLayout,
        converter: &Converter,
        top_n: usize,
    ) -> Vec<(String, f64)> {
        return self.finger_speed_values.iter().map(|(p, _)| unsafe {
            let u1 = layout.c(p.0);
            let u2 = layout.c(p.1);

            let bigram = converter.as_string(&[u1, u2]);
            let bigram2 = converter.as_string(&[u2, u1]);

            let i = (u1 as usize) * self.language_data.characters.len() + (u2 as usize);
            let i2 = (u2 as usize) * self.language_data.characters.len() + (u1 as usize);

            let freq = self.language_data.bigrams[i];
            let freq2 = self.language_data.bigrams[i2];

            return [(bigram, freq), (bigram2, freq2)];
        }).flatten().sorted_by(|(_, a), (_, b)| {
            return b.partial_cmp(a).unwrap();
        }).take(top_n).collect::<Vec<_>>();
    }

    pub unsafe fn trigram_stats(&self, layout: &FastLayout) -> TrigramStats {
        let mut freqs = TrigramStats::default();

        for (trigram, freq) in self.language_data.trigrams.iter() {
            let freq = freq * 100.0;

            match layout.get_trigram_pattern(trigram) {
                | Alternate => freqs.alternates += freq,
                | AlternateSfs => freqs.alternates_same_finger_skipgrams += freq,
                | Inroll => freqs.inrolls += freq,
                | Outroll => freqs.outrolls += freq,
                | Onehand => freqs.one_hands += freq,
                | Redirect => freqs.redirects += freq,
                | RedirectSfs => freqs.redirects_same_finger_skipgrams += freq,
                | BadRedirect => freqs.bad_redirects += freq,
                | BadRedirectSfs => freqs.bad_redirects_same_finger_skipgrams += freq,
                | Sfb => freqs.same_finger_bigrams += freq,
                | BadSfb => freqs.bad_same_finger_bigrams += freq,
                | Sft => freqs.same_finger_trigrams += freq,
                | Other => freqs.other += freq,
                | Invalid => freqs.invalid += freq,
            }
        }

        return freqs;
    }

    pub unsafe fn score(&self, layout: &FastLayout) -> f64 {
        let effort = (0..layout.matrix.len()).map(|i| self.char_effort(layout, i)).sum::<f64>();

        let fspeed_usage = (0..8).map(|col| self.column_usage(layout, col) + self.column_finger_speed(layout, col)).sum::<f64>();

        let scissors = self.scissor_score(layout);
        let lsbs = self.lateral_stretch_bigram_score(layout);
        let pinky_ring = self.pinky_ring_score(layout);
        let trigram_score = self.trigram_score_iter(layout);

        return trigram_score - effort - fspeed_usage - scissors - lsbs - pinky_ring;
    }

    pub unsafe fn weighted_bigrams(&self) -> BigramData {
        let len = self.language_data.characters.len();
        let chars = 0..len;

        return chars.clone().cartesian_product(chars).map(|(c1, c2)| {
            let bigram = c1 * len + c2;
            let sfb = self.language_data.bigrams.get(bigram).unwrap_or(&0.0);

            let dsfb = self.language_data.skipgrams.get(bigram).unwrap_or(&0.0) * self.config.weights.bigrams.d_same_finger_ratio;

            let dsfb2 = self.language_data.skipgrams2.get(bigram).unwrap_or(&0.0) * self.config.weights.bigrams.d_same_finger_ratio1;

            let dsfb3 = self.language_data.skipgrams3.get(bigram).unwrap_or(&0.0) * self.config.weights.bigrams.d_same_finger_ratio2;

            return (sfb + dsfb + dsfb2 + dsfb3) * self.config.weights.fingers.speed;
        }).collect();
    }

    pub unsafe fn per_char_trigrams(&self, highest: u8, trigram_precision: u32)
        -> CharTrigramsData {
        let mut n_trigrams = self.language_data.trigrams.iter().clone();

        n_trigrams.truncate(trigram_precision as usize);

        let thingy: Vec<(Trigram<u8>, TrigramData)> = (0..highest).cartesian_product(0..highest).map(|(c1, c2)| {
            let v1 = self.iter_trigrams(&mut n_trigrams, &c1);
            let v2 = self.iter_trigrams(&mut n_trigrams, &c2);

            let (big, small, c) = if v1.len() >= v2.len()
            {
                (v1, v2, &c1)
            } else {
                (v2, v1, &c2)
            };

            let per_char = big.into_iter().chain(small.into_iter().filter(|(t, _)| !t.contains(c))).collect::<Vec<_>>();

            return ([c1, c2], per_char);
        }).collect();

        return CharTrigramsData::from_iter(thingy);
    }

    pub fn iter_trigrams(
        n_trigrams: &mut Vec<(Trigram<u8>, f64)>,
        c1: &u8,
    ) -> Vec<(Trigram<u8>, f64)> {
        return n_trigrams.iter().filter(|(t, _)| t.contains(&c1)).collect::<Vec<_>>();
    }

    #[inline]
    pub unsafe fn trigram_score_iter(&self, layout: &FastLayout) -> f64 {
        let mut freqs = TrigramStats::default();

        for (trigram, freq) in self.language_data.trigrams {
            match layout.get_trigram_pattern(trigram) {
                | Alternate => freqs.alternates += freq,
                | AlternateSfs => freqs.alternates_same_finger_skipgrams += freq,
                | Inroll => freqs.inrolls += freq,
                | Outroll => freqs.outrolls += freq,
                | Onehand => freqs.one_hands += freq,
                | Redirect => freqs.redirects += freq,
                | RedirectSfs => freqs.redirects += freq,
                | BadRedirect => freqs.bad_redirects += freq,
                | BadRedirectSfs => freqs.bad_redirects += freq,
                | _ => {}
            }
        }

        let mut score = 0.0;

        score += self.config.weights.rolls.inward * freqs.inrolls;
        score += self.config.weights.rolls.outward * freqs.outrolls;
        score += self.config.weights.fingers.onehands * freqs.one_hands;
        score += self.config.weights.alternates.base * freqs.alternates;
        score += self.config.weights.alternates.same_finger_skip * freqs.alternates_same_finger_skipgrams;

        score -= self.config.weights.redirects.base * freqs.redirects;
        score -= self.config.weights.redirects.same_finger_skips * freqs.redirects_same_finger_skipgrams;
        score -= self.config.weights.redirects.bad * freqs.bad_redirects;
        score -= self.config.weights.redirects.bad_same_finger_skips * freqs.bad_redirects_same_finger_skipgrams;

        return score;
    }

    pub unsafe fn trigram_char_score(&self, layout: &FastLayout, pos: &Pair) -> f64 {
        let c1 = layout.cu(pos.0);
        let c2 = layout.cu(pos.1);

        match self.per_char_trigrams().get(&[c1, c2]) {
            | Some(t_vec) => self.trigram_score_iter(layout, t_vec),
            | None => 0.0,
        }
    }

    #[inline]
    pub unsafe fn scissor_score(&self, layout: &FastLayout) -> f64 {
        let mut res = 0.0;
        let len = self.language_data.characters.len();

        for Pair(i1, i2) in self.scissor_indices {
            let c1 = unsafe { layout.cu(i1) } as usize;
            let c2 = unsafe { layout.cu(i2) } as usize;

            res += self.language_data.bigrams.get(c1 * len + c2).unwrap_or(&0.0);

            res += self.language_data.bigrams.get(c2 * len + c1).unwrap_or(&0.0);
        }

        return res * self.config.weights.fingers.scissors;
    }

    unsafe fn pinned_swaps(&self) -> Vec<Pair> {
        let mut map: Fixed<bool> = [true; 30];

        for (i, m) in map.iter_mut().enumerate() {
            if self.config.pins.pins.contains(&(i as u8))
            {
                *m = false;
            }
        }

        let mut res = Vec::new();

        for possible_swap in get_possible_swaps() {
            if map[possible_swap.0] && map[possible_swap.1]
            {
                res.push(possible_swap);
            }
        }

        return res;
    }

    #[inline]
    pub unsafe fn lateral_stretch_bigram_score(&self, layout: &FastLayout) -> f64 {
        let mut res = 0.0;
        let len = self.language_data.characters.len();

        for Pair(i0, i1) in self.lateral_stretch_bigram_indices {
            let c1 = unsafe { layout.cu(i0.clone()) } as usize;
            let c2 = unsafe { layout.cu(i1.clone()) } as usize;

            res += self.language_data.bigrams.get(c1 * len + c2).unwrap_or(&0.0);

            res += self.language_data.bigrams.get(c2 * len + c1).unwrap_or(&0.0);
        }

        return res * self.config.weights.bigrams.lateral_stretch;
    }
}

const COLS: [usize; 6] = [0, 1, 2, 7, 8, 9];

fn get_effort_map(heatmap_weight: f64, k_type: &KeyboardType) -> Fixed<f64> {
    let mut res = match k_type {
        | IsoAngle => {
            [
                3.0, 2.4, 2.0, 2.2, 2.4, 3.3, 2.2, 2.0, 2.4, 3.0, 1.8, 1.3, 1.1, 1.0, 2.6, 2.6,
                1.0, 1.1, 1.3, 1.8, 3.3, 2.8, 2.4, 1.8, 2.2, 2.2, 1.8, 2.4, 2.8, 3.3,
            ]
        }
        | AnsiAngle => {
            [
                3.0, 2.4, 2.0, 2.2, 2.4, 3.3, 2.2, 2.0, 2.4, 3.0, 1.8, 1.3, 1.1, 1.0, 2.6, 2.6,
                1.0, 1.1, 1.3, 1.8, 3.7, 2.8, 2.4, 1.8, 2.2, 2.2, 1.8, 2.4, 2.8, 3.3,
            ]
        }
        | RowstagDefault => {
            [
                3.0, 2.4, 2.0, 2.2, 2.4, 3.3, 2.2, 2.0, 2.4, 3.0, 1.8, 1.3, 1.1, 1.0, 2.6, 2.6,
                1.0, 1.1, 1.3, 1.8, 3.5, 3.0, 2.7, 2.3, 3.7, 2.2, 1.8, 2.4, 2.8, 3.3,
            ]
        }
        | Ortho => {
            [
                3.0, 2.4, 2.0, 2.2, 3.1, 3.1, 2.2, 2.0, 2.4, 3.0, 1.7, 1.3, 1.1, 1.0, 2.6, 2.6,
                1.0, 1.1, 1.3, 1.7, 3.2, 2.6, 2.3, 1.6, 3.0, 3.0, 1.6, 2.3, 2.6, 3.2,
            ]
        }
        | Colstag => {
            [
                3.0, 2.4, 2.0, 2.2, 3.1, 3.1, 2.2, 2.0, 2.4, 3.0, 1.7, 1.3, 1.1, 1.0, 2.6, 2.6,
                1.0, 1.1, 1.3, 1.7, 3.4, 2.6, 2.2, 1.8, 3.2, 3.2, 1.8, 2.2, 2.6, 3.4,
            ]
        }
    };

    for r in &mut res {
        *r -= 0.2;
        *r /= 4.5;
        *r *= heatmap_weight;
    }

    return res;
}

fn get_finger_speeds(lat_multiplier: f64) -> [(Pair, f64); 48] {
    let mut res = Vec::new();

    for (b, dist) in sfb_indices().iter().zip(get_distances(lat_multiplier)) {
        res.push((*b, dist));
    }

    return res.try_into().unwrap();
}

fn get_distances(lat_multiplier: f64) -> [f64; 48] {
    let mut i = 0;
    let mut res = [0.0; 48];

    let mut fweight_i = 0;
    let fweights = [1.4, 3.6, 4.8, 4.8, 3.6, 1.4];

    let help = |f: f64, r: f64| f.powi(2).powf(0.65) * r;

    while fweight_i < 6 {
        let fweight = fweights[fweight_i];
        let ratio = 5.5 / fweight;

        res[i] = help(1.0, ratio);
        res[i + 1] = help(2.0, ratio);
        res[i + 2] = help(1.0, ratio);

        fweight_i += 1;
        i += 3;
    }

    let mut c = 0;

    while c <= 2 {
        let index = [
            ((0, 0), (0, 1)),
            ((0, 0), (0, 2)),
            ((0, 0), (1, 0)),
            ((0, 0), (1, 1)),
            ((0, 0), (1, 2)),
            ((0, 1), (0, 2)),
            ((0, 1), (1, 0)),
            ((0, 1), (1, 1)),
            ((0, 1), (1, 2)),
            ((0, 2), (1, 0)),
            ((0, 2), (1, 1)),
            ((0, 2), (1, 2)),
            ((1, 0), (1, 1)),
            ((1, 0), (1, 2)),
            ((1, 1), (1, 2)),
        ];

        let mut pair_i = 0;

        while pair_i < 15 {
            let ((x1, y1), (x2, y2)) = index[pair_i];

            let x_dist = (x1 - x2) as f64;
            let y_dist = (y1 - y2) as f64;
            let distance = (x_dist.powi(2) * lat_multiplier + y_dist.powi(2)).powf(0.65);

            res[i] = distance;

            i += 1;
            pair_i += 1;
        }

        c += 2;
    }

    return res;
}

const fn sfb_indices() -> [Pair; 48] {
    let mut res = [Pair::default(); 48];
    let mut i = 0;

    let mut col_i = 0;
    let cols = [0, 1, 2, 7, 8, 9];

    while col_i < cols.len() {
        let col = cols[col_i];

        res[i] = Pair(col, col + 10);
        res[i + 1] = Pair(col, col + 20);
        res[i + 2] = Pair(col + 10, col + 20);

        col_i += 1;
        i += 3;
    }

    let mut c = 0;

    while c <= 2 {
        let index = [
            (3 + c, 13 + c),
            (3 + c, 23 + c),
            (3 + c, 4 + c),
            (3 + c, 14 + c),
            (3 + c, 24 + c),
            (13 + c, 23 + c),
            (13 + c, 4 + c),
            (13 + c, 14 + c),
            (13 + c, 24 + c),
            (23 + c, 4 + c),
            (23 + c, 14 + c),
            (23 + c, 24 + c),
            (4 + c, 14 + c),
            (4 + c, 24 + c),
            (14 + c, 24 + c),
        ];

        let mut pair_i = 0;

        while pair_i < 15 {
            res[i] = Pair(index[pair_i].0, index[pair_i].1);

            i += 1;
            pair_i += 1;
        }

        c += 2;
    }

    return res;
}

const LATERAL_STRETCH_BIGRAM_INDICES: LateralStretchBigramIndices = [
    // left
    Pair(2, 4),
    Pair(2, 14),
    Pair(2, 24),
    Pair(12, 4),
    Pair(12, 14),
    Pair(22, 4),
    Pair(22, 14),
    Pair(22, 24),
    // right
    Pair(5, 7),
    Pair(5, 17),
    Pair(5, 27),
    Pair(15, 7),
    Pair(15, 17),
    Pair(15, 27),
    Pair(25, 17),
    Pair(25, 27),
];

const PINKY_RING_INDICES: PinkyRingIndices = [
    Pair(0, 1),
    Pair(0, 11),
    Pair(0, 21),
    Pair(11, 1),
    Pair(11, 11),
    Pair(11, 21),
    Pair(21, 1),
    Pair(21, 11),
    Pair(21, 21),
    // right
    Pair(8, 9),
    Pair(8, 19),
    Pair(8, 29),
    Pair(18, 9),
    Pair(18, 19),
    Pair(18, 29),
    Pair(28, 9),
    Pair(28, 19),
    Pair(28, 29),
];

const GET_SCISSOR_INDICES: ScissorIndices = [
    Pair(0, 21),
    Pair(1, 22),
    Pair(6, 27),
    Pair(7, 28),
    Pair(8, 29),
    Pair(1, 20),
    Pair(2, 21),
    Pair(3, 22),
    Pair(8, 27),
    Pair(9, 28),
    //pinky->ring 1u stretches
    Pair(0, 11),
    Pair(9, 18),
    Pair(10, 21),
    Pair(19, 28),
    //inner index scissors (no qwerty `ni` because of stagger)
    Pair(2, 24),
    Pair(22, 4),
    Pair(5, 27),
];

fn chars_for_generation(language: &str) -> Fixed<char> {
    let languages_cfg_map = chars_in_language_default();

    return match languages_cfg_map.get(language) {
        | Some(cfg) => cfg.chars().collect::<Vec<char>>().try_into().unwrap(),
        | None => {
            let default = languages_cfg_map.get("default").unwrap();

            return default.chars().collect::<Vec<char>>().try_into().unwrap();
        }
    };
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

#[cfg(test)]
pub static PRUNED_COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
#[cfg(test)]
pub static NOT_PRUNED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
