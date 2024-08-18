use crate::language::language_data::LanguageData;
use crate::layout::layout::FastLayout;
use crate::n_gram::bigram_type::BigramType;
use crate::n_gram::trigram_type::TrigramType;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

// Type Constraint
pub trait NGramPatterConstraint {}
impl NGramPatterConstraint for BigramType {}
impl NGramPatterConstraint for TrigramType {}

#[derive(Default)]
pub struct NGramStats<T: NGramPatterConstraint> {
    inner: HashMap<T, f64>,
}

impl<T: NGramPatterConstraint> NGramStats<T> {
    pub fn total_score(&self) -> f64 {
        return self.inner.values().sum();
    }
}

impl NGramStats<BigramType> {
    fn new(
        language_data: &LanguageData,
        layout: &FastLayout,
        all_bigrams: &[BigramType; 64],
    ) -> Self {
        let mut inner: HashMap<BigramType, f64> = HashMap::new();

        for (n_gram, freq) in language_data.bigrams.iter() {
            let pattern = BigramType::get_pattern(layout, n_gram, all_bigrams);

            match inner.entry(pattern) {
                | Entry::Occupied(mut v) => *v.get_mut() += freq,
                | Entry::Vacant(v) => {
                    v.insert(0.);
                }
            };
        }

        return Self { inner };
    }
}

impl NGramStats<TrigramType> {
    fn new(
        language_data: &LanguageData,
        layout: &FastLayout,
        all_trigrams: &[TrigramType; 512],
    ) -> Self {
        let mut inner: HashMap<TrigramType, f64> = HashMap::new();

        for (n_gram, freq) in language_data.trigrams.iter() {
            let pattern = TrigramType::get_pattern(layout, n_gram, all_trigrams);

            match inner.entry(pattern) {
                | Entry::Occupied(mut v) => *v.get_mut() += freq,
                | Entry::Vacant(v) => {
                    v.insert(0.);
                }
            };
        }

        return Self { inner };
    }
}
