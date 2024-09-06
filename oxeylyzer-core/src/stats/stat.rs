use crate::language_data::LanguageData;
use crate::stats::bigram_stats::BType;
use crate::stats::character_stats::CType;
use crate::stats::disjoint_stats::DType;
use crate::stats::predicates::Predicates;
use crate::stats::skip_stats::{
    S1Type,
    S2Type,
    S3Type,
};
use crate::stats::trigram_stats::TType;
use crate::type_def::Fixed;
use core::fmt::{
    Display,
    Formatter,
};
use indexmap::IndexMap;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use std::collections::HashMap;
use std::fmt;
use std::ops::Index;

#[derive(Default, Clone)]
pub struct Stat<T>
{
    pub stats: IndexMap<T, f32>,
}

#[allow(dead_code)]
impl<T> Stat<T>
{
    fn p1(chars: &Fixed<char>, data: &HashMap<char, f32>, f: fn(&u8) -> bool) -> f32
    {
        let mut v = 0.;

        for i in 0 .. 30
        {
            let c0 = chars[i as usize];

            if char::is_ascii_punctuation(&c0)
            {
                continue;
            }

            if f(&i)
            {
                let p = data.get(&c0).unwrap_or(&0.0);

                v += *p;
            }
        }

        return v * 100.;
    }

    fn p1_punct(chars: &Fixed<char>, data: &HashMap<char, f32>, f: fn(&u8) -> bool) -> f32
    {
        let mut v = 0.;

        for i in 0 .. 30
        {
            let c0 = chars[i as usize];

            if f(&i)
            {
                v += *data.get(&c0).unwrap_or(&0.0);
            }
        }

        return v * 100.;
    }

    fn p2(chars: &Fixed<char>, data: &HashMap<String, f32>, f: fn(&[u8]) -> bool) -> f32
    {
        let mut v = 0.;

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

                if f(&[i, j])
                {
                    v += *data.get(&format!("{c0}{c1}")).unwrap_or(&0.0);
                }
            }
        }

        return v * 100.;
    }

    fn p2_punct(chars: &Fixed<char>, data: &HashMap<String, f32>, f: fn(&[u8]) -> bool) -> f32
    {
        let mut v = 0.;

        for i in 0 .. 30
        {
            let c0 = chars[i as usize];

            for j in 0 .. 30
            {
                let c1 = chars[j as usize];

                if f(&[i, j])
                {
                    v += *data.get(&format!("{c0}{c1}")).unwrap_or(&0.0);
                }
            }
        }

        return v * 100.;
    }

    fn p3(chars: &Fixed<char>, data: &HashMap<String, f32>, f: fn(&[u8]) -> bool) -> f32
    {
        let mut v = 0.;

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

                    if f(&[i, j, k])
                    {
                        v += data.get(&format!("{c0}{c1}{c2}")).unwrap_or(&0.0);
                    }
                }
            }
        }

        return v * 100.;
    }

    fn p3_punct(chars: &Fixed<char>, data: &HashMap<String, f32>, f: fn(&[u8]) -> bool) -> f32
    {
        let mut v = 0.;

        for i in 0 .. 30
        {
            let c0 = chars[i as usize];

            for j in 0 .. 30
            {
                let c1 = chars[j as usize];

                for k in 0 .. 30
                {
                    let c2 = chars[k as usize];

                    if f(&[i, j, k])
                    {
                        v += data.get(&format!("{c0}{c1}{c2}")).unwrap_or(&0.0);
                    }
                }
            }
        }

        return v * 100.;
    }

    fn p3_m1(chars: &Fixed<char>, data: &HashMap<String, f32>, f: fn(&[u8]) -> bool) -> f32
    {
        let mut v = 0.;

        for i in 0 .. 30
        {
            let c0 = chars[i as usize];

            if char::is_ascii_punctuation(&c0)
            {
                continue;
            }

            let i_left = i % 10 < 5;

            for j in 0 .. 30
            {
                let j_left = j % 10 < 5;
                let c1 = chars[j as usize];

                if char::is_ascii_punctuation(&c1) || i_left == j_left
                {
                    continue;
                }

                for k in 0 .. 30
                {
                    let c2 = chars[k as usize];

                    if char::is_ascii_punctuation(&c2) || j_left == (k % 10 < 5)
                    {
                        continue;
                    }

                    if f(&[i, k])
                    {
                        v += data.get(&format!("{c0}{c1}{c2}")).unwrap_or(&0.0);
                    }
                }
            }
        }

        return v * 100.;
    }

    fn p3_m1_punct(chars: &Fixed<char>, data: &HashMap<String, f32>, f: fn(&[u8]) -> bool) -> f32
    {
        let mut v = 0.;

        for i in 0 .. 30
        {
            let c0 = chars[i as usize];

            let i_left = i % 10 < 5;

            for j in 0 .. 30
            {
                let j_left = j % 10 < 5;

                if i_left == j_left
                {
                    continue;
                }

                let c1 = chars[j as usize];

                for k in 0 .. 30
                {
                    if j_left == (k % 10 < 5)
                    {
                        continue;
                    }

                    let c2 = chars[k as usize];

                    if f(&[i, k])
                    {
                        v += data.get(&format!("{c0}{c1}{c2}")).unwrap_or(&0.0);
                    }
                }
            }
        }

        return v * 100.;
    }
}

