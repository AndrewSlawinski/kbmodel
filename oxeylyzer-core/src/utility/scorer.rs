use crate::config::weights::weights::Weights;
use crate::language::language_data::LanguageData;
use crate::layout::keyboard_type::KeyboardType;
use crate::layout::layout::FastLayout;
use crate::n_gram::bigram_type::BigramType;
use crate::n_gram::n_gram::NGram;
use crate::n_gram::trigram_type::TrigramType;
use crate::stats::bigram_stats::{
    BigramStats,
    SCISSOR_INDICES,
};
use crate::stats::layout_cache::StatsCache;
use crate::stats::trigram_stats::TrigramStats;
use crate::type_def::*;
use crate::utility::pair::Pair;
use itertools::Itertools;
use std::cmp::Ordering::Greater;

pub struct Scorer<'a> {
    language_data: &'a LanguageData,
    weights: &'a Weights,

    all_possible_bigrams: [BigramType; 64],
    all_possible_trigrams: [TrigramType; 512],

    effort_map: Fixed<f64>,
    finger_speeds: FingerSpeeds,

    stats_cache: StatsCache,
}

impl<'a> Scorer<'a> {
    pub fn new(
        language_data: &'a LanguageData,
        weights: &'a Weights,
        keyboard_type: &KeyboardType,
    ) -> Self {
        return Self {
            language_data,
            weights,
            all_possible_bigrams: BigramType::get_all_combinations(),
            all_possible_trigrams: TrigramType::get_all_combinations(),
            effort_map: keyboard_type.get_effort_map(),
            finger_speeds: Self::get_finger_speeds(&Self::get_distances(
                weights.fingers.lateral_penalty,
            )),
            stats_cache: Default::default(),
        };
    }

    pub fn total_score(
        &self,
        layout: &FastLayout,
        bigram_stats: &BigramStats,
        trigram_stats: &TrigramStats,
    ) -> f64 {
        let effort: f64 = (0..30).map(|i| self.char_effort(layout, i)).sum();

        let finger_speed_usage: f64 = (0..8).map(|col| self.column_usage(layout, col) + self.column_finger_speed(layout, col)).sum();

        // let scissors = self.scissor_score(layout);
        // let lsbs = self.lateral_stretch_bigram_score(layout);
        // let pinky_ring = self.pinky_ring_score(layout);
        // let trigram_score = self.trigram_score_iter(layout);

        // return trigram_score + effort + finger_speed_usage + scissors + lsbs + pinky_ring;
        return effort + finger_speed_usage + trigram_stats.total_score() + bigram_stats.total_score();
    }

    pub fn score_swap_cached(&self, layout: &mut FastLayout, swap: &Pair, cache: &StatsCache)
        -> f64 {
        layout.swap_xy(swap);

        let pair = Pair(FINGER_TO_COLUMN[swap.0], FINGER_TO_COLUMN[swap.1]);

        let fspeed_score = self.finger_speed_score(layout, cache, &pair);
        let usage_score = self.usage_score(layout, cache, pair);
        let effort_score = self.effort_score(layout, swap, cache);
        let scissors_score = self.scissor_score2(layout, swap, cache);
        let lsbs_score = self.lateral_scretch_bigram_score2(layout, swap, cache);

        let trigrams_score = self.sus_trigram_score(layout, swap, cache);

        return trigrams_score + scissors_score + lsbs_score + effort_score + usage_score + fspeed_score;
    }

    fn sus_trigram_score(&self, layout: &mut FastLayout, swap: &Pair, cache: &StatsCache) -> f64 {
        let trigrams_end = self.trigram_char_score(layout, swap);

        layout.swap_xy(swap);

        let trigrams_start = self.trigram_char_score(layout, swap);

        return cache.trigrams_total - trigrams_start + trigrams_end;
    }

