use crate::generic::Fixed;

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
        return unsafe {
            *AFFECTS_SCISSOR.get_unchecked(self.0) || *AFFECTS_SCISSOR.get_unchecked(self.1)
        };
    }

    #[inline]
    pub fn affects_lsb(&self) -> bool {
        return unsafe { *AFFECTS_LSB.get_unchecked(self.0) || *AFFECTS_LSB.get_unchecked(self.1) };
    }

    #[inline]
    pub fn affects_pinky_ring(&self) -> bool {
        return unsafe {
            *AFFECTS_PINKY_RING.get_unchecked(self.0) || *AFFECTS_PINKY_RING.get_unchecked(self.1)
        };
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
