use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Rolls {
    pub inward: f64,
    pub outward: f64,
}

impl Default for Rolls {
    fn default() -> Self {
        return Self {
            inward: 1.6,
            outward: 1.3,
        };
    }
}
