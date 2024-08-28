use crate::hand::finger::Finger;
use crate::layout::layout::Layout;
use crate::ngram::ngram::NGram;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum BigramType
{
    LateralStretchBigrams,
    SFB,
    Scissors,
    Repeat,
    Skip1,
    Skip2,
    Skip3,
    Other,
    Invalid,
}

impl BigramType
{
    pub fn get_all_combinations() -> [Self; 64]
    {
        let mut combinations = [Self::Other; 64];

        for c2 in 0 .. 8
        {
            for c1 in 0 .. 8
            {
                let bigram = NGram::new([Finger::from(c2), Finger::from(c1)]);

                let index = c2 * 8 + c1;
                combinations[index as usize] = bigram.get_bigram_pattern();
            }
        }

        return combinations;
    }

    #[inline(always)]
    pub fn get_pattern(layout: &Layout, bigram: &[u8; 2], all_bigrams: &[Self; 64]) -> Self
    {
        let a = match layout.char_to_finger.get(&bigram[0])
        {
            | Some(&v) if v != usize::MAX => v,
            | _ => return Self::Invalid,
        };

        let b = match layout.char_to_finger.get(&bigram[1])
        {
            | Some(&v) if v != usize::MAX => v,
            | _ => return Self::Invalid,
        };

        let combination = (a << 3) | b;

        return all_bigrams[combination];
    }
}
