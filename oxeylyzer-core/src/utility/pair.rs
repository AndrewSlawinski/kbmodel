use itertools::Itertools;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct Pair(pub usize, pub usize);

impl Pair
{
    pub const fn default() -> Self
    {
        return Self(0, 0);
    }

    pub const fn new(x1: usize, x2: usize) -> Self
    {
        return Self(x1, x2);
    }

    pub fn distance(&self, rhs: &Self) -> Self
    {
        return Self(self.0 - rhs.0, self.1 - rhs.1);
    }

    pub fn squared(&self) -> Self
    {
        return Self(self.0.pow(2), self.1.pow(2));
    }

    pub fn is_sfb(&self) -> bool
    {
        if self.0 == self.1
        {
            return false;
        }

        let p0 = self.0 % 10;
        let p1 = self.1 % 10;

        return p0.abs_diff(p1) == 0
            || (p0 == 3 && p1 == 4)
            || (p0 == 4 && p1 == 3)
            || (p0 == 5 && p1 == 6)
            || (p0 == 6 && p1 == 5);
    }

    pub fn all_key_indices() -> Vec<Self>
    {
        let columns = 0 .. 10;
        let rows = 0 .. 3;

        return columns
            .cartesian_product(rows)
            .map(|x| Pair(x.0, x.1))
            .collect_vec();
    }
}

impl std::fmt::Display for Pair
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        return write!(f, "({}, {})", self.0, self.1);
    }
}
