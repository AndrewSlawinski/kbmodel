use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Redirects {
    pub base: f64,
    pub same_finger_skips: f64,
    pub bad: f64,
    pub bad_same_finger_skips: f64,
}

impl Default for Redirects {
    fn default() -> Self {
        return Self {
            base: 1.5,
            same_finger_skips: 2.75,
            bad: 4.0,
            bad_same_finger_skips: 6.0,
        };
    }
}
