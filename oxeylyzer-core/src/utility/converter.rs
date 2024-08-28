use crate::language::language_data::{
    LanguageData,
    LanguageDataIntermediate,
};
use crate::layout::layout::Layout;
use crate::type_def::Fixed;
use itertools::Itertools;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct Converter
{
    domain: Vec<char>,
    codomain: HashMap<char, u8>,
}

impl Converter
{
    pub fn parse_layout(&mut self, string: &str) -> Layout
    {
        let string = string
            .to_string()
            .chars()
            .filter(|x| !x.is_whitespace())
            .collect_vec();

        assert_eq!(string.len(), 30);

        let layout_bytes: Fixed<u8> = self.to(string).try_into().unwrap();

        return Layout::from(layout_bytes);
    }

    pub fn language_data(&mut self, language_data_intermediate: &str) -> LanguageData
    {
        let mut language_data: LanguageDataIntermediate =
            serde_json::from_str(language_data_intermediate).unwrap();

        for c in ['\'', ',', '.', ';', '/', '~']
        {
            language_data.characters.entry(c).or_insert(0.0);
        }

        return LanguageData {
            language: language_data.language.clone(),
            characters: self.extract_character_data(&language_data.characters),
            bigrams: self.extract_ngram_data(&language_data.bigrams),
            skip_grams: self.extract_ngram_data(&language_data.skipgrams),
            skip2_grams: self.extract_ngram_data(&language_data.skipgrams2),
            skip3_grams: self.extract_ngram_data(&language_data.skipgrams3),
            trigrams: self.extract_trigram_data(&language_data.trigrams),
        };
    }

    pub fn index_char(&self, c: u8) -> char
    {
        *self.domain.get(c as usize).unwrap_or(&' ')
    }

    pub fn from<T>(&self, input: T) -> Vec<char>
    where
        T: IntoIterator<Item = u8>,
    {
        input.into_iter().map(|c| self.index_char(c)).collect()
    }

    pub fn char_to_vec_index(&mut self, c: char) -> u8
    {
        return if let Some(u) = self.codomain.get(&c)
        {
            *u
        }
        else
        {
            let new = self.len();
            self.domain.push(c);
            self.codomain.insert(c, new);

            new
        };
    }

    pub fn char_to_u8_lossy(&self, c: char) -> u8
    {
        return match self.codomain.get(&c)
        {
            | Some(u) => *u,
            | None => self.len(),
        };
    }

    pub fn ngram_to_indices<const N: usize>(&mut self, from: &[char; N]) -> [u8; N]
    {
        let mut indices = [0; N];

        for i in 0 .. N
        {
            indices[i] = self.char_to_vec_index(from[i]);
        }

        return indices;
    }

    pub fn to<T>(&mut self, input: T) -> Vec<u8>
    where
        T: IntoIterator<Item = char>,
    {
        input
            .into_iter()
            .map(|c| self.char_to_vec_index(c))
            .collect()
    }

    pub fn insert_single(&mut self, c: char)
    {
        let new = self.len();

        if let Entry::Vacant(entry) = self.codomain.entry(c)
        {
            self.domain.push(c);
            entry.insert(new);
        }
    }

    pub fn insert<T>(&mut self, input: T)
    where
        T: IntoIterator<Item = char>,
    {
        input.into_iter().for_each(|c| self.insert_single(c));
    }

    pub fn with_chars(s: &str) -> Self
    {
        let mut res = Self::default();
        res.insert(s.chars());

        return res;
    }

    pub fn as_string(&self, input: &[u8]) -> String
    {
        return input
            .iter()
            .map(|&c| self.domain.get(c as usize).unwrap_or(&' '))
            .collect();
    }

    pub fn len(&self) -> u8
    {
        debug_assert_eq!(self.codomain.len(), self.domain.len());

        return self.codomain.len() as u8;
    }

    pub fn is_empty(&self) -> bool
    {
        self.codomain.len() == 0
    }

    // Sus Global Mutation...
    pub fn extract_character_data(&mut self, data: &HashMap<char, f64>) -> Vec<f64>
    {
        return data
            .iter()
            .map(|(x, f)| {
                assert!(*f < 1.);

                self.insert_single(*x);

                return *f;
            })
            .collect_vec();
    }

    pub fn extract_ngram_data(&mut self, data: &HashMap<String, f64>) -> Vec<f64>
    {
        let len = 0 .. self.len();
        // vec![(0, '0'), (0, '1'), (1, '0'), (1, '1')]

        return len
            .clone()
            .cartesian_product(len)
            .map(|(c0, c1)| {
                let bigram = self.as_string(&[c0, c1]);
                let p = data.get(&bigram).cloned().unwrap_or(0.0);

                if c0 as usize * self.len() as usize + c1 as usize == 162
                {
                    println!("{} {} {}: {} | {}", 162, c0, c1, p, bigram);
                }

                return p;
            })
            .collect_vec();
    }

    pub fn extract_trigram_data(&mut self, data: &HashMap<String, f64>) -> Vec<f64>
    {
        let len = 0 .. self.len();

        return len
            .collect_vec()
            .iter()
            .combinations_with_replacement(3)
            .map(|x| {
                let trigram = self.as_string(&[*x[0], *x[1], *x[2]]);

                return data.get(&trigram).cloned().unwrap_or(0.0);
            })
            .collect_vec();
    }
}
