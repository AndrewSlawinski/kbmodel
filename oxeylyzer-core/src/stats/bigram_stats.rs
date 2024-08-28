use crate::language::language_data::LanguageData;
pub use std::collections::hash_map::Entry;

use crate::ngram::bigram_type::BigramType;
use crate::type_def::Fixed;
use crate::utility::pair::Pair;
use indexmap::IndexMap;
use std::fmt;
use std::fmt::{
    Display,
    Formatter,
};
use std::ops::Index;

#[derive(Default, Clone)]
pub struct BigramStats
{
    inner: IndexMap<BigramType, f64>,
}

impl Index<BigramType> for BigramStats
{
    type Output = f64;

    fn index(&self, index: BigramType) -> &Self::Output
    {
        return &self.inner[&index];
    }
}

impl BigramStats
{
    pub fn new(language_data: &LanguageData, char_indices: &Fixed<u8>) -> Self
    {
        let mut stats = IndexMap::new();

        for (bigram_type, bigrams) in vec![
            (BigramType::SFB, &language_data.bigrams),
            (BigramType::Skip1, &language_data.skip_grams),
            (BigramType::Skip2, &language_data.skip2_grams),
            (BigramType::Skip3, &language_data.skip3_grams),
        ]
        {
            stats.insert(
                bigram_type,
                Self::bigram_percent(char_indices, bigrams, language_data.characters.len()),
            );
        }

        return Self { inner: stats };
    }

    pub fn total_score(&self) -> f64
    {
        return self.inner.values().sum();
    }

    fn bigram_percent(char_indices: &Fixed<u8>, data: &Vec<f64>, chars_len: usize) -> f64
    {
        let mut res = 0.0;

        let mut bigram_counter = 0;

        for i in 0 .. 30
        {
            for j in 0 .. 30
            {
                if Pair(i, j).is_sfb()
                {
                    let c0 = char_indices[i] as usize;
                    let c1 = char_indices[j] as usize;

                    let k = c0 * chars_len + c1;
                    let p = data.get(k).unwrap_or(&0.0);

                    if bigram_counter == 0
                    {
                        println!("{} {} {}: {}", k, c0, c1, p);
                    }

                    res += p;

                    bigram_counter += 1;
                }
            }
        }

        return res;
    }
}

impl Display for BigramStats
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        let mut format = "".to_string();

        self.inner.iter().for_each(|(key, value)| {
            let s = format!("{:?}: {:.3}%\n", key.clone(), value.clone());

            format.push_str(s.as_str());
        });

        write!(f, "{}", format.clone())
    }
}
