use crate::stats::bigram_stats::BigramStats;
use crate::stats::effort_stats::EffortStats;
use crate::stats::layout_stats::LayoutStats;
use crate::stats::trigram_stats::TrigramStats;

#[derive(Default)]
pub struct StatsCache {
    pub bigrams_total: f64,
    pub bigram_stats: BigramStats,

    pub trigrams_total: f64,
    pub trigram_stats: TrigramStats,

    pub usage_total: f64,
    pub char_effort_total: f64,
    pub finger_speed_total: f64,
    pub effort_stats: EffortStats,

    pub total_score: f64,
}

impl From<&LayoutStats> for StatsCache {
    fn from(value: &LayoutStats) -> Self {
        let mut new = Self {
            bigram_stats: value.bigram_stats.clone(),
            trigram_stats: value.trigram_stats.clone(),
            effort_stats: value.effort_stats.clone(),
            ..Default::default()
        };

        new.bigrams_total = value.bigram_stats.total_score();
        new.trigrams_total = value.trigram_stats.total_score();

        new.usage_total = value.effort_stats.usage_score();
        new.char_effort_total = value.effort_stats.char_effort_score();
        new.finger_speed_total = value.effort_stats.finger_speed_score();

        new.total_score = value.bigram_stats.total_score();

        return new;
    }
}

impl StatsCache {
    pub fn score(&self) -> f64 {
        return self.trigrams_total + self.bigrams_total + self.char_effort_total + self.usage_total + self.finger_speed_total;
    }
}
