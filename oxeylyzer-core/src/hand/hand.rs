use Hand::*;

use crate::hand::finger::Finger;

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Hand {
    Left,
    Right,
}

impl std::ops::Not for Hand {
    type Output = Self;

    fn not(self) -> Self::Output {
        return match self {
            | Left => Right,
            | Right => Left,
        };
    }
}

impl From<Finger> for Hand {
    fn from(value: Finger) -> Self {
        return value.hand();
    }
}