    fn lateral_scretch_bigram_score2(
        &self,
        layout: &mut FastLayout,
        swap: &Pair,
        cache: &StatsCache,
    ) -> f64 {
        if swap.affects_lsb()
        {
            self.lateral_stretch_bigram_score(layout)
        } else {
            cache.bigram_stats[BigramType::LateralStretchBigrams]
        }
    }

    fn scissor_score2(&self, layout: &mut FastLayout, swap: &Pair, cache: &StatsCache) -> f64 {
        if swap.affects_scissor()
        {
            self.scissor_score(layout)
        } else {
            cache.bigram_stats[BigramType::Scissors]
        }
    }

    fn effort_score(&self, layout: &mut FastLayout, swap: &Pair, cache: &StatsCache) -> f64 {
        cache.char_effort_total - cache.effort_stats.char_effort[swap.0] - cache.effort_stats.char_effort[swap.1] + self.char_effort(layout, swap.0) + self.char_effort(layout, swap.1)
    }

    fn usage_score(&self, layout: &mut FastLayout, cache: &StatsCache, pair: Pair) -> f64 {
        if pair.0 == pair.1
        {
            cache.usage_total - cache.effort_stats.usage[pair.0] + self.column_usage(layout, pair.0)
        } else {
            cache.usage_total - cache.effort_stats.usage[pair.0] - cache.effort_stats.usage[pair.1] + self.column_usage(layout, pair.0) + self.column_usage(layout, pair.1)
        }
    }

    fn finger_speed_score(&self, layout: &mut FastLayout, cache: &StatsCache, pair: &Pair) -> f64 {
        if pair.0 == pair.1
        {
            cache.finger_speed_total - cache.effort_stats.finger_speeds[pair.0] + self.column_finger_speed(layout, pair.0)
        } else {
            cache.finger_speed_total - cache.effort_stats.finger_speeds[pair.0] - cache.effort_stats.finger_speeds[pair.1] + self.column_finger_speed(layout, pair.0) + self.column_finger_speed(layout, pair.1)
        }
    }

    pub fn trigram_char_score(&self, layout: &FastLayout, pair: &Pair) -> f64 {
        let mut trigram_stats = TrigramStats::new(
            self.language_data,
            layout,
            self.weights,
            &self.all_possible_trigrams,
        );

        let c0 = layout[pair.0];
        let c1 = layout[pair.1];

        match self.per_char_trigrams().get(&NGram::from(&[c0, c1])) {
            | Some(t_vec) => trigram_stats.total_score(),
            | None => 0.0,
        }
    }

    pub fn scissor_score(&self, layout: &FastLayout) -> f64 {
        let mut res = 0.0;

        for pair in SCISSOR_INDICES {
            let c0 = layout[pair.0];
            let c1 = layout[pair.1];

            res += self.language_data.bigrams.get(&NGram::from(&[c0, c1])).unwrap_or(&0.0);

            res += self.language_data.bigrams.get(&NGram::from(&[c1, c0])).unwrap_or(&0.0);
        }

        return res * self.weights.bigrams.scissors;
    }

    pub fn column_usage(&self, layout: &FastLayout, column: usize) -> f64 {
        let mut res = 0.0;

        match column {
            | 0..=2 => self.iter_columns(layout, column, &mut res),
            | 3 | 4 => {
                let col = (column - 3) * 2 + 3;

                let cu = {
                    [
                        layout[col],
                        layout[col + 10],
                        layout[col + 20],
                        layout[col + 1],
                        layout[col + 11],
                        layout[col + 21],
                    ]
                };

                for c in cu {
                    if let Some(v) = self.language_data.characters.get(&NGram::from(&[c]))
                    {
                        res += v;
                    }
                }
            }
            | 5..=7 => {
                let col = column + 2;

                self.iter_columns(layout, col, &mut res);
            }
            | _ => panic!(),
        };

        self.weights.fingers.overuse_penalty * match column {
            | 0 | 7 => (res - self.weights.fingers.bias.pinky).max(0.0),
            | 1 | 6 => (res - self.weights.fingers.bias.ring).max(0.0),
            | 2 | 5 => (res - self.weights.fingers.bias.middle).max(0.0),
            | 3 | 4 => (res - self.weights.fingers.bias.index).max(0.0),
            | _ => panic!(),
        }
    }

