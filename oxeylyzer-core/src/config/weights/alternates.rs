use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Alternates {
    pub base: f64,
    pub same_finger_skip: f64,
}

impl Default for Alternates {
    fn default() -> Self {
        return Self {
            base: 0.7,
            same_finger_skip: 0.35,
        };
    }
}
