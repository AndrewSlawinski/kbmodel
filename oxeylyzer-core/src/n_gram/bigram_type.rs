use crate::hand::finger::Finger;
use crate::layout::layout::FastLayout;
use crate::n_gram::n_gram::NGram;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum BigramType {
    Scissors,
    SameFingerBigram,
    SameFingerSkipGram,
    SameFingerSkip2Gram,
    SameFingerSkip3Gram,
    LateralStretchBigrams,
    Other,
    Invalid,
}

impl BigramType {
    pub fn get_all_combinations() -> [Self; 64] {
        let mut combinations = [Self::Other; 64];

        for c2 in 0..8 {
            for c1 in 0..8 {
                let bigram = NGram::new([Finger::from(c2), Finger::from(c1)]);

                let index = c2 * 8 + c1;
                combinations[index as usize] = bigram.get_bigram_pattern();
            }
        }

        return combinations;
    }

    #[inline(always)]
    pub fn get_pattern(
        layout: &FastLayout,
        bigram: &NGram<char, 2>,
        all_bigrams: &[Self; 64],
    ) -> Self {
        let a = match layout.char_to_finger.get(&bigram[0]) {
            | Some(&v) if v != usize::MAX => v,
            | _ => return Self::Invalid,
        };

        let b = match layout.char_to_finger.get(&bigram[1]) {
            | Some(&v) if v != usize::MAX => v,
            | _ => return Self::Invalid,
        };

        let combination = (a << 3) | b;

        return all_bigrams[combination];
    }
}
