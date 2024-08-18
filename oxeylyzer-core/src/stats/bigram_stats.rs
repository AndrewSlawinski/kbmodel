use crate::config::weights::weights::Weights;
use crate::language::language_data::LanguageData;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::layout::layout::FastLayout;
use crate::n_gram::bigram_type::BigramType;
use crate::n_gram::n_gram::NGram;
use crate::type_def::FingerSpeeds;
use crate::utility::pair::Pair;
use std::fmt;
use std::fmt::{
    Display,
    Formatter,
};
use std::ops::Index;

#[derive(Default, Clone)]
pub struct BigramStats {
    inner: HashMap<BigramType, f64>,
}

impl Index<BigramType> for BigramStats {
    type Output = f64;

    fn index(&self, index: BigramType) -> &Self::Output {
        return &self.inner[&index];
    }
}

impl BigramStats {
    pub fn new(
        language_data: &LanguageData,
        layout: &FastLayout,
        weights: &Weights,
        all_bigrams: &[BigramType; 64],
    ) -> Self {
        let mut stats = BigramStats::default();

        for (bigram, freq) in language_data.bigrams.iter() {
            let pattern = BigramType::get_pattern(layout, bigram, all_bigrams);

            match stats.inner.entry(pattern) {
                | Entry::Occupied(mut v) => {
                    *v.get_mut() += freq;
                    *v.get_mut() *= match pattern {
                        | BigramType::Scissors => weights.bigrams.scissors,
                        | BigramType::SameFingerBigram => 1.,
                        | BigramType::SameFingerSkipGram => weights.bigrams.same_finger_skip,
                        | BigramType::SameFingerSkip2Gram => {
                            weights.bigrams.same_finger_skip * 2_f64.recip()
                        }
                        | BigramType::SameFingerSkip3Gram => {
                            weights.bigrams.same_finger_skip * 3_f64.recip()
                        }
                        | BigramType::LateralStretchBigrams => weights.bigrams.lateral_stretch,
                        | BigramType::Other => 0.,
                        | BigramType::Invalid => 0.,
                    }
                }
                | Entry::Vacant(v) => {
                    v.insert(0.);
                }
            };
        }

        return stats;

        // let stats = Self::bigram_percent(language_data, layout, finger_speeds);
        //
        // return Self {
        //     inner: Default::default(),
        //     scissors: Self::scissor_score(language_data, layout, weights),
        //     pinky_ring: Self::pinky_ring_score(language_data, layout, weights),
        //     same_finger_bigram: stats[0],
        //     same_finger_skipgram: stats[1],
        //     same_finger_skip2_gram: stats[2],
        //     same_finger_skip3_gram: stats[3],
        //     lateral_stretch_bigrams: Self::lateral_stretch_bigram_score(
        //         language_data,
        //         layout,
        //         weights,
        //     ) / weights.bigrams.lateral_stretch,
        // };
    }

    pub fn total_score(&self) -> f64 {
        return self.inner.values().sum();
    }

    fn bigram_percent(
        language_data: &LanguageData,
        layout: &FastLayout,
        finger_speeds: &FingerSpeeds,
    ) -> Vec<f64> {
        let mut stats = Vec::new();

        for bigram in vec![
            &language_data.bigrams,
            &language_data.skip_grams,
            &language_data.skip2_grams,
            &language_data.skip3_grams,
        ] {
            let mut res = 0.0;

            for (pair, _) in finger_speeds {
                let c0 = layout[pair.0];
                let c1 = layout[pair.1];

                res += bigram.get(&NGram::from(&[c0, c1])).unwrap_or(&0.0);
                res += bigram.get(&NGram::from(&[c1, c0])).unwrap_or(&0.0);
            }

            stats.push(res);
        }

        return stats;
    }

    #[inline]
    pub fn lateral_stretch_bigram_score(
        language_data: &LanguageData,
        layout: &FastLayout,
        weights: &Weights,
        indices: &[Pair],
    ) -> f64 {
        let mut res = 0.0;

        for pair in indices {
            let c0 = layout[pair.0];
            let c1 = layout[pair.1];

            res += language_data.bigrams.get(&NGram::from(&[c0, c1])).unwrap_or(&0.0);

            res += language_data.bigrams.get(&NGram::from(&[c1, c0])).unwrap_or(&0.0);
        }

        return res * weights.bigrams.lateral_stretch;
    }
}

impl Display for BigramStats {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut format = "".to_string();

        self.inner.iter().for_each(|(key, value)| {
            let s = format!("{:?}: {:.3}%\n", key.clone(), value.clone());

            format.push_str(s.as_str());
        });

        write!(f, "{}", format.clone())
    }
}

pub const LATERAL_STRETCH_BIGRAM_INDICES: [Pair; 16] = [
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

pub const PINKY_RING_INDICES: [Pair; 18] = [
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

pub const SCISSOR_INDICES: [Pair; 17] = [
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
