use crate::n_gram::bigram_type::BigramType;
use crate::n_gram::trigram_type::TrigramType;
use crate::stats::bigram_stats::BigramStats;
use crate::stats::effort_stats::EffortStats;
use crate::stats::trigram_stats::TrigramStats;
use crate::{
    config::weights::weights::Weights,
    language::language_data::LanguageData,
    layout::layout::FastLayout,
};
use std::fmt;
use std::fmt::{
    Display,
    Formatter,
};

pub enum NGramType {
    Bigram,
    Trigram,
}

#[derive(Default, Clone)]
pub struct LayoutStats {
    pub bigram_stats: BigramStats,
    pub trigram_stats: TrigramStats,
    pub effort_stats: EffortStats,
}

impl LayoutStats {
    pub fn new(
        language_data: &LanguageData,
        layout: &FastLayout,
        weights: &Weights,
        all_bigrams: &[BigramType; 64],
        all_trigrams: &[TrigramType; 512],
    ) -> Self {
        let bigram_stats = BigramStats::new(language_data, layout, weights, all_bigrams);
        let trigram_stats = TrigramStats::new(language_data, layout, weights, all_trigrams);

        return LayoutStats {
            bigram_stats,
            trigram_stats,
            effort_stats: Default::default(),
        };
    }

    pub fn format_finger_speed(&self) -> String {
        let mut finger_speed_str: Vec<String> = Vec::new();

        for (finger, v) in self.effort_stats.finger_speeds.iter() {
            finger_speed_str.push(format!("{:?}: {:.3}", finger, v));
        }

        return finger_speed_str.join(", ");
    }
}

impl Display for LayoutStats {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        return write!(
            formatter,
            "{}\n{}\n\n{}",
            self.bigram_stats,
            self.trigram_stats,
            Self::format_finger_speed(self),
        );
    }
}
