use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Bigrams {
    pub scissors: f64,
    pub pinky_ring: f64,
    pub lateral_stretch: f64,

    pub same_finger_skip: f64,
}

impl From<Vec<f64>> for Bigrams {
    fn from(value: Vec<f64>) -> Self {
        return Self {
            scissors: value[0],
            pinky_ring: value[1],
            lateral_stretch: value[2],
            same_finger_skip: value[3],
        };
    }
}

impl Default for Bigrams {
    fn default() -> Self {
        return Self {
            scissors: 5.0,
            pinky_ring: 0.0,
            lateral_stretch: 2.0,
            same_finger_skip: 0.12,
        };
    }
}
