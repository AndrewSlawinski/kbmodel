use crate::config::weights::weights::Weights;
use crate::language::language_data::LanguageData;
use crate::layout::layout::FastLayout;
use crate::n_gram::trigram_type::TrigramType;
use fmt::Display;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt;

#[derive(Clone, Default)]
pub struct TrigramStats {
    pub inner: HashMap<TrigramType, f64>,
}

impl TrigramStats {
    pub fn new(
        language_data: &LanguageData,
        layout: &FastLayout,
        weights: &Weights,
        all_trigrams: &[TrigramType; 512],
    ) -> Self {
        let mut stats = TrigramStats::default();

        for (trigram, freq) in language_data.trigrams.iter() {
            let pattern = TrigramType::get_pattern(layout, trigram, all_trigrams);

            match stats.inner.entry(pattern) {
                | Entry::Occupied(mut v) => {
                    *v.get_mut() += freq;
                    *v.get_mut() *= match pattern {
                        | TrigramType::Alternate => weights.alternates.base,
                        | TrigramType::Inroll => weights.rolls.inward,
                        | TrigramType::Outroll => weights.rolls.outward,
                        | TrigramType::Redirect => weights.redirects.base,
                        | TrigramType::SameFingerTrigram => 1.,
                        | TrigramType::Other => 1.,
                        | TrigramType::Invalid => 1.,
                    }
                }
                | Entry::Vacant(v) => {
                    v.insert(0.);
                }
            };
        }

        return stats;
    }

    pub fn total_score(&self) -> f64 {
        return self.inner.values().sum();
    }
}

impl Display for TrigramStats {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = "".to_string();

        for (key, value) in self.inner.iter() {
            output = format!("{output}{:?}: {:.3}\n", key, value).to_string()
        }

        return write!(formatter, "{}", output);
    }
}
