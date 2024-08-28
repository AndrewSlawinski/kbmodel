use smartstring::{
    SmartString,
    SmartStringMode,
};
use std::collections::HashMap;

#[derive(Default)]
pub struct TextNgrams<'a, const N: usize>
{
    pub ngrams: HashMap<&'a str, usize>,
}

impl<'a, const N: usize> TextNgrams<'a, N>
{
    pub fn new<M>(s: &'a str, last: &'a SmartString<M>) -> Self
    where
        M: SmartStringMode,
    {
        let mut ngrams = HashMap::default();

        let it1 = s.char_indices().map(|(i, _)| i);
        let it2 = s.char_indices().map(|(i, _)| i).skip(N);

        it1.zip(it2).map(|(i1, i2)| &s[i1 .. i2]).for_each(|ngram| {
            ngrams.entry(ngram).and_modify(|f| *f += 1).or_insert(1);
        });

        let it1 = last.char_indices().map(|(i, _)| i);
        let it2 = last.char_indices().map(|(i, _)| i).skip(N);

        it1.zip(it2)
            .map(|(i1, i2)| &last[i1 .. i2])
            .for_each(|ngram| {
                ngrams.entry(ngram).and_modify(|f| *f += 1).or_insert(1);
            });

        return Self { ngrams };
    }

    pub fn combine_with(mut self, rhs: Self) -> Self
    {
        for (trigram, freq) in rhs.ngrams.into_iter()
        {
            self.ngrams
                .entry(trigram)
                .and_modify(|f| *f += freq)
                .or_insert(freq);
        }

        return self;
    }
}