    fn iter_columns(&self, layout: &FastLayout, column: usize, res: &mut f64) {
        for c in [layout[column], layout[column + 10], layout[column + 20]] {
            if let Some(v) = self.language_data.characters.get(&NGram::from(&[c]))
            {
                *res += v;
            }
        }
    }

    #[inline]
    pub fn pair_finger_speed(&self, layout: &FastLayout, pair: &Pair, distance: f64) -> f64 {
        let c1 = layout[pair.0] as usize;
        let c2 = layout[pair.1] as usize;

        let mut res = 0.0;

        let len = self.language_data.characters.len();

        res += self.weighted_bigrams().get(c1 * len + c2).unwrap_or(&0.0) * distance;
        res += self.weighted_bigrams().get(c2 * len + c1).unwrap_or(&0.0) * distance;

        return res;
    }

    #[inline]
    pub fn char_effort(&self, layout: &FastLayout, i: usize) -> f64 {
        let c = layout[i];

        match self.language_data.characters.get(&NGram::from(&[c])) {
            | Some(&v) => v * self.effort_map.get(i).unwrap(),
            | None => 0.0,
        }
    }

    pub fn same_finger_bigrams(
        &self,
        layout: &FastLayout,
        top_n: usize,
    ) -> Vec<(NGram<char, 2>, f64)> {
        return self.finger_speeds.iter().map(|(p, _)| {
            let u0 = layout[p.0];
            let u1 = layout[p.1];

            let bigram0 = NGram::from(&[u0, u1]);
            let bigram1 = NGram::from(&[u1, u0]);

            let freq0 = self.language_data.bigrams[&bigram0];
            let freq1 = self.language_data.bigrams[&bigram1];

            // let bigram0 = converter.as_string(&[u0, u1]);
            // let bigram1 = converter.as_string(&[u1, u0]);
            //
            // let i0 = (u0 as usize) * self.language_data.characters.len() + (u1 as usize);
            // let i1 = (u1 as usize) * self.language_data.characters.len() + (u0 as usize);

            // let freq0 = self.language_data.bigrams[i0];
            // let freq1 = self.language_data.bigrams[i1];

            return [(bigram0, freq0), (bigram1, freq1)];
        }).flatten().sorted_by(|(_, a), (_, b)| {
            return b.partial_cmp(a).unwrap();
        }).take(top_n).collect_vec();
    }

    pub fn weighted_bigrams(&self) -> Vec<f64> {
        let mut res = Vec::new();

        for char0 in self.language_data.characters.iter().clone() {
            for char1 in self.language_data.characters.iter().clone() {
                let bigram = NGram::from(&[char0.0, char1.0]);

                let sfb = self.language_data.bigrams.get(&bigram).unwrap_or(&0.0);

                let dsfb = self.language_data.skip_grams.get(&bigram).unwrap_or(&0.0) * self.weights.bigrams.same_finger_skip;

                let dsfb2 = self.language_data.skip2_grams.get(&bigram).unwrap_or(&0.0) * self.weights.bigrams.same_finger_skip * 2_f64.recip();

                let dsfb3 = self.language_data.skip3_grams.get(&bigram).unwrap_or(&0.0) * self.weights.bigrams.same_finger_skip * 3_f64.recip();

                res.push((sfb + dsfb + dsfb2 + dsfb3) * self.weights.fingers.speed);
            }
        }

        return res;
    }

    #[inline]
    pub fn column_finger_speed(&self, layout: &FastLayout, column: usize) -> f64 {
        let (start, len) = *COLUMN_TO_START_LENGTH.get(column).unwrap();
        let mut res = 0.0;

        let finger_speeds = self.finger_speeds;

        for i in start..(start + len) {
            let (pair, dist) = finger_speeds.get(i).unwrap();

            res += self.pair_finger_speed(layout, pair, *dist);
        }

        return res;
    }