#[allow(dead_code)]
impl<T> Stat<T>
{
    fn p1_par(chars: &Fixed<char>, data: &HashMap<char, f32>, f: fn(&u8) -> bool) -> f32
    {
        return 100.
            * (0_u8 .. 30_u8)
                .into_par_iter()
                .fold(
                    || 0.,
                    |acc, i| {
                        let c0 = chars[i as usize];

                        acc + if char::is_ascii_punctuation(&c0)
                        {
                            0.
                        }
                        else if f(&i)
                        {
                            *data.get(&c0).unwrap_or(&0.)
                        }
                        else
                        {
                            0.
                        }
                    },
                )
                .sum::<f32>();
    }

    fn p2_par(chars: &Fixed<char>, data: &HashMap<String, f32>, f: fn(&[u8]) -> bool) -> f32
    {
        return 100.
            * (0_u8 .. 30_u8)
                .into_par_iter()
                .fold(
                    || 0.,
                    |acc0, i| {
                        let c0 = chars[i as usize];

                        acc0 + if char::is_ascii_punctuation(&c0)
                        {
                            0.
                        }
                        else
                        {
                            (0_u8 .. 30_u8)
                                .into_par_iter()
                                .fold(
                                    || 0.,
                                    |acc1, j| {
                                        let c1 = chars[j as usize];

                                        acc1 + if char::is_ascii_punctuation(&c1)
                                        {
                                            0.
                                        }
                                        else if f(&[i, j])
                                        {
                                            *data.get(&format!("{c0}{c1}")).unwrap_or(&0.)
                                        }
                                        else
                                        {
                                            0.
                                        }
                                    },
                                )
                                .sum::<f32>()
                        }
                    },
                )
                .sum::<f32>();
    }

    fn p3_par(chars: &Fixed<char>, data: &HashMap<String, f32>, f: fn(&[u8]) -> bool) -> f32
    {
        return 100.
            * (0_u8 .. 30_u8)
                .into_par_iter()
                .fold(
                    || 0.,
                    |acc0, i| {
                        let c0 = chars[i as usize];

                        acc0 + if char::is_ascii_punctuation(&c0)
                        {
                            0.
                        }
                        else
                        {
                            (0_u8 .. 30_u8)
                                .into_par_iter()
                                .fold(
                                    || 0.,
                                    |acc1, j| {
                                        let c1 = chars[j as usize];

                                        acc1 + if char::is_ascii_punctuation(&c1)
                                        {
                                            0.
                                        }
                                        else
                                        {
                                            (0_u8 .. 30_u8)
                                                .into_par_iter()
                                                .fold(
                                                    || 0.,
                                                    |acc2, k| {
                                                        let c2 = chars[k as usize];

                                                        acc2 + if char::is_ascii_punctuation(&c2)
                                                        {
                                                            0.
                                                        }
                                                        else if f(&[i, j, k])
                                                        {
                                                            *data
                                                                .get(&format!("{c0}{c1}{c2}"))
                                                                .unwrap_or(&0.)
                                                        }
                                                        else
                                                        {
                                                            0.
                                                        }
                                                    },
                                                )
                                                .sum::<f32>()
                                        }
                                    },
                                )
                                .sum::<f32>()
                        }
                    },
                )
                .sum::<f32>();
    }

