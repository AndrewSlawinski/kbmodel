use crate::{
    config::weights::weights::Weights,
    generic::Fixed,
    language::language_data::LanguageData,
    layout::layout::FastLayout,
};

#[derive(Default, Debug)]
pub struct LayoutCache {
    pub effort: Fixed<f64>,
    pub effort_total: f64,

    pub scissors: f64,
    pub lsbs: f64,
    pub pinky_ring: f64,

    pub usage: [f64; 8],
    pub usage_total: f64,

    pub finger_speeds: [f64; 8],
    pub finger_speed_total: f64,

    pub trigrams_total: f64,

    pub total_score: f64,
}

impl LayoutCache {
    pub fn new(data: &LanguageData, layout: &FastLayout, weights: &Weights) -> LayoutCache {
        let mut res = LayoutCache::default();

        for i in 0..layout.matrix.len() {
            res.effort[i] = char_effort(data, layout, i);
        }

        res.effort_total = res.effort.iter().sum();

        for column in 0..8 {
            res.usage[column] = column_usage(data, layout, weights, column);
            res.finger_speeds[column] = column_finger_speed(data, layout, column)
        }

        res.usage_total = res.usage.iter().sum();
        res.finger_speed_total = res.finger_speeds.iter().sum();

        res.scissors = scissor_score(data, layout, weights);

        res.lsbs = lateral_stretch_bigram_score(data, layout, weights);

        res.pinky_ring = pinky_ring_score(data, layout, weights);

        res.trigrams_total = trigram_score_iter(layout, data.trigrams.iter().take(100000), weights);

        res.total_score = res.total_score();

        return res;
    }

    pub fn total_score(&self) -> f64 {
        return self.trigrams_total - self.scissors - self.lsbs - self.pinky_ring - self.effort_total - self.usage_total - self.finger_speed_total;
    }
}
