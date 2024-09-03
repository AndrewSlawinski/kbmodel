use crate::type_def::Fixed;
use std::collections::HashMap;
use std::ops::Index;

pub type CharToFinger = HashMap<u8, usize>;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Layout
{
    pub matrix: Fixed<char>,
}

impl Index<usize> for Layout
{
    type Output = char;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output
    {
        return &self.matrix[index];
    }
}

impl From<Fixed<char>> for Layout
{
    fn from(layout: Fixed<char>) -> Self
    {
        // const FINGER_TO_COLUMN: Fixed<usize> = [
        //     0, 1, 2, 3, 3, 4, 4, 5, 6, 7, 0, 1, 2, 3, 3, 4, 4, 5, 6, 7, 0, 1, 2, 3, 3, 4, 4, 5, 6,
        //     7,
        // ];

        return Self { matrix: layout };
    }
}
