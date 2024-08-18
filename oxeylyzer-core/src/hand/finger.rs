use crate::{
    hand::finger::Finger::{
        LI,
        LM,
        LP,
        LR,
        LT,
        RI,
        RM,
        RP,
        RR,
        RT,
    },
    hand::hand::Hand,
    hand::hand::Hand::*,
};
use num_derive::FromPrimitive;

#[repr(u8)]
#[derive(FromPrimitive, Copy, Clone, Debug, Eq, PartialEq)]
pub enum Finger {
    LP = 0,
    LR = 1,
    LM = 2,
    LI = 3,
    LT = 4,
    RT = 5,
    RI = 6,
    RM = 7,
    RR = 8,
    RP = 9,
}

impl From<u8> for Finger {
    fn from(value: u8) -> Self {
        return match value {
            | 0 => LP,
            | 1 => LR,
            | 2 => LM,
            | 3 => LI,
            | 4 => LT,
            | 5 => RT,
            | 6 => RI,
            | 7 => RM,
            | 8 => RR,
            | 9 => RP,
            | _ => unreachable!(),
        };
    }
}

impl Finger {
    pub const fn eq(self, other: Self) -> bool {
        return self as u8 == other as u8;
    }

    pub fn relative_eq(self, other: Self) -> bool {
        return if self.hand() != other.hand()
        {
            9 - (self as u8).min(other as u8) == (self as u8).max(other as u8)
        } else {
            self as u8 == other as u8
        };
    }

    pub const fn gt(self, other: Self) -> bool {
        return self as u8 > other as u8;
    }

    pub const fn lt(self, other: Self) -> bool {
        return (self as u8) < (other as u8);
    }

    pub fn relative_gt(self, other: Self) -> bool {
        return if self.hand() != other.hand()
        {
            9 - (self as u8).min(other as u8) < (self as u8).max(other as u8)
        } else {
            match self.hand() {
                | Left => (self as u8) > (other as u8),
                | Right => (self as u8) < (other as u8),
            }
        };
    }

    pub fn relative_lt(self, other: Self) -> bool {
        return if self.hand() != other.hand()
        {
            9 - (self as u8).min(other as u8) > (self as u8).max(other as u8)
        } else {
            match self.hand() {
                | Left => (self as u8) < (other as u8),
                | Right => (self as u8) > (other as u8),
            }
        };
    }

    pub const fn hand(&self) -> Hand {
        return match self {
            | LP | LR | LM | LI | LT => Left,
            | _ => Right,
        };
    }

    pub const fn is_bad(&self) -> bool {
        return matches!(self, LP | LR | RR | RP);
    }
}

impl std::fmt::Display for Finger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let to_write = match self {
            | LP => "left pinky",
            | LR => "left ring",
            | LM => "left middle",
            | LI => "left index",
            | RI => "right index",
            | RM => "right middle",
            | RR => "right ring",
            | RP => "right pinky",
            | LT => "left thumb",
            | RT => "right thumb",
        };

        return write!(f, "{}", to_write);
    }
}
