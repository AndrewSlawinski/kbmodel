use crate::language_data::LanguageData;
use crate::stats::layout_stats::LayoutStats;
use crate::stats::skip_stats::S1Type::*;
use crate::stats::skip_stats::S2Type::*;
use crate::stats::skip_stats::S3Type::*;
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
pub enum S1Type
{
    S1SFB,
    S1LSB,
    S1IRB,
    S1ORB,
    S1Repeat,
    S1S,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum S2Type
{
    S2SFB,
    S2LSB,
    S2IRB,
    S2ORB,
    S2Repeat,
    S2S,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum S3Type
{
    S3SFB,
    S3LSB,
    S3IRB,
    S3ORB,
    S3Repeat,
    S3S,
}

impl S1Type
{
    fn f(&self) -> fn(a: &mut [u8]) -> bool
    {
        return match self
        {
            | S1SFB => LayoutStats::is_sf,
            | S1LSB => LayoutStats::is_lsb,
            | S1IRB => LayoutStats::is_inroll,
            | S1ORB => LayoutStats::is_outroll,
            | S1Repeat => LayoutStats::is_repeat,
            | S1S => LayoutStats::is_scissor,
        };
    }
}

impl S2Type
{
    fn f(&self) -> fn(a: &mut [u8]) -> bool
    {
        return match self
        {
            | S2SFB => LayoutStats::is_sf,
            | S2LSB => LayoutStats::is_lsb,
            | S2IRB => LayoutStats::is_inroll,
            | S2ORB => LayoutStats::is_outroll,
            | S2Repeat => LayoutStats::is_repeat,
            | S2S => LayoutStats::is_scissor,
        };
    }
}

impl S3Type
{
    fn f(&self) -> fn(a: &mut [u8]) -> bool
    {
        return match self
        {
            | S3SFB => LayoutStats::is_sf,
            | S3LSB => LayoutStats::is_lsb,
            | S3IRB => LayoutStats::is_inroll,
            | S3ORB => LayoutStats::is_outroll,
            | S3Repeat => LayoutStats::is_repeat,
            | S3S => LayoutStats::is_scissor,
        };
    }
}

#[derive(Default, Clone)]
pub struct S1Stats
{
    pub inner: IndexMap<S1Type, f32>,
}

impl S1Stats
{
    pub fn new(language_data: &LanguageData, chars: &Fixed<char>, a: &[S1Type]) -> Self
    {
        let mut stats = IndexMap::new();

        Self::p2(chars, &language_data.trigrams, &mut stats, a);

        return Self { inner: stats };
    }

    pub(crate) fn p2(
        chars: &Fixed<char>,
        data: &HashMap<String, f32>,
        index_map: &mut IndexMap<S1Type, f32>,
        a: &[S1Type],
    )
    {
        for i in 0 .. 30
        {
            let i_left = LayoutStats::is_left_hand(&i);

            for j in 0 .. 30
            {
                let j_left = LayoutStats::is_left_hand(&j);

                if i_left == j_left
                {
                    continue;
                }

                for t in a
                {
                    for k in 0 .. 30
                    {
                        if t.f()(&mut [i as u8, k as u8])
                        {
                            let k_left = LayoutStats::is_left_hand(&k);

                            if j_left == k_left
                            {
                                continue;
                            }

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
                                | Entry::Vacant(e) =>
                                {
                                    *e.insert(*p);
                                },
                            }
                        }
                    }
                }
            }
        }

        index_map.values_mut().for_each(|mut x| *x *= 100.);
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

impl Display for S1Stats
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        let mut format = "Disjoints:\n".to_string();

        self.inner.iter().for_each(|(key, value)| {
            let k = format!("{:?}", key);
            let s = format!("  {:11} {:.3}%\n", k, *value);

            format.push_str(s.as_str());
        });

        write!(f, "{}", format.clone())
    }
}

#[derive(Default, Clone)]
pub struct S2Stats
{
    pub inner: IndexMap<S2Type, f32>,
}

impl S2Stats
{
    pub fn new(language_data: &LanguageData, chars: &Fixed<char>, a: &[S2Type]) -> Self
    {
        let mut stats = IndexMap::new();

        Self::p2(chars, &language_data.trigrams, &mut stats, a);

        return Self { inner: stats };
    }

    pub(crate) fn p2(
        chars: &Fixed<char>,
        data: &HashMap<String, f32>,
        index_map: &mut IndexMap<S2Type, f32>,
        a: &[S2Type],
    )
    {
        for i in 0 .. 30
        {
            let i_left = LayoutStats::is_left_hand(&i);

            for j in 0 .. 30
            {
                let j_left = LayoutStats::is_left_hand(&j);

                if i_left == j_left
                {
                    continue;
                }

                for t in a
                {
                    for k in 0 .. 30
                    {
                        if t.f()(&mut [i as u8, k as u8])
                        {
                            let k_left = LayoutStats::is_left_hand(&k);

                            if j_left == k_left
                            {
                                continue;
                            }

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
                                | Entry::Vacant(e) =>
                                {
                                    *e.insert(*p);
                                },
                            }
                        }
                    }
                }
            }
        }

        index_map.values_mut().for_each(|mut x| *x *= 100.);
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

impl Display for S2Stats
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        let mut format = "Disjoints:\n".to_string();

        self.inner.iter().for_each(|(key, value)| {
            let k = format!("{:?}", key);
            let s = format!("  {:11} {:.3}%\n", k, *value);

            format.push_str(s.as_str());
        });

        write!(f, "{}", format.clone())
    }
}

#[derive(Default, Clone)]
pub struct S3Stats
{
    pub inner: IndexMap<S3Type, f32>,
}

impl S3Stats
{
    pub fn new(language_data: &LanguageData, chars: &Fixed<char>, a: &[S3Type]) -> Self
    {
        let mut stats = IndexMap::new();

        Self::p2(chars, &language_data.trigrams, &mut stats, a);

        return Self { inner: stats };
    }

    pub(crate) fn p2(
        chars: &Fixed<char>,
        data: &HashMap<String, f32>,
        index_map: &mut IndexMap<S3Type, f32>,
        a: &[S3Type],
    )
    {
        for i in 0 .. 30
        {
            let i_left = LayoutStats::is_left_hand(&i);

            for j in 0 .. 30
            {
                let j_left = LayoutStats::is_left_hand(&j);

                if i_left == j_left
                {
                    continue;
                }

                for t in a
                {
                    for k in 0 .. 30
                    {
                        if t.f()(&mut [i as u8, k as u8])
                        {
                            let k_left = LayoutStats::is_left_hand(&k);

                            if j_left == k_left
                            {
                                continue;
                            }

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
                                | Entry::Vacant(e) =>
                                {
                                    *e.insert(*p);
                                },
                            }
                        }
                    }
                }
            }
        }

        index_map.values_mut().for_each(|mut x| *x *= 100.);
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

impl Display for S3Stats
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        let mut format = "Disjoints:\n".to_string();

        self.inner.iter().for_each(|(key, value)| {
            let k = format!("{:?}", key);
            let s = format!("  {:11} {:.3}%\n", k, *value);

            format.push_str(s.as_str());
        });

        write!(f, "{}", format.clone())
    }
}
