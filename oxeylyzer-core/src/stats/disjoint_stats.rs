use crate::language_data::LanguageData;
use crate::stats::disjoint_stats::DType::*;
use crate::stats::layout_stats::LayoutStats;
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
pub enum DType
{
    D1SFB,
    D1LSB,
    D1IRB,
    D1ORB,
    D1Repeat,
    D1S,
}

impl DType
{
    fn f(&self) -> fn(a: &mut [u8]) -> bool
    {
        return match self
        {
            | D1SFB => LayoutStats::is_sf,
            | D1LSB => LayoutStats::is_lsb,
            | D1IRB => LayoutStats::is_inroll,
            | D1ORB => LayoutStats::is_outroll,
            | D1Repeat => LayoutStats::is_repeat,
            | D1S => LayoutStats::is_scissor,
        };
    }
}

#[derive(Default, Clone)]
pub struct D1Stats
{
    pub inner: IndexMap<DType, f32>,
}

impl D1Stats
{
    pub fn new(language_data: &LanguageData, chars: &Fixed<char>, a: &[DType]) -> Self
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
        index_map: &mut IndexMap<DType, f32>,
        a: &[DType],
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

                for k in 0 .. 30
                {
                    for t in a
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
            let i_left = LayoutStats::is_left_hand(&i);

            let b = (0 .. 30).into_par_iter().map(|j| {
                let j_left = LayoutStats::is_left_hand(&j);

                if i_left == j_left
                {
                    return 0.;
                }

                let c = (0 .. 30).into_par_iter().map(|k| {
                    let k_left = LayoutStats::is_left_hand(&k);

                    if j_left == k_left
                    {
                        return 0.;
                    }

                    if f(&mut [i as u8, k as u8])
                    {
                        let c0 = chars[i];
                        let c1 = chars[j];
                        let c2 = chars[k];

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
            let i_left = LayoutStats::is_left_hand(&i);

            for j in 0 .. 30
            {
                let j_left = LayoutStats::is_left_hand(&j);

                if i_left == j_left
                {
                    continue;
                }

                for k in 0 .. 30
                {
                    let k_left = LayoutStats::is_left_hand(&k);

                    if j_left == k_left
                    {
                        continue;
                    }

                    if f(&mut [i as u8, k as u8])
                    {
                        let c0 = chars[i];
                        let c1 = chars[j];
                        let c2 = chars[k];

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
            let k = format!("{:?}", key);
            let s = format!("  {:11} {:.3}%\n", k, *value);

            format.push_str(s.as_str());
        });

        write!(f, "{}", format.clone())
    }
}
