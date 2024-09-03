use crate::language_data::LanguageData;
use crate::stats::bigram_stats::BType::*;
use crate::stats::layout_stats::LayoutStats;
use crate::type_def::Fixed;
use indexmap::IndexMap;
pub use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{
    Display,
    Formatter,
};
use std::ops::Index;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum BType
{
    SFB,
    LSB,
    IRB,
    ORB,
    AB,
    Repeat,
    S,
}

impl BType
{
    fn f(&self) -> fn(a: &mut [u8]) -> bool
    {
        return match self
        {
            | SFB => LayoutStats::is_sf,
            | LSB => LayoutStats::is_lsb,
            | IRB => LayoutStats::is_sf,
            | ORB => LayoutStats::is_sf,
            | AB => LayoutStats::is_sf,
            | Repeat => LayoutStats::is_sf,
            | S => LayoutStats::is_sf,
        };
    }
}

#[derive(Default, Clone)]
pub struct BigramStats
{
    pub inner: IndexMap<BType, f32>,
}

impl Index<BType> for BigramStats
{
    type Output = f32;

    fn index(&self, index: BType) -> &Self::Output
    {
        return &self.inner[&index];
    }
}

impl BigramStats
{
    pub fn new(language_data: &LanguageData, chars: &Fixed<char>, a: &[BType]) -> Self
    {
        let mut stats = IndexMap::new();

        for t in a
        {
            let p = match t
            {
                | SFB => Self::p1(chars, &language_data.bigrams, LayoutStats::is_sf),
                | LSB => Self::p1(chars, &language_data.bigrams, LayoutStats::is_lsb),
                | Repeat => Self::p1(chars, &language_data.bigrams, LayoutStats::is_repeat),
                | S1SFB => Self::p1(chars, &language_data.skipgrams, LayoutStats::is_sf),
                | S2SFB => Self::p1(chars, &language_data.skipgrams2, LayoutStats::is_sf),
                | S3SFB => Self::p1(chars, &language_data.skipgrams3, LayoutStats::is_sf),
                | IRB => Self::p1(chars, &language_data.bigrams, LayoutStats::is_inroll),
                | ORB => Self::p1(chars, &language_data.bigrams, LayoutStats::is_outroll),
                | S => Self::p1(chars, &language_data.bigrams, LayoutStats::is_scissor),
                | AB => Self::p1(chars, &language_data.bigrams, LayoutStats::is_alternate),
            };

            stats.insert(*t, p);
        }

        return Self { inner: stats };
    }

    pub(crate) fn p1(
        chars: &Fixed<char>,
        data: &HashMap<String, f32>,
        f: fn(&mut [u8]) -> bool,
    ) -> f32
    {
        use rayon::iter::*;

        let a = (0 .. 30).into_par_iter().map(|i| {
            let b = (0 .. 30).into_par_iter().map(|j| {
                if f(&mut [i as u8, j as u8])
                {
                    let c0 = chars[i];
                    let c1 = chars[j];

                    if [c0, c1].iter().any(char::is_ascii_punctuation)
                    {
                        return 0.;
                    }

                    let p = data.get(&format!("{}{}", c0, c1)).unwrap_or(&0.);

                    return *p;
                }

                return 0.;
            });

            return b.collect::<Vec<f32>>().iter().sum();
        });

        let q: f32 = a.collect::<Vec<f32>>().iter().sum();

        return q * 100.;
    }

    pub(crate) fn p(
        chars: &Fixed<char>,
        data: &HashMap<String, f32>,
        f: fn(&mut [u8]) -> bool,
    ) -> f32
    {
        let mut res = 0.;

        for i in 0 .. 30
        {
            for j in 0 .. 30
            {
                if f(&mut [i as u8, j as u8])
                {
                    let c0 = chars[i];
                    let c1 = chars[j];

                    if [c0, c1].iter().any(char::is_ascii_punctuation)
                    {
                        continue;
                    }

                    let p = data.get(&format!("{}{}", c0, c1)).unwrap_or(&0.0);

                    res += p;
                }
            }
        }

        return res * 100.;
    }
}

impl Display for BigramStats
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        let mut format = "Bigrams:\n".to_string();

        self.inner.iter().for_each(|(key, value)| {
            let k = format!("{:?}", key);
            let s = format!("  {:11} {:.3}%\n", k, *value);

            format.push_str(s.as_str());
        });

        write!(f, "{}", format.clone())
    }
}