    pub fn per_char_trigrams(&self) -> Vec<(NGram<char, 3>, f64)> {
        let mut n_trigrams = Vec::new();

        self.language_data.trigrams.iter().for_each(|x| n_trigrams.push(x));

        n_trigrams.sort_by(|x1, x2| Greater);
        n_trigrams.truncate(10000);

        let thingy: Vec<(NGram<u8, 3>, f64)> = (0..u8::MAX).cartesian_product(0..u8::MAX).map(|(c1, c2)| {
            let v1 = self.iter_trigrams(&mut n_trigrams, &c1);
            let v2 = self.iter_trigrams(&mut n_trigrams, &c2);

            let (big, small, c) = if v1.len() >= v2.len()
            {
                (v1, v2, &c1)
            } else {
                (v2, v1, &c2)
            };

            let per_char = big.into_iter().chain(small.into_iter().filter(|(t, _)| !t.contains(c))).collect_vec();

            return ([c1, c2], per_char);
        }).collect();

        return thingy;
    }

    fn get_finger_speeds(distances: &[f64; 48]) -> FingerSpeeds {
        let mut res = Vec::new();

        for (b, dist) in SFB_INDICES.iter().zip(distances) {
            res.push((*b, *dist));
        }

        return res.try_into().unwrap();
    }

    #[allow(unused)]
    pub fn best_swap_cached(
        &self,
        layout: &mut FastLayout,
        cache: &StatsCache,
        current_best_score: Option<f64>,
        possible_swaps: &[Pair],
    ) -> (Option<Pair>, f64) {
        let mut best_score = current_best_score.unwrap_or(f64::MIN / 2.0);
        let mut best_swap: Option<Pair> = None;

        for swap in possible_swaps {
            let score = self.score_swap_cached(layout, swap, cache);

            if score > best_score
            {
                best_score = score;
                best_swap = Some(*swap);
            }
        }

        return (best_swap, best_score);
    }

    fn get_distances(lateral_penalty: f64) -> [f64; 48] {
        fn scale(f: f64, r: f64) -> f64 {
            return f.powi(2).powf(0.65) * r;
        }

        let mut distances = [0.0; 48];

        let fweights = [1.4, 3.6, 4.8, 4.8, 3.6, 1.4];

        let mut i = 0;

        for finger_index in 0..6 {
            let fweight = fweights[finger_index];
            let ratio = 5.5 / fweight;

            while i < ROWS {
                let scaler = 1. + if i == ROWS - 2 { 1. } else { 0. };

                distances[i] = scale(scaler, ratio);
            }
        }

        for _ in 0..2 {
            let indices = [
                (Pair(0, 0), Pair(0, 1)),
                (Pair(0, 0), Pair(0, 2)),
                (Pair(0, 0), Pair(1, 0)),
                (Pair(0, 0), Pair(1, 1)),
                (Pair(0, 0), Pair(1, 2)),
                (Pair(0, 1), Pair(0, 2)),
                (Pair(0, 1), Pair(1, 0)),
                (Pair(0, 1), Pair(1, 1)),
                (Pair(0, 1), Pair(1, 2)),
                (Pair(0, 2), Pair(1, 0)),
                (Pair(0, 2), Pair(1, 1)),
                (Pair(0, 2), Pair(1, 2)),
                (Pair(1, 0), Pair(1, 1)),
                (Pair(1, 0), Pair(1, 2)),
                (Pair(1, 1), Pair(1, 2)),
            ];

            for pair_i in 0..15 {
                let (a, b) = indices[pair_i];

                let dist = a.distance(&b).squared();

                let distance = ((dist.0 as f64) * lateral_penalty + (dist.1 as f64)).powf(0.65);

                distances[i] = distance;

                i += 1;
            }
        }

        return distances;
    }
}
