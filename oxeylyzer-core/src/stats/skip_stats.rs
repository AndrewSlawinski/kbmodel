use crate::language_data::LanguageData;
use crate::stats::predicates::Predicates;
use crate::stats::skip_stats::S1Type::*;
use crate::stats::skip_stats::S2Type::*;
use crate::stats::skip_stats::S3Type::*;
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
pub enum S1Type
{
    S1SFB,
    S1LSB,
    S1IRB,
    S1ORB,
    S1Rep,
    S1S,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum S2Type
{
    S2SFB,
    S2LSB,
    S2IRB,
    S2ORB,
    S2Rep,
    S2S,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum S3Type
{
    S3SFB,
    S3LSB,
    S3IRB,
    S3ORB,
    S3Rep,
    S3S,
}

impl S1Type
{
    #[inline]
    pub fn f(&self) -> fn(a: &[u8]) -> bool
    {
        return match self
        {
            | S1SFB => Predicates::is_sf,
            | S1LSB => Predicates::is_ls,
            | S1IRB => Predicates::is_inroll,
            | S1ORB => Predicates::is_outroll,
            | S1Rep => Predicates::all_equal,
            | S1S => Predicates::is_scissor,
        };
    }

    pub const fn default() -> [S1Type; 6]
    {
        return [S1SFB, S1LSB, S1IRB, S1ORB, S1Rep, S1S];
    }

    pub const fn source(language_data: &LanguageData) -> &HashMap<String, f32>
    {
        return &language_data.skipgrams;
    }
}

impl S2Type
{
    #[inline]
    pub const fn f(&self) -> fn(a: &[u8]) -> bool
    {
        return match self
        {
            | S2SFB => Predicates::is_sf,
            | S2LSB => Predicates::is_ls,
            | S2IRB => Predicates::is_inroll,
            | S2ORB => Predicates::is_outroll,
            | S2Rep => Predicates::all_equal,
            | S2S => Predicates::is_scissor,
        };
    }

    pub const fn default() -> [S2Type; 6]
    {
        return [S2SFB, S2LSB, S2IRB, S2ORB, S2Rep, S2S];
    }

    pub const fn source(language_data: &LanguageData) -> &HashMap<String, f32>
    {
        return &language_data.skipgrams2;
    }
}

impl S3Type
{
    #[inline]
    pub const fn f(&self) -> fn(a: &[u8]) -> bool
    {
        return match self
        {
            | S3SFB => Predicates::is_sf,
            | S3LSB => Predicates::is_ls,
            | S3IRB => Predicates::is_inroll,
            | S3ORB => Predicates::is_outroll,
            | S3Rep => Predicates::all_equal,
            | S3S => Predicates::is_scissor,
        };
    }

    pub const fn default() -> [S3Type; 6]
    {
        return [S3SFB, S3LSB, S3IRB, S3ORB, S3Rep, S3S];
    }

    pub const fn source(language_data: &LanguageData) -> &HashMap<String, f32>
    {
        return &language_data.skipgrams3;
    }
}

#[derive(Default, Clone)]
pub struct S1Stats
{
    pub inner: IndexMap<S1Type, f32>,
}

impl S1Stats
{
    #[inline]
    pub async fn new(
        chars: &Fixed<char>,
        language_data: &LanguageData,
        a: Option<&[S1Type]>,
    ) -> Self
    {
        let mut stats = IndexMap::new();

        Self::p2(
            chars,
            S1Type::source(language_data),
            &mut stats,
            a.unwrap_or(&S1Type::default()),
        );

        return Self { inner: stats };
    }

    fn p2(
        chars: &Fixed<char>,
        data: &HashMap<String, f32>,
        map: &mut IndexMap<S1Type, f32>,
        a: &[S1Type],
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

                for (key, value) in map.iter_mut()
                {
                    if key.f()(&[i, j])
                    {
                        let p = data.get(&format!("{c0}{c1}")).unwrap_or(&0.0);

                        *value += *p;
                    }
                }
            }
        }

        map.values_mut().for_each(|x| *x *= 100.);
    }
}

