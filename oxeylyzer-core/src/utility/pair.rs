#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct Pair(pub usize, pub usize);

impl Pair
{
    pub const fn default() -> Self
    {
        return Self(0, 0);
    }

    pub const fn new(x0: usize, x1: usize) -> Self
    {
        return Self(x0, x1);
    }

    pub fn distance(&self, rhs: &Self) -> Self
    {
        return Self(self.0 - rhs.0, self.1 - rhs.1);
    }

    pub fn squared(&self) -> Self
    {
        return Self(self.0.pow(2), self.1.pow(2));
    }
}

impl Pair {}

impl std::fmt::Display for Pair
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        return write!(f, "({}, {})", self.0, self.1);
    }
}
