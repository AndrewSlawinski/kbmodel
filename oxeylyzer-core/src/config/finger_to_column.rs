use crate::hand::finger::Finger;
use serde::Deserialize;
use std::ops;

#[derive(Deserialize, Clone)]
#[serde(from = "String")]
pub struct FingerToColumn {
    pub finger_to_column: Vec<Finger>,
}

impl Default for FingerToColumn {
    fn default() -> Self {
        return Self {
            finger_to_column: Vec::new(),
        };
    }
}

impl ops::Index<usize> for FingerToColumn {
    type Output = Finger;

    fn index(&self, index: usize) -> &Self::Output {
        return &self.finger_to_column[index];
    }
}

impl From<String> for FingerToColumn {
    fn from(value: String) -> Self {
        let mut finger_to_column = Vec::new();

        let value = value.trim().replace([' ', '\n'], "");

        for c in value.chars().into_iter() {
            finger_to_column.push(Finger::from(c.to_digit(10).unwrap() as u8));
        }

        return Self { finger_to_column };
    }
}
