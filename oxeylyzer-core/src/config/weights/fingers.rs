use crate::config::weights::bias::Bias;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Fingers {
    pub speed: f64,
    pub heatmap: f64,
    pub onehands: f64,

    pub lateral_penalty: f64,
    pub overuse_penalty: f64,

    pub bias: Bias,
}

impl Default for Fingers {
    fn default() -> Self {
        return Self {
            speed: 8.0,
            heatmap: 0.85,
            onehands: 0.8,
            lateral_penalty: 1.3,
            overuse_penalty: 2.5,

            bias: Default::default(),
        };
    }
}
