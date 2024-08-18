use crate::hand::finger::Finger;
use crate::layout::layout::FastLayout;
use crate::n_gram::n_gram::NGram;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum TrigramType {
    Alternate,
    Inroll,
    Outroll,
    Redirect,
    SameFingerTrigram,
    Other,
    Invalid,
}

impl TrigramType {
    pub fn get_all_combinations() -> [Self; 512] {
        let mut combinations = [Self::Other; 512];

        for c2 in 0..8 {
            for c1 in 0..8 {
                for c0 in 0..8 {
                    let trigram = NGram::new([Finger::from(c2), Finger::from(c1), Finger::from(c0)]);

                    let index = c2 * 64 + c1 * 8 + c0;
                    combinations[index as usize] = trigram.get_trigram_pattern();
                }
            }
        }

        return combinations;
    }

    #[inline(always)]
    pub fn get_pattern(
        layout: &FastLayout,
        trigram: &NGram<char, 3>,
        all_trigrams: &[Self; 512],
    ) -> Self {
        let a = match layout.char_to_finger.get(&trigram[0]) {
            | Some(&v) if v != usize::MAX => v,
            | _ => return Self::Invalid,
        };

        let b = match layout.char_to_finger.get(&trigram[1]) {
            | Some(&v) if v != usize::MAX => v,
            | _ => return Self::Invalid,
        };

        let c = match layout.char_to_finger.get(&trigram[2]) {
            | Some(&v) if v != usize::MAX => v,
            | _ => return Self::Invalid,
        };

        // a, b and c are numbers between 0 and 7. This means they fit in exactly 3 bits (7 == 0b111)
        let combination = (a << 6) | (b << 3) | c;

        return all_trigrams[combination];
    }
}
