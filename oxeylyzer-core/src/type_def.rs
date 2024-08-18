use crate::utility::pair::Pair;
use std::collections::HashMap;

pub type FingerSpeeds = [(Pair, f64); 48];
pub type CharToFinger = HashMap<char, usize>;
pub type Fixed<T> = [T; 30];
pub const COLS: [usize; 6] = [0, 1, 2, 7, 8, 9];
pub const ROWS: usize = 3;
pub const COLUMNS: usize = 10;
pub const COLUMN_TO_START_LENGTH: [(usize, usize); 8] = [
    (0, 3),
    (3, 3),
    (6, 3),
    (18, 15),
    (33, 15),
    (9, 3),
    (12, 3),
    (15, 3),
];

pub const FINGER_TO_COLUMN: Fixed<usize> = [
    0, 1, 2, 3, 3, 4, 4, 5, 6, 7, 0, 1, 2, 3, 3, 4, 4, 5, 6, 7, 0, 1, 2, 3, 3, 4, 4, 5, 6, 7,
];

pub const SFB_INDICES: [Pair; 48] = sfb_indices();

const fn sfb_indices() -> [Pair; 48] {
    let mut res = [Pair::default(); 48];
    let mut i = 0;

    let mut col_i = 0;
    let cols = [0, 1, 2, 7, 8, 9];

    while col_i < cols.len() {
        let col = cols[col_i];

        res[i] = Pair(col, col + 10);
        res[i + 1] = Pair(col, col + 20);
        res[i + 2] = Pair(col + 10, col + 20);

        col_i += 1;
        i += 3;
    }

    let mut c = 0;

    while c <= 2 {
        let index = [
            (3 + c, 13 + c),
            (3 + c, 23 + c),
            (3 + c, 4 + c),
            (3 + c, 14 + c),
            (3 + c, 24 + c),
            (13 + c, 23 + c),
            (13 + c, 4 + c),
            (13 + c, 14 + c),
            (13 + c, 24 + c),
            (23 + c, 4 + c),
            (23 + c, 14 + c),
            (23 + c, 24 + c),
            (4 + c, 14 + c),
            (4 + c, 24 + c),
            (14 + c, 24 + c),
        ];

        let mut pair_i = 0;

        while pair_i < 15 {
            res[i] = Pair(index[pair_i].0, index[pair_i].1);

            i += 1;
            pair_i += 1;
        }

        c += 2;
    }

    return res;
}
