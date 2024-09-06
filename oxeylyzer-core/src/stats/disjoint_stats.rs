use crate::language_data::LanguageData;
use crate::stats::disjoint_stats::DType::*;
use crate::stats::predicates::Predicates;
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
pub enum DType
{
    D1SFB,
    D1LSB,
    D1IRB,
    D1ORB,
    D1Rep,
    D1S,
}

impl DType
{
    #[inline]
    pub fn f(&self) -> fn(a: &[u8]) -> bool
    {
        return match self
        {
            | D1SFB => Predicates::is_sf,
            | D1LSB => Predicates::is_ls,
            | D1IRB => Predicates::is_inroll,
            | D1ORB => Predicates::is_outroll,
            | D1Rep => Predicates::all_equal,
            | D1S => Predicates::is_scissor,
        };
    }

    pub const fn default() -> [DType; 6]
    {
        return [D1SFB, D1LSB, D1IRB, D1ORB, D1Rep, D1S];
    }

    pub const fn source(language_data: &LanguageData) -> &HashMap<String, f32>
    {
        return &language_data.trigrams;
    }
}

#[derive(Default, Clone)]
pub struct D1Stats
{
    pub inner: IndexMap<DType, f32>,
}

impl D1Stats
{
    #[inline]
    pub async fn new(chars: &Fixed<char>, language_data: &LanguageData, a: Option<&[DType]>)
    -> Self
    {
        let mut stats = IndexMap::new();

        Self::p2(
            chars,
            DType::source(language_data),
            &mut stats,
            a.unwrap_or(&DType::default()),
        );

        return Self { inner: stats };
    }

    fn p2(
        chars: &Fixed<char>,
        data: &HashMap<String, f32>,
        map: &mut IndexMap<DType, f32>,
        a: &[DType],
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

            let i_left = Predicates::is_left_hand(&i);

            for j in 0 .. 30
            {
                let j_left = Predicates::is_left_hand(&j);

                if i_left == j_left
                {
                    continue;
                }

                let c1 = chars[j as usize];

                if char::is_ascii_punctuation(&c1)
                {
                    continue;
                }

                for k in 0 .. 30
                {
                    if j_left == Predicates::is_left_hand(&k)
                    {
                        continue;
                    }

                    let c2 = chars[k as usize];

                    if char::is_ascii_punctuation(&c2)
                    {
                        continue;
                    }

                    for (key, value) in map.iter_mut()
                    {
                        if key.f()(&mut [i, k])
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
}

impl Index<DType> for D1Stats
{
    type Output = f32;

    fn index(&self, index: DType) -> &Self::Output
    {
        return &self.inner[&index];
    }
}

impl Display for D1Stats
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        let mut format = "Disjoints:\n".to_string();

        self.inner.iter().for_each(|(key, value)| {
            let key = format!("{:?}", key);
            let value = format!("{:.3}%", value);

            let s = format!("{key:7}{:5}{value:0>7}\n", "");

            format.push_str(s.as_str());
        });

        write!(f, "{}", format.clone())
    }
}
