use crate::stats::bigram_stats::BigramStats;

pub struct Scorer {}

impl Scorer
{
    pub fn total_score(&self, bigram_stats: &BigramStats) -> f32
    {
        // let scissors = self.scissor_score(layout);
        // let lsbs = self.lateral_stretch_bigram_score(layout);
        // let pinky_ring = self.pinky_ring_score(layout);

        todo!()
    }

    pub fn same_finger_bigrams(&self) -> Vec<([u8; 2], f32)>
    {
        return Vec::new();
    }
}
