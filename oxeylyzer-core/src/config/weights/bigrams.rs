use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Bigrams {
    pub pinky_ring: f64,
    pub lateral_stretch: f64,
    pub d_same_finger_ratio: f64,
    #[serde(skip)]
    pub d_same_finger_ratio1: f64,
    #[serde(skip)]
    pub d_same_finger_ratio2: f64,
}

impl From<Vec<f64>> for Bigrams {
    fn from(value: Vec<f64>) -> Self {
        return Self {
            pinky_ring: value[0],
            lateral_stretch: value[1],
            d_same_finger_ratio: value[2],
            d_same_finger_ratio1: value[2].powi(2),
            d_same_finger_ratio2: value[2].powi(3),
        };
    }
}

impl Default for Bigrams {
    fn default() -> Self {
        return Self {
            pinky_ring: 0.0,
            lateral_stretch: 2.0,
            d_same_finger_ratio: 0.12,
            d_same_finger_ratio1: (0.10 * 6_f64).powi(2),
            d_same_finger_ratio2: (0.08 * 6_f64).powi(3),
        };
    }
}
