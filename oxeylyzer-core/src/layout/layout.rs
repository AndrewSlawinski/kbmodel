use crate::{
    config::pins::Pins,
    generic::{
        CharToFinger,
        Fixed,
    },
    hand::finger::Finger,
    language::trigram_patterns::TrigramPattern::Other,
    language::trigram_patterns::*,
    stat::trigram::Trigram,
    utility::converter::Converter,
    utility::pair::Pair,
};
use nanorand::{
    tls_rng,
    Rng,
};

pub const FINGER_TO_COLUMN: Fixed<usize> = [
    0, 1, 2, 3, 3, 4, 4, 5, 6, 7, 0, 1, 2, 3, 3, 4, 4, 5, 6, 7, 0, 1, 2, 3, 3, 4, 4, 5, 6, 7,
];

#[derive(Debug, Clone, PartialEq)]
pub struct FastLayout {
    pub name: String,
    pub matrix: Fixed<u8>,
    pub char_to_finger: CharToFinger,
}

impl FastLayout {
    pub fn layout_str(&self, converter: &Converter) -> String {
        return converter.as_string(&self.matrix);
    }

    pub fn formatted_string(&self, converter: &Converter) -> String {
        let mut res = String::new();

        for (i, u) in self.matrix.iter().enumerate() {
            let c = converter.u8_to_char(*u);

            if i % 10 == 0 && i > 0
            {
                res.push('\n');
            }

            if (i + 5) % 10 == 0
            {
                res.push(' ');
            }

            res.push(c);
            res.push(' ');
        }

        return res;
    }
}

impl From<Fixed<u8>> for FastLayout {
    fn from(layout: Fixed<u8>) -> Self {
        let mut new_layout = FastLayout::new();

        for (i, byte) in layout.into_iter().enumerate() {
            new_layout.matrix[i] = byte;
            new_layout.char_to_finger[byte as usize] = FINGER_TO_COLUMN[i];
        }

        return new_layout;
    }
}

impl From<&[u8]> for FastLayout {
    fn from(layout_bytes: &[u8]) -> Self {
        return if layout_bytes.len() >= 30
        {
            let mut new_layout = FastLayout::new();

            for (i, &byte) in layout_bytes.iter().enumerate() {
                new_layout.matrix[i] = byte;
                new_layout.char_to_finger[byte as usize] = FINGER_TO_COLUMN[i];
            }

            new_layout
        } else {
            panic!("you should provide at least 30 bytes to create a layout from.")
        };
    }
}

pub trait Layout<T: Copy + Default> {
    fn new() -> Self;

    fn random(available_chars: Fixed<T>) -> Self;

    fn random_pins(layout_chars: Fixed<T>, pins: &Pins) -> Self;

    fn c(&self, i: usize) -> T;

    fn swap_indexes(&mut self);

    fn get_trigram_pattern(&self, trigram: &Trigram<u8>) -> TrigramPattern;
}

pub trait LayoutInternal<T: Copy + Default> {
    unsafe fn cu(&self, i: usize) -> T;

    unsafe fn swap_xy_no_bounds(&mut self, i0: usize, i1: usize);

    unsafe fn swap_no_bounds(&mut self, pair: &Pair);

    unsafe fn swap_cols_no_bounds(&mut self, col0: usize, col1: usize);
}

impl LayoutInternal<u8> for FastLayout {
    #[inline(always)]
    unsafe fn cu(&self, i: usize) -> u8 {
        return *self.matrix.get_unchecked(i);
    }

    #[inline(always)]
    unsafe fn swap_xy_no_bounds(&mut self, i0: usize, i1: usize) {
        let char0 = self.cu(i0);
        let char1 = self.cu(i1);

        *self.matrix.get_unchecked_mut(i0) = char1;
        *self.matrix.get_unchecked_mut(i1) = char0;

        *self.char_to_finger.get_unchecked_mut(char0 as usize) = *FINGER_TO_COLUMN.get_unchecked(i1);
        *self.char_to_finger.get_unchecked_mut(char1 as usize) = *FINGER_TO_COLUMN.get_unchecked(i0);
    }

    #[inline(always)]
    unsafe fn swap_no_bounds(&mut self, pair: &Pair) {
        self.swap_xy_no_bounds(pair.0, pair.1);
    }

    unsafe fn swap_cols_no_bounds(&mut self, col0: usize, col1: usize) {
        self.swap_xy_no_bounds(col0, col1);
        self.swap_xy_no_bounds(col0 + 10, col1 + 10);
        self.swap_xy_no_bounds(col0 + 20, col1 + 20);
    }
}

impl Layout<u8> for FastLayout {
    fn new() -> Self {
        return Self {
            name: "".to_string(),
            matrix: [u8::MAX; 30],
            char_to_finger: [usize::MAX; 60],
        };
    }

    fn random(mut with_chars: Fixed<u8>) -> Self {
        shuffle_pins(&mut with_chars, None);

        return Self::from(with_chars);
    }

    fn random_pins(mut layout_chars: Fixed<u8>, pins: &Pins) -> Self {
        shuffle_pins(&mut layout_chars, Some(pins));

        return Self::from(layout_chars);
    }

    #[inline(always)]
    fn c(&self, i: usize) -> u8 {
        return self.matrix[i];
    }

    fn swap_indexes(&mut self) {
        unsafe {
            self.swap_cols_no_bounds(3, 6);
            self.swap_cols_no_bounds(4, 5);
        }
    }

    #[inline(always)]
    fn get_trigram_pattern(&self, trigram: &Trigram<u8>) -> TrigramPattern {
        let a = match self.char_to_finger.get(trigram.0 as usize) {
            | Some(&v) if v != usize::MAX => v,
            | _ => return TrigramPattern::Invalid,
        };

        let b = match self.char_to_finger.get(trigram.1 as usize) {
            | Some(&v) if v != usize::MAX => v,
            | _ => return TrigramPattern::Invalid,
        };

        let c = match self.char_to_finger.get(trigram.2 as usize) {
            | Some(&v) if v != usize::MAX => v,
            | _ => return TrigramPattern::Invalid,
        };

        // a, b and c are numbers between 0 and 7. This means they fit in exactly 3 bits (7 == 0b111)
        let combination = (a << 6) | (b << 3) | c;

        return get_trigram_combinations()[combination];
    }
}

pub fn get_trigram_combinations() -> [TrigramPattern; 512] {
    let mut c3 = 0;
    let mut combinations: [TrigramPattern; 512] = [Other; 512];

    while c3 < 8 {
        let mut c2 = 0;

        while c2 < 8 {
            let mut c1 = 0;

            while c1 < 8 {
                let index = c3 * 64 + c2 * 8 + c1;

                let trigram = Trigram::new(Finger::from(c3), Finger::from(c2), Finger::from(c1));

                combinations[index as usize] = trigram.get_trigram_pattern();

                c1 += 1;
            }

            c2 += 1;
        }

        c3 += 1;
    }

    return combinations;
}

#[inline]
fn shuffle_pins(slice: &mut [u8], pins: Option<&Pins>) {
    let range = 0_u8..slice.len() as u8;

    let mapped = if pins.is_some()
    {
        range.filter(|x| !Some(pins).unwrap().unwrap().pins.contains(x)).collect::<Vec<u8>>()
    } else {
        range.collect()
    };

    let mut rng = tls_rng();

    for (map, &swap1) in mapped.iter().enumerate() {
        let swap2 = rng.generate_range(map..mapped.len());

        slice.swap(swap1 as usize, mapped[swap2] as usize);
    }
}
