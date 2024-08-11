use crate::{
    config::weights::weights::Weights,
    language::language_data::LanguageData,
    layout::layout::FastLayout,
    layout::layout_cache::LayoutCache,
    stat::layout_stats::NGramType::*,
    stat::trigram_stats::TrigramStats,
};

pub enum NGramType {
    SFB,
    Skipgram,
    DSFB,
    Skipgram2,
    DSFB2,
    Skipgram3,
    DSFB3,
}

#[derive(Clone)]
pub struct LayoutStats {
    pub same_finger_bigram: f64,
    pub d_same_finger_bigram: f64,
    pub d_same_finger_bigram2: f64,
    pub d_same_finger_bigram3: f64,
    pub lateral_stretch_bigrams: f64,

    pub scissors: f64,
    pub pinky_ring: f64,

    pub trigram_stats: TrigramStats,

    pub speed_score: f64,
    pub finger_speeds: [f64; 8],
}

impl LayoutStats {
    pub fn new(
        language_data: &LanguageData,
        layout: &FastLayout,
        cache: &LayoutCache,
        weights: &Weights,
    ) -> Self {
        let sfb = bigram_percent(language_data, layout, SFB);
        let dsfb = bigram_percent(language_data, layout, DSFB);
        let dsfb2 = bigram_percent(language_data, layout, DSFB2);
        let dsfb3 = bigram_percent(language_data, layout, DSFB3);

        let finger_speed = cache.finger_speed_total;
        let finger_speeds = cache.finger_speeds;

        let scissors = scissor_score(language_data, layout, weights) / weights.fingers.scissors;

        let lsbs = lateral_stretch_bigram_score(language_data, layout, weights) / weights.bigrams.lateral_stretch;

        let pinky_ring = pinky_ring_score(language_data, layout, weights) / weights.bigrams.pinky_ring;

        let trigram_stats = trigram_stats(language_data, layout, usize::MAX);

        return LayoutStats {
            same_finger_bigram: sfb,
            d_same_finger_bigram: dsfb,
            d_same_finger_bigram2: dsfb2,
            d_same_finger_bigram3: dsfb3,
            speed_score: finger_speed,
            finger_speeds,
            scissors,
            lateral_stretch_bigrams: lsbs,
            pinky_ring,
            trigram_stats,
        };
    }
}

impl std::fmt::Display for LayoutStats {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            formatter,
            concat!(
            "Sfb: {:.3}%\nDsfb: {:.3}%\nFinger Speed: {:.3}\n",
            "    [{}]\nScissors: {:.3}%\nLsbs: {:.3}%\nPinky Ring Bigrams: {:.3}%\n\n{}"
            ),
            self.same_finger_bigram * 100.0,
            self.d_same_finger_bigram * 100.0,
            self.speed_score * 10.0,
            format_finger_speed(&self.finger_speeds),
            self.scissors * 100.0,
            self.lateral_stretch_bigrams * 100.0,
            self.pinky_ring * 100.0,
            self.trigram_stats
        );
    }
}
