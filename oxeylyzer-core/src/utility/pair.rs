use crate::type_def::{
    Fixed,
    COLUMNS,
    ROWS,
};
use itertools::Itertools;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct Pair(pub usize, pub usize);

impl Pair {
    pub const fn default() -> Self {
        return Self(0, 0);
    }

    pub const fn new(x1: usize, x2: usize) -> Self {
        return Self(x1, x2);
    }

    #[inline]
    pub fn affects_scissor(&self) -> bool {
        return *AFFECTS_SCISSOR.get(self.0).unwrap() || *AFFECTS_SCISSOR.get(self.1).unwrap();
    }

    #[inline]
    pub fn affects_lsb(&self) -> bool {
        return *AFFECTS_LSB.get(self.0).unwrap() || *AFFECTS_LSB.get(self.1).unwrap();
    }

    #[inline]
    pub fn affects_pinky_ring(&self) -> bool {
        return *AFFECTS_PINKY_RING.get(self.0).unwrap() || *AFFECTS_PINKY_RING.get(self.1).unwrap();
    }

    pub fn distance(&self, rhs: &Self) -> Self {
        return Self(self.0 - rhs.0, self.1 - rhs.1);
    }

    pub fn squared(&self) -> Self {
        return Self(self.0.pow(2), self.1.pow(2));
    }

    pub fn all_key_indices() -> Vec<Self> {
        let columns = 0..COLUMNS;
        let rows = 0..ROWS;

        return columns.cartesian_product(rows).map(|x| Pair(x.0, x.1)).collect_vec();
    }
}

impl std::fmt::Display for Pair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "({}, {})", self.0, self.1);
    }
}

const AFFECTS_SCISSOR: Fixed<bool> = [
    true, true, true, true, true, true, true, true, true, true, true, true, false, false, false,
    false, false, false, true, true, true, true, true, false, true, false, false, true, true, true,
];

const AFFECTS_LSB: Fixed<bool> = [
    false, false, true, false, true, true, false, true, false, false, false, false, true, false,
    true, true, false, true, false, false, false, false, true, false, true, true, false, true,
    false, false,
];

const AFFECTS_PINKY_RING: Fixed<bool> = [
    true, true, false, false, false, false, false, false, true, true, true, true, false, false,
    false, false, false, false, true, true, true, true, false, false, false, false, false, false,
    true, true,
];