#[derive(Default, Clone)]
pub struct S2Stats
{
    pub inner: IndexMap<S2Type, f32>,
}

impl S2Stats
{
    #[inline]
    pub async fn new(
        chars: &Fixed<char>,
        language_data: &LanguageData,
        a: Option<&[S2Type]>,
    ) -> Self
    {
        let mut stats = IndexMap::new();

        Self::p2(
            chars,
            S2Type::source(language_data),
            &mut stats,
            a.unwrap_or(&S2Type::default()),
        );

        return Self { inner: stats };
    }

    fn p2(
        chars: &Fixed<char>,
        data: &HashMap<String, f32>,
        map: &mut IndexMap<S2Type, f32>,
        a: &[S2Type],
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

                for (key, value) in map.iter_mut()
                {
                    if key.f()(&[i, j])
                    {
                        let p = data.get(&format!("{c0}{c1}")).unwrap_or(&0.0);

                        *value += *p;
                    }
                }
            }
        }

        map.values_mut().for_each(|x| *x *= 100.);
    }
}

#[derive(Default, Clone)]
pub struct S3Stats
{
    pub inner: IndexMap<S3Type, f32>,
}

impl S3Stats
{
    #[inline]
    pub async fn new(
        chars: &Fixed<char>,
        language_data: &LanguageData,
        a: Option<&[S3Type]>,
    ) -> Self
    {
        let mut stats = IndexMap::new();

        Self::p2(
            chars,
            S3Type::source(language_data),
            &mut stats,
            a.unwrap_or(&S3Type::default()),
        );

        return Self { inner: stats };
    }

    fn p2(
        chars: &Fixed<char>,
        data: &HashMap<String, f32>,
        map: &mut IndexMap<S3Type, f32>,
        a: &[S3Type],
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

                for (key, value) in map.iter_mut()
                {
                    if key.f()(&[i, j])
                    {
                        let p = data.get(&format!("{c0}{c1}")).unwrap_or(&0.0);

                        *value += *p;
                    }
                }
            }
        }

        map.values_mut().for_each(|x| *x *= 100.);
    }
}

impl Index<S1Type> for S1Stats
{
    type Output = f32;

    fn index(&self, index: S1Type) -> &Self::Output
    {
        return &self.inner[&index];
    }
}

impl Index<S2Type> for S2Stats
{
    type Output = f32;

    fn index(&self, index: S2Type) -> &Self::Output
    {
        return &self.inner[&index];
    }
}

impl Index<S3Type> for S3Stats
{
    type Output = f32;

    fn index(&self, index: S3Type) -> &Self::Output
    {
        return &self.inner[&index];
    }
}

impl Display for S1Stats
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        let mut format = "Skip1:\n".to_string();

        self.inner.iter().for_each(|(key, value)| {
            let key = format!("{:?}", key);
            let value = format!("{:.3}%", value);

            let s = format!("{key:7}{:5}{value:0>7}\n", "");

            format.push_str(s.as_str());
        });

        write!(f, "{}", format.clone())
    }
}

impl Display for S2Stats
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        let mut format = "Skip2:\n".to_string();

        self.inner.iter().for_each(|(key, value)| {
            let key = format!("{:?}", key);
            let value = format!("{:.3}%", value);

            let s = format!("{key:7}{:5}{value:0>7}\n", "");

            format.push_str(s.as_str());
        });

        write!(f, "{}", format.clone())
    }
}

impl Display for S3Stats
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        let mut format = "Skip3:\n".to_string();

        self.inner.iter().for_each(|(key, value)| {
            let key = format!("{:?}", key);
            let value = format!("{:.3}%", value);

            let s = format!("{key:7}{:5}{value:0>7}\n", "");

            format.push_str(s.as_str());
        });

        write!(f, "{}", format.clone())
    }
}