    fn p3_m1_par(chars: &Fixed<char>, data: &HashMap<String, f32>, f: fn(&[u8]) -> bool) -> f32
    {
        return 100.
            * (0_u8 .. 30_u8)
                .into_par_iter()
                .fold(
                    || 0.,
                    |acc0, i| {
                        let c0 = chars[i as usize];

                        acc0 + if char::is_ascii_punctuation(&c0)
                        {
                            0.
                        }
                        else
                        {
                            let i_left = Predicates::is_left_hand(&i);

                            (0_u8 .. 30_u8)
                                .into_par_iter()
                                .fold(
                                    || 0.,
                                    |acc1, j| {
                                        let j_left = Predicates::is_left_hand(&j);

                                        let c1 = chars[j as usize];

                                        acc1 + if char::is_ascii_punctuation(&c1)
                                            || i_left == j_left
                                        {
                                            0.
                                        }
                                        else
                                        {
                                            (0_u8 .. 30_u8)
                                                .into_par_iter()
                                                .fold(
                                                    || 0.,
                                                    |acc2, k| {
                                                        let c2 = chars[k as usize];

                                                        acc2 + if char::is_ascii_punctuation(&c2)
                                                            || j_left
                                                                == Predicates::is_left_hand(&k)
                                                        {
                                                            0.
                                                        }
                                                        else if f(&[i, k])
                                                        {
                                                            *data
                                                                .get(&format!("{c0}{c1}{c2}"))
                                                                .unwrap_or(&0.)
                                                        }
                                                        else
                                                        {
                                                            0.
                                                        }
                                                    },
                                                )
                                                .sum::<f32>()
                                        }
                                    },
                                )
                                .sum::<f32>()
                        }
                    },
                )
                .sum::<f32>();
    }
}

impl Stat<CType>
{
    #[inline]
    pub async fn new_c(
        chars: &Fixed<char>,
        language_data: &LanguageData,
        a: Option<&[CType]>,
    ) -> Self
    {
        let mut stats = IndexMap::new();

        for t in a.unwrap_or(&CType::default())
        {
            let p = Self::p1(chars, CType::source(language_data), t.f());

            stats.insert(*t, p);
        }

        return Self { stats };
    }
}

impl Stat<BType>
{
    #[inline]
    pub async fn new_b(
        chars: &Fixed<char>,
        language_data: &LanguageData,
        a: Option<&[BType]>,
    ) -> Self
    {
        let mut stats = IndexMap::new();

        for t in a.unwrap_or(&BType::default())
        {
            let p = Self::p2(chars, BType::source(language_data), t.f());

            stats.insert(*t, p);
        }

        return Self { stats };
    }
}

impl Stat<TType>
{
    #[inline]
    pub async fn new_t(
        chars: &Fixed<char>,
        language_data: &LanguageData,
        a: Option<&[TType]>,
    ) -> Self
    {
        let mut stats = IndexMap::new();

        for t in a.unwrap_or(&TType::default())
        {
            let p = Self::p3(chars, TType::source(language_data), t.f());

            stats.insert(*t, p);
        }

        return Self { stats };
    }
}

impl Stat<DType>
{
    #[inline]
    pub async fn new_d(
        chars: &Fixed<char>,
        language_data: &LanguageData,
        a: Option<&[DType]>,
    ) -> Self
    {
        let mut stats = IndexMap::new();

        for t in a.unwrap_or(&DType::default())
        {
            let p = Self::p3_m1(chars, DType::source(language_data), t.f());

            stats.insert(*t, p);
        }

        return Self { stats };
    }
}

impl Stat<S1Type>
{
    #[inline]
    pub async fn new_s1(
        chars: &Fixed<char>,
        language_data: &LanguageData,
        a: Option<&[S1Type]>,
    ) -> Self
    {
        let mut stats = IndexMap::new();

        for t in a.unwrap_or(&S1Type::default())
        {
            let p = Self::p2(chars, S1Type::source(language_data), t.f());

            stats.insert(*t, p);
        }

        return Self { stats };
    }
}

impl Stat<S2Type>
{
    #[inline]
    pub async fn new_s2(
        chars: &Fixed<char>,
        language_data: &LanguageData,
        a: Option<&[S2Type]>,
    ) -> Self
    {
        let mut stats = IndexMap::new();

        for t in a.unwrap_or(&S2Type::default())
        {
            let p = Self::p2(chars, S2Type::source(language_data), t.f());

            stats.insert(*t, p);
        }

        return Self { stats };
    }
}

impl Stat<S3Type>
{
    #[inline]
    pub async fn new_s3(
        chars: &Fixed<char>,
        language_data: &LanguageData,
        a: Option<&[S3Type]>,
    ) -> Self
    {
        let mut stats = IndexMap::new();

        for t in a.unwrap_or(&S3Type::default())
        {
            let p = Self::p2(chars, S3Type::source(language_data), t.f());

            stats.insert(*t, p);
        }

        return Self { stats };
    }
}

impl Default for Stat<CType>
{
    fn default() -> Self
    {
        Self {
            stats: IndexMap::new(),
        }
    }
}

impl Default for Stat<BType>
{
    fn default() -> Self
    {
        Self {
            stats: IndexMap::new(),
        }
    }
}

impl Default for Stat<TType>
{
    fn default() -> Self
    {
        Self {
            stats: IndexMap::new(),
        }
    }
}

impl Default for Stat<DType>
{
    fn default() -> Self
    {
        Self {
            stats: IndexMap::new(),
        }
    }
}

