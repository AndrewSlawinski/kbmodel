use serde::Deserialize;
use std::ops;

#[derive(Deserialize, Clone)]
#[serde(from = "String")]
pub struct Pins {
    pub pins: Vec<u8>,
}

impl Pins {
    fn len(&self) -> usize {
        return self.pins.len();
    }
}

impl Default for Pins {
    fn default() -> Self {
        return Self { pins: Vec::new() };
    }
}

impl ops::Index<usize> for Pins {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        return &self.pins[index];
    }
}

impl From<String> for Pins {
    fn from(value: String) -> Self {
        let mut pins = Vec::new();

        let value = value.trim().replace([' ', '\n'], "");

        for (i, c) in value.chars().enumerate() {
            if c == 'x'
            {
                pins.push(i as u8);
            }
        }

        return Self { pins };
    }
}
