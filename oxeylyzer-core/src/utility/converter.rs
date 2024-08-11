use std::collections::hash_map::Entry;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct Converter {
    domain: Vec<char>,
    codomain: HashMap<char, u8>,
}

impl Converter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn u8_to_char(&self, c: u8) -> char {
        *self.domain.get(c as usize).unwrap_or(&' ')
    }

    pub fn from<T>(&self, input: T) -> Vec<char>
        where
            T: IntoIterator<Item = u8>,
    {
        input.into_iter().map(|c| self.u8_to_char(c)).collect()
    }

    pub fn char_to_u8(&mut self, c: char) -> u8 {
        if let Some(u) = self.codomain.get(&c)
        {
            *u
        } else {
            let new = self.len();
            self.domain.push(c);
            self.codomain.insert(c, new);
            new
        }
    }

    pub fn char_to_u8_lossy(&self, c: char) -> u8 {
        return match self.codomain.get(&c) {
            | Some(u) => *u,
            | None => self.len(),
        };
    }

    pub fn bigram_to_u8_bigram(&mut self, from: [char; 2]) -> [u8; 2] {
        [self.char_to_u8(from[0]), self.char_to_u8(from[1])]
    }

    pub fn trigram_to_u8_trigram(&mut self, from: [char; 3]) -> [u8; 3] {
        [
            self.char_to_u8(from[0]),
            self.char_to_u8(from[1]),
            self.char_to_u8(from[2]),
        ]
    }

    pub fn to<T>(&mut self, input: T) -> Vec<u8>
        where
            T: IntoIterator<Item = char>,
    {
        input.into_iter().map(|c| self.char_to_u8(c)).collect()
    }

    pub fn to_bigram_lossy(&self, from: [char; 2], char_count: usize) -> usize {
        let c1 = self.char_to_u8_lossy(from[0]) as usize;
        let c2 = self.char_to_u8_lossy(from[1]) as usize;
        if c1 < char_count && c2 < char_count
        {
            c1 * char_count + c2
        } else {
            u8::MAX as usize
        }
    }

    pub fn trigram_to_u8_trigram_lossy(&self, from: [char; 3]) -> [u8; 3] {
        return [
            self.char_to_u8_lossy(from[0]),
            self.char_to_u8_lossy(from[1]),
            self.char_to_u8_lossy(from[2]),
        ];
    }

    pub fn to_lossy<T>(&self, input: T) -> Vec<u8>
        where
            T: IntoIterator<Item = char>,
    {
        return input.into_iter().map(|c| self.char_to_u8_lossy(c)).collect();
    }

    pub fn insert_single(&mut self, c: char) {
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

    pub fn with_chars(s: &str) -> Self {
        let mut res = Self::default();
        res.insert(s.chars());

        return res;
    }

    pub fn as_string(&self, input: &[u8]) -> String {
        return input.iter().map(|&c| self.domain.get(c as usize).unwrap_or(&' ')).collect();
    }

    pub fn len(&self) -> u8 {
        debug_assert_eq!(self.codomain.len(), self.domain.len());

        return self.codomain.len() as u8;
    }

    pub fn is_empty(&self) -> bool {
        self.codomain.len() == 0
    }
}
