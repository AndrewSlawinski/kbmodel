use crate::layout::keyboard_type::KeyboardType;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Info {
    pub language: String,
    pub keyboard_type: KeyboardType,
    pub trigram_precision: u32,
}

impl Default for Info {
    fn default() -> Self {
        return Self {
            language: "english".to_string(),
            keyboard_type: KeyboardType::default(),
            trigram_precision: 100000,
        };
    }
}