impl Default for Stat<S1Type>
{
    fn default() -> Self
    {
        Self {
            stats: IndexMap::new(),
        }
    }
}

impl Default for Stat<S2Type>
{
    fn default() -> Self
    {
        Self {
            stats: IndexMap::new(),
        }
    }
}

impl Default for Stat<S3Type>
{
    fn default() -> Self
    {
        Self {
            stats: IndexMap::new(),
        }
    }
}

impl Index<&CType> for Stat<CType>
{
    type Output = f32;

    fn index(&self, index: &CType) -> &Self::Output
    {
        return &self.stats[index];
    }
}

impl Index<&BType> for Stat<BType>
{
    type Output = f32;

    fn index(&self, index: &BType) -> &Self::Output
    {
        return &self.stats[index];
    }
}

impl Index<&TType> for Stat<TType>
{
    type Output = f32;

    fn index(&self, index: &TType) -> &Self::Output
    {
        return &self.stats[index];
    }
}

impl Index<&DType> for Stat<DType>
{
    type Output = f32;

    fn index(&self, index: &DType) -> &Self::Output
    {
        return &self.stats[index];
    }
}

impl Index<&S1Type> for Stat<S1Type>
{
    type Output = f32;

    fn index(&self, index: &S1Type) -> &Self::Output
    {
        return &self.stats[index];
    }
}

impl Index<&S2Type> for Stat<S2Type>
{
    type Output = f32;

    fn index(&self, index: &S2Type) -> &Self::Output
    {
        return &self.stats[index];
    }
}

impl Index<&S3Type> for Stat<S3Type>
{
    type Output = f32;

    fn index(&self, index: &S3Type) -> &Self::Output
    {
        return &self.stats[index];
    }
}

impl Display for Stat<CType>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        let mut format = "Columns:\n".to_string();

        for i in 0 .. self.stats.len() / 2
        {
            let (key0, value0) = self.stats.get_index(i).unwrap();
            let (key1, value1) = self.stats.get_index(self.stats.len() - (i + 1)).unwrap();

            let key0 = format!("{key0:?}");

            let mut value0 = format!("{value0:.3}%");
            value0 = format!("{value0:0>7}");

            let key1 = format!("{key1:?}");

            let mut value1 = format!("{value1:.3}%");
            value1 = format!("{value1:0>7}");

            let k = format!("{key0:7}{:5}{key1}", "");
            let s = format!("{k}\n{value0}{:5}{value1}\n", "");

            format.push_str(s.as_str());
        }

        write!(f, "{format}")
    }
}

impl Display for Stat<BType>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        let mut format = "Bigrams:\n".to_string();

        self.stats.iter().for_each(|(key, value)| {
            let key = format!("{key:?}");
            let value = format!("{value:.3}%");

            let s = format!("{key:7}{:5}{value:0>7}\n", "");

            format.push_str(s.as_str());
        });

        write!(f, "{format}")
    }
}

impl Display for Stat<TType>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        let mut format = "Trigram:\n".to_string();

        self.stats.iter().for_each(|(key, value)| {
            let key = format!("{key:?}");
            let value = format!("{value:.3}%");

            let s = format!("{key:7}{:5}{value:0>7}\n", "");

            format.push_str(s.as_str());
        });

        write!(f, "{format}")
    }
}

impl Display for Stat<DType>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        let mut format = "Disjoint:\n".to_string();

        self.stats.iter().for_each(|(key, value)| {
            let key = format!("{key:?}");
            let value = format!("{value:.3}%");

            let s = format!("{key:7}{:5}{value:0>7}\n", "");

            format.push_str(s.as_str());
        });

        write!(f, "{format}")
    }
}

impl Display for Stat<S1Type>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        let mut format = "Skip1:\n".to_string();

        self.stats.iter().for_each(|(key, value)| {
            let key = format!("{key:?}");
            let value = format!("{value:.3}%");

            let s = format!("{key:7}{:5}{value:0>7}\n", "");

            format.push_str(s.as_str());
        });

        write!(f, "{format}")
    }
}

impl Display for Stat<S2Type>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        let mut format = "Skip2:\n".to_string();

        self.stats.iter().for_each(|(key, value)| {
            let key = format!("{key:?}");
            let value = format!("{value:.3}%");

            let s = format!("{key:7}{:5}{value:0>7}\n", "");

            format.push_str(s.as_str());
        });

        write!(f, "{format}")
    }
}

impl Display for Stat<S3Type>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        let mut format = "Skip3:\n".to_string();

        self.stats.iter().for_each(|(key, value)| {
            let key = format!("{key:?}");
            let value = format!("{value:.3}%");

            let s = format!("{key:7}{:5}{value:0>7}\n", "");

            format.push_str(s.as_str());
        });

        write!(f, "{format}")
    }
}
