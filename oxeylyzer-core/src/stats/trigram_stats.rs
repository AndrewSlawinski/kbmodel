use crate::language_data::LanguageData;
use crate::stats::layout_stats::LayoutStats;
use crate::stats::trigram_stats::TType::*;
use crate::type_def::Fixed;
use indexmap::map::Entry;
use indexmap::IndexMap;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{
    Display,
    Formatter,
};
use std::ops::Index;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum TType
{
    SFT,
    IRT,
    ORT,
    Redirect,
    AT,
}

impl TType
{
    fn f(&self) -> fn(a: &mut [u8]) -> bool
    {
        return match self
        {
            | SFT => LayoutStats::is_sf,
            | IRT => LayoutStats::is_inroll,
            | ORT => LayoutStats::is_outroll,
            | Redirect => LayoutStats::is_redirect,
            | AT => LayoutStats::is_inroll,
        };
    }
}

#[derive(Default, Clone)]
pub struct TStats
{
    pub inner: IndexMap<TType, f32>,
}
impl TStats
{
    pub fn new(language_data: &LanguageData, chars: &Fixed<char>, a: &[TType]) -> Self
    {
        let mut stats = IndexMap::new();

        for t in a
        {
            stats.insert(*t, 0.);
        }

        Self::p2(chars, &language_data.trigrams, &mut stats, a);

        return Self { inner: stats };
    }

    pub(crate) fn p2(
        chars: &Fixed<char>,
        data: &HashMap<String, f32>,
        index_map: &mut IndexMap<TType, f32>,
        a: &[TType],
    )
    {
        for i in 0 .. 30
        {
            for j in 0 .. 30
            {
                for k in 0 .. 30
                {
                    for t in a
                    {
                        if t.f()(&mut [i as u8, j as u8, k as u8])
                        {
                            let c0 = chars[i];
                            let c1 = chars[j];
                            let c2 = chars[k];

                            if [c0, c1, c2].iter().any(char::is_ascii_punctuation)
                            {
                                continue;
                            }

                            let p = data.get(&format!("{}{}{}", c0, c1, c2)).unwrap_or(&0.0);

                            match index_map.entry(*t)
                            {
                                | Entry::Occupied(mut e) =>
                                {
                                    *e.get_mut() += p;
                                },
                                | Entry::Vacant(_) =>
                                {
                                    panic!();
                                },
                            }
                        }
                    }
                }
            }
        }

        index_map.values_mut().for_each(|mut x| *x *= 100.);
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
                let c = (0 .. 30).into_par_iter().map(|k| {
                    if f(&mut [i as u8, j as u8, k as u8])
                    {
                        let c0 = chars[i];
                        let c1 = chars[j];
                        let c2 = chars[k];

                        if c0 == c1 && c1 == c2
                        {
                            return 0.;
                        }

                        if [c0, c1, c2].iter().any(char::is_ascii_punctuation)
                        {
                            return 0.;
                        }

                        let p = data.get(&format!("{}{}{}", c0, c1, c2)).unwrap_or(&0.);

                        return *p;
                    }

                    return 0.;
                });

                return c.collect::<Vec<f32>>().iter().sum();
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
                for k in 0 .. 30
                {
                    if f(&mut [i as u8, j as u8, k as u8])
                    {
                        let c0 = chars[i];
                        let c1 = chars[j];
                        let c2 = chars[k];

                        if c0 == c1 && c1 == c2
                        {
                            continue;
                        }

                        if [c0, c1, c2].iter().any(char::is_ascii_punctuation)
                        {
                            continue;
                        }

                        let p = data.get(&format!("{}{}{}", c0, c1, c2)).unwrap_or(&0.0);

                        res += p;
                    }
                }
            }
        }

        return res * 100.;
    }
}

impl TStats {}

impl Index<TType> for TStats
{
    type Output = f32;

    fn index(&self, index: TType) -> &Self::Output
    {
        return &self.inner[&index];
    }
}

impl Display for TStats
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        let mut format = "Trigrams:\n".to_string();

        self.inner.iter().for_each(|(key, value)| {
            let k = format!("{:?}", key);
            let s = format!("  {:11} {:.3}%\n", k, *value);

            format.push_str(s.as_str());
        });

        write!(f, "{}", format.clone())
    }
}
