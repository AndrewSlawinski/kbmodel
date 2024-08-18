use crate::type_def::{
    CharToFinger,
    Fixed,
    FINGER_TO_COLUMN,
};
use crate::{
    config::pins::Pins,
    utility::pair::Pair,
};
use itertools::Itertools;
use nanorand::{
    tls_rng,
    Rng,
};
use std::ops::Index;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FastLayout {
    pub name: String,
    pub matrix: Fixed<char>,
    pub char_to_finger: CharToFinger,
}

impl Index<usize> for FastLayout {
    type Output = char;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        return &self.matrix[index];
    }
}

impl FastLayout {
    pub fn based_on_layout(layout: &FastLayout, pins: Option<&Pins>) -> Self {
        let mut chars = layout.matrix.clone();
        Self::shuffle_pins(&mut chars, pins);

        return Self::from(&chars);
    }

    #[inline(always)]
    pub fn swap_xy(&mut self, pair: &Pair) {
        let char0 = self.matrix[pair.0];
        let char1 = self.matrix[pair.1];

        *self.matrix.get_mut(pair.0).unwrap() = char1;
        *self.matrix.get_mut(pair.1).unwrap() = char0;

        *self.char_to_finger.get_mut(&char0).unwrap() = FINGER_TO_COLUMN[pair.1];
        *self.char_to_finger.get_mut(&char1).unwrap() = FINGER_TO_COLUMN[pair.0];
    }

    fn swap_cols_no_bounds(&mut self, pair: &Pair) {
        self.swap_xy(pair);
        self.swap_xy(&Pair(pair.0 + 10, pair.1 + 10));
        self.swap_xy(&Pair(pair.0 + 20, pair.1 + 20));
    }

    pub fn swap_indexes(&mut self) {
        self.swap_cols_no_bounds(&Pair(3, 6));
        self.swap_cols_no_bounds(&Pair(4, 5));
    }

    pub fn layout_str(&self) -> String {
        return self.matrix.iter().join(" ");
        // return converter.as_string(&self.matrix);
    }

    pub fn formatted_string(&self) -> String {
        let mut res = String::new();

        for (i, u) in self.matrix.iter().enumerate() {
            if i % 10 == 0 && i > 0
            {
                res.push('\n');
            }

            if (i + 5) % 10 == 0
            {
                res.push(' ');
            }

            res.push(*u);
            res.push(' ');
        }

        return res;
    }

    #[inline]
    fn shuffle_pins(slice: &mut [char], pins: Option<&Pins>) {
        let range = 0..slice.len() as u8;

        let mapped = if pins.is_some()
        {
            let pins = pins.unwrap();

            range.filter(|x| !pins.contains(x)).collect_vec()
        } else {
            range.collect()
        };

        let mut rng = tls_rng();

        for (map, &swap1) in mapped.iter().enumerate() {
            let swap2 = rng.generate_range(map..mapped.len());

            slice.swap(swap1 as usize, mapped[swap2] as usize);
        }
    }
}

impl From<&Fixed<char>> for FastLayout {
    fn from(layout: &Fixed<char>) -> Self {
        let mut new_layout = FastLayout::default();

        for (i, c) in layout.iter().enumerate() {
            new_layout.matrix[i] = c.clone();

            new_layout.char_to_finger.entry(*c).or_insert(FINGER_TO_COLUMN[i]);
        }

        return new_layout;
    }
}

impl From<&[char]> for FastLayout {
    fn from(layout_bytes: &[char]) -> Self {
        if layout_bytes.len() >= 30
        {
            let mut new_layout = FastLayout::default();

            for (i, &c) in layout_bytes.iter().enumerate() {
                new_layout.matrix[i] = c;

                new_layout.char_to_finger.entry(c).or_insert(FINGER_TO_COLUMN[i]);
            }

            return new_layout;
        };

        panic!("you should provide at least 30 bytes to create a layout from.")
    }
}
