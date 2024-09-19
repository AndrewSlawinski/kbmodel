use crate::language_data::LanguageData;
use futures::join;
use std::collections::HashMap;

pub struct StatMatrices
{
    pub char_map: [f32; 30],
    pub bigram_map: [f32; 900],
    pub disjoint_map: [f32; 450],

    pub skip1_map: [f32; 900],
    pub skip2_map: [f32; 900],
    pub skip3_map: [f32; 900],

    pub trigram_map: [f32; 27000],
}

#[allow(dead_code)]
impl StatMatrices
{
    pub fn new() -> Self
    {
        return Self {
            char_map: [0.; 30],
            bigram_map: [0.; 900],
            disjoint_map: [0.; 450],
            skip1_map: [0.; 900],
            skip2_map: [0.; 900],
            skip3_map: [0.; 900],
            trigram_map: [0.; 27000],
        };
    }

    pub async fn compute(&mut self, chars: &[char; 30], language_data: &LanguageData)
    {
        join!(
            Self::char_map(&mut self.char_map, chars, &language_data.characters),
            Self::bigram_map(&mut self.bigram_map, chars, &language_data.bigrams),
            Self::disjoint_map(&mut self.disjoint_map, chars, &language_data.trigrams),
            Self::bigram_map(&mut self.skip1_map, chars, &language_data.skipgrams),
            Self::bigram_map(&mut self.skip2_map, chars, &language_data.skipgrams2),
            Self::bigram_map(&mut self.skip3_map, chars, &language_data.skipgrams3),
            Self::trigram_map(&mut self.trigram_map, chars, &language_data.trigrams),
        );
    }

    async fn char_map(a: &mut [f32; 30], chars: &[char; 30], hash_map: &HashMap<String, f32>)
    {
        for i in 0 .. a.len()
        {
            if chars[i].is_ascii_punctuation()
            {
                continue;
            }

            a[i] = *hash_map.get(&format!("{}", chars[i])).unwrap_or(&0.0) * 100.0;
        }
    }

    async fn bigram_map(a: &mut [f32; 900], chars: &[char; 30], hash_map: &HashMap<String, f32>)
    {
        for (i, c0) in chars.iter().enumerate()
        {
            if c0.is_ascii_punctuation()
            {
                continue;
            }

            for (j, c1) in chars.iter().enumerate()
            {
                if c1.is_ascii_punctuation()
                {
                    continue;
                }

                let k = j + i * 30;
                let s = format!("{c0}{c1}");

                // println!("{}", s);

                a[k] = *hash_map.get(&s).unwrap_or(&0.0) * 100.0;
            }
        }
    }

    async fn disjoint_map(a: &mut [f32; 450], chars: &[char; 30], hash_map: &HashMap<String, f32>)
    {
        let mut l = 0;
        for (i, c0) in chars.iter().enumerate()
        {
            if c0.is_ascii_punctuation()
            {
                l += 1;

                continue;
            }

            let i_left = i % 10 < 5;
            for (j, c1) in chars.iter().enumerate()
            {
                let j_left = j % 10 < 5;

                if i_left == j_left
                {
                    continue;
                }

                if c1.is_ascii_punctuation()
                {
                    continue;
                }

                let mut m = 0;
                for (k, c2) in chars.iter().enumerate()
                {
                    if i_left != (k % 10 < 5)
                    {
                        continue;
                    }

                    if c2.is_ascii_punctuation()
                    {
                        m += 1;

                        continue;
                    }

                    let n = m + l * 15;
                    let s = format!("{c0}{c1}{c2}");

                    a[n] += *hash_map.get(&s).unwrap_or(&0.0) * 100.0;

                    m += 1;
                }
            }

            l += 1;
        }
    }

    async fn trigram_map(a: &mut [f32; 27000], chars: &[char; 30], hash_map: &HashMap<String, f32>)
    {
        for (i, c0) in chars.iter().enumerate()
        {
            if c0.is_ascii_punctuation()
            {
                continue;
            }

            for (j, c1) in chars.iter().enumerate()
            {
                if c1.is_ascii_punctuation()
                {
                    continue;
                }

                for (k, c2) in chars.iter().enumerate()
                {
                    if c2.is_ascii_punctuation()
                    {
                        continue;
                    }

                    let l = k + j * 30 + i * 300;
                    let s = format!("{c0}{c1}{c2}");

                    // println!("{}", s);

                    a[l] = *hash_map.get(&s).unwrap_or(&0.0) * 100.0;
                }
            }
        }
    }
}
