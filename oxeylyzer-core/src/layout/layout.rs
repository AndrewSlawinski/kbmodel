use crate::type_def::Fixed;
use std::collections::HashMap;
use std::ops::Index;

pub type CharToFinger = HashMap<u8, usize>;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Layout
{
    pub matrix: Fixed<u8>,
    pub char_to_finger: CharToFinger,
}

impl Index<usize> for Layout
{
    type Output = u8;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output
    {
        return &self.matrix[index];
    }
}

impl From<Fixed<u8>> for Layout
{
    fn from(layout: Fixed<u8>) -> Self
    {
        const FINGER_TO_COLUMN: Fixed<usize> = [
            0, 1, 2, 3, 3, 4, 4, 5, 6, 7, 0, 1, 2, 3, 3, 4, 4, 5, 6, 7, 0, 1, 2, 3, 3, 4, 4, 5, 6,
            7,
        ];

        let mut new_layout = Layout::default();

        for (i, c) in layout.iter().enumerate()
        {
            new_layout.matrix[i] = c.clone();

            new_layout
                .char_to_finger
                .entry(*c)
                .or_insert(FINGER_TO_COLUMN[i]);
        }

        return new_layout;
    }
}
