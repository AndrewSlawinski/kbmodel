use crate::hand::finger::Finger;
use std::cmp::PartialEq;
use std::ops::Index;

pub trait NGramConstraint {}

impl NGramConstraint for u8 {}
impl NGramConstraint for char {}
impl NGramConstraint for Finger {}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct NGram<T: NGramConstraint, const N: usize>
{
    pub inner: [T; N],
}

#[allow(unused)]
impl<T: NGramConstraint, const N: usize> Index<usize> for NGram<T, N>
{
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output
    {
        return &self.inner[index];
    }
}

#[allow(unused)]
impl<const N: usize> NGram<Finger, N>
{
    pub const fn new(f: [Finger; N]) -> Self
    {
        return Self { inner: f };
    }

    fn is_alternate(&self) -> bool
    {
        for i in 0 .. N
        {
            if i == 0
            {
                continue;
            }

            if self.inner[i].hand() == self.inner[i - 1].hand()
            {
                return false;
            }
        }

        return N > 1;
    }

    const fn is_same_finger_skipgram(&self) -> bool
    {
        if N < 2
        {
            return false;
        }

        return self.inner[0].eq(self.inner[2]);
    }

    fn is_roll(&self) -> bool
    {
        if N < 2
        {
            return false;
        }

        if N == 2 && self.inner[0] == self.inner[1]
        {
            return false;
        }

        for i in 1 .. self.inner.len()
        {
            if self.inner[i - 1].hand() == self.inner[i].hand()
            {
                return true;
            }
        }

        return false;
    }

    fn is_inroll(&self) -> bool
    {
        if N < 2
        {
            return false;
        }

        if N == 2
        {
            return self.inner[0].relative_lt(self.inner[1]);
        }

        for i in 1 .. self.inner.len()
        {
            if self.inner[i - 1].relative_lt(self.inner[i])
            {
                return true;
            }
        }

        return false;
    }

    fn is_outroll(&self) -> bool
    {
        if N < 2
        {
            return false;
        }

        if N == 2
        {
            return self.inner[0].relative_gt(self.inner[1]);
        }

        for i in 1 .. self.inner.len()
        {
            if self.inner[i - 1].relative_gt(self.inner[i])
            {
                return true;
            }
        }

        return false;
    }

    fn is_one_hand(&self) -> bool
    {
        return self.inner.iter().all(|x| x.hand() == self.inner[0].hand());
    }

    fn is_redirect(&self) -> bool
    {
        if N < 3
        {
            return false;
        }

        return self.is_one_hand()
            && (self.inner[0].lt(self.inner[1]) == self.inner[1].gt(self.inner[2]));
    }

    fn is_bad_redirect(&self) -> bool
    {
        return self.is_redirect() && self.inner.iter().any(|x| x.is_bad());
    }

    fn is_same_finger_ngram(&self, n: u8) -> bool
    {
        if n < 2 || n > self.inner.len() as u8
        {
            return false;
        }

        for i in (n - 1) as usize .. self.inner.len()
        {
            if self.inner[(i - 1) .. i]
                .iter()
                .all(|x| *x == self.inner[i - 1])
            {
                return true;
            }
        }

        return false;
    }
}

#[allow(unused)]
impl<const N: usize> From<&[char; N]> for NGram<char, N>
{
    fn from(value: &[char; N]) -> Self
    {
        return Self {
            inner: value.clone(),
        };
    }
}

#[allow(unused)]
impl<const N: usize> From<&[u8; N]> for NGram<u8, N>
{
    fn from(value: &[u8; N]) -> Self
    {
        return Self {
            inner: value.clone(),
        };
    }
}

#[allow(unused)]
impl<const N: usize> From<&[&NGram<char, 1>; N]> for NGram<char, N>
{
    fn from(value: &[&NGram<char, 1>; N]) -> Self
    {
        let mut v = [' '; N];

        value
            .iter()
            .enumerate()
            .for_each(|(i, t)| v[i] = t.inner.to_vec()[0]);

        return Self::from(&v);
    }
}

#[allow(unused)]
impl<const N: usize> std::fmt::Display for NGram<Finger, N>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let mut form = "".to_string();

        for i in 0 .. self.inner.len()
        {
            form += format!("{} ", self.inner[i]).as_str();
        }

        return write!(f, "{}", form);
    }
}
