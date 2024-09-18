use crate::language_data::LanguageData;
use crate::stats::predicates::Predicates;
use crate::stats::trigram_stats::TType::*;
use crate::type_def::Fixed;
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
    SameFingerT,
    IRT,
    ORT,
    Redirect,
    AlternateT,
    RepeatT,
}

impl TType
{
    #[inline]
    pub const fn f(&self) -> fn(a: &[u8]) -> bool
    {
        return match self
        {
            | SameFingerT => Predicates::is_sf,
            | IRT => Predicates::is_inroll,
            | ORT => Predicates::is_outroll,
            | Redirect => Predicates::is_redirect,
            | AlternateT => Predicates::is_alternate,
            | RepeatT => Predicates::all_equal,
        };
    }

    pub const fn default() -> [TType; 6]
    {
        return [SameFingerT, IRT, ORT, Redirect, AlternateT, RepeatT];
    }

    pub const fn source(language_data: &LanguageData) -> &HashMap<String, f32>
    {
        return &language_data.trigrams;
    }
}

#[derive(Default, Clone)]
pub struct TStats
{
    pub inner: IndexMap<TType, f32>,
}
impl TStats
{
    #[inline]
    pub async fn new(chars: &Fixed<char>, language_data: &LanguageData, a: Option<&[TType]>)
    -> Self
    {
        let mut stats = IndexMap::new();

        Self::p2(
            chars,
            TType::source(language_data),
            &mut stats,
            a.unwrap_or(&TType::default()),
        );

        return Self { inner: stats };
    }

    fn p2(
        chars: &Fixed<char>,
        data: &HashMap<String, f32>,
        map: &mut IndexMap<TType, f32>,
        a: &[TType],
    )
    {
        for t in a
        {
            map.insert(*t, 0.);
        }

        for i in 0 .. 30
        {
            let c0 = chars[i as usize];

            if char::is_ascii_punctuation(&c0)
            {
                continue;
            }

            for j in 0 .. 30
            {
                let c1 = chars[j as usize];

                if char::is_ascii_punctuation(&c1)
                {
                    continue;
                }

                for k in 0 .. 30
                {
                    let c2 = chars[k as usize];

                    if char::is_ascii_punctuation(&c2)
                    {
                        continue;
                    }

                    for (key, value) in map.iter_mut()
                    {
                        if key.f()(&mut [i, j, k])
                        {
                            let p = data.get(&format!("{c0}{c1}{c2}")).unwrap_or(&0.0);

                            *value += *p;
                        }
                    }
                }
            }
        }

        map.values_mut().for_each(|x| *x *= 100.);
    }

    #[allow(unused)]
    fn p_par(chars: &Fixed<char>, data: &HashMap<String, f32>, f: fn(&mut [u8]) -> bool) -> f32
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
}

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
            let key = format!("{key:?}");
            let value = format!("{value:.3}%");

            let s = format!("{key:7}{:5}{value:0>7}\n", "");

            format.push_str(s.as_str());
        });

        write!(f, "{}", format.clone())
    }
}
