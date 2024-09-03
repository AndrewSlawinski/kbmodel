use crate::language_data::LanguageData;
use crate::layout::layout::Layout;
use crate::stats::bigram_stats::BType::{
    Repeat,
    AB,
    IRB,
    LSB,
    ORB,
    S,
    SFB,
};
use crate::stats::bigram_stats::{
    BType,
    BigramStats,
};
use crate::stats::disjoint_stats::DType::{
    D1Repeat,
    D1IRB,
    D1LSB,
    D1ORB,
    D1S,
    D1SFB,
};
use crate::stats::disjoint_stats::{
    D1Stats,
    DType,
};
use crate::stats::skip_stats::S1Type::{
    S1Repeat,
    S1IRB,
    S1LSB,
    S1ORB,
    S1S,
    S1SFB,
};
use crate::stats::skip_stats::S2Type::{
    S2Repeat,
    S2IRB,
    S2LSB,
    S2ORB,
    S2S,
    S2SFB,
};
use crate::stats::skip_stats::S3Type::{
    S3Repeat,
    S3IRB,
    S3LSB,
    S3ORB,
    S3S,
    S3SFB,
};
use crate::stats::skip_stats::{
    S1Stats,
    S1Type,
    S2Stats,
    S2Type,
    S3Stats,
    S3Type,
};
use crate::stats::trigram_stats::TType::{
    Redirect,
    AT,
    IRT,
    ORT,
    SFT,
};
use crate::stats::trigram_stats::{
    TStats,
    TType,
};
use itertools::Itertools;
use std::ops::Index;

#[derive(Default, Clone)]
pub struct LayoutStats
{
    pub bigram_stats: BigramStats,
    pub trigram_stats: TStats,
    pub disjoint_stats: D1Stats,
    pub skip1_stats: S1Stats,
    pub skip2_stats: S2Stats,
    pub skip3_stats: S3Stats,
}

impl Index<BType> for LayoutStats
{
    type Output = f32;

    fn index(&self, index: BType) -> &Self::Output
    {
        return &self.bigram_stats[index];
    }
}
impl Index<TType> for LayoutStats
{
    type Output = f32;

    fn index(&self, index: TType) -> &Self::Output
    {
        return &self.trigram_stats[index];
    }
}

impl Index<DType> for LayoutStats
{
    type Output = f32;

    fn index(&self, index: DType) -> &Self::Output
    {
        return &self.disjoint_stats[index];
    }
}

impl LayoutStats
{
    pub fn new(language_data: &LanguageData, layout: &Layout) -> Self
    {
        let b = [SFB, LSB, IRB, ORB, AB, Repeat, S];
        let t = [SFT, IRT, ORT, Redirect, AT];
        let d = [D1SFB, D1LSB, D1IRB, D1ORB, D1Repeat, D1S];
        let s1 = [S1SFB, S1LSB, S1IRB, S1ORB, S1Repeat, S1S];
        let s2 = [S2SFB, S2LSB, S2IRB, S2ORB, S2Repeat, S2S];
        let s3 = [S3SFB, S3LSB, S3IRB, S3ORB, S3Repeat, S3S];

        return Self {
            bigram_stats: BigramStats::new(language_data, &layout.matrix, &b),
            trigram_stats: TStats::new(language_data, &layout.matrix, &t),
            disjoint_stats: D1Stats::new(language_data, &layout.matrix, &d),
            skip1_stats: S1Stats::new(language_data, &layout.matrix, &s1),
            skip2_stats: S2Stats::new(language_data, &layout.matrix, &s2),
            skip3_stats: S3Stats::new(language_data, &layout.matrix, &s3),
        };
    }

    pub fn with(
        language_data: &LanguageData,
        layout: &Layout,
        b: Option<&[BType]>,
        t: Option<&[TType]>,
        d: Option<&[DType]>,
        s1: Option<&[S1Type]>,
        s2: Option<&[S2Type]>,
        s3: Option<&[S3Type]>,
    ) -> Self
    {
        let bigram_stats = match b
        {
            | None => BigramStats::default(),
            | Some(s) => BigramStats::new(language_data, &layout.matrix, s),
        };

        let trigram_stats = match t
        {
            | None => TStats::default(),
            | Some(s) => TStats::new(language_data, &layout.matrix, s),
        };

        let disjoint_stats = match d
        {
            | None => D1Stats::default(),
            | Some(s) => D1Stats::new(language_data, &layout.matrix, s),
        };

        let skip1_stats = match s1
        {
            | None => S1Stats::default(),
            | Some(s) => S1Stats::new(language_data, &layout.matrix, s),
        };

        let skip2_stats = match s2
        {
            | None => S2Stats::default(),
            | Some(s) => S2Stats::new(language_data, &layout.matrix, s),
        };

        let skip3_stats = match s3
        {
            | None => S3Stats::default(),
            | Some(s) => S3Stats::new(language_data, &layout.matrix, s),
        };

        return Self {
            bigram_stats,
            trigram_stats,
            disjoint_stats,
            skip1_stats,
            skip2_stats,
            skip3_stats,
        };
    }

    #[inline(always)]
    fn mod_all(a: &mut [u8])
    {
        let mut i = 0;

        while i < a.len()
        {
            a[i] %= 10;

            i += 1;
        }
    }

    #[inline(always)]
    const fn all_equal(a: &[u8]) -> bool
    {
        let mut i = 1;

        while i < a.len()
        {
            if a[0] != a[i]
            {
                return false;
            }

            i += 1;
        }

        return true;
    }

    pub fn is_repeat(a: &mut [u8]) -> bool
    {
        return a[0] == a[1];
    }

    #[inline(always)]
    const fn unique(a: &[u8]) -> bool
    {
        let mut x = 0;

        while x < a.len()
        {
            let mut j = 0;
            let mut y = 0;

            while y < a.len()
            {
                j += if a[x] == a[y] { 1 } else { 0 };
                y += 1;
            }

            if j > 1
            {
                return false;
            }

            x += 1;
        }

        return true;
    }

    #[inline]
    pub fn is_sf(a: &mut [u8]) -> bool
    {
        if !Self::unique(a)
        {
            return false;
        }

        Self::mod_all(a);

        if Self::all_equal(a)
        {
            return true;
        }

        return match a[0]
        {
            | 3 | 4 => a.iter().all(|x| *x == 3 || *x == 4),
            | 5 | 6 => a.iter().all(|x| *x == 5 || *x == 6),
            | _ => false,
        };
    }

    #[inline]
    pub fn is_scissor(a: &mut [u8]) -> bool
    {
        if Self::all_equal(a)
        {
            return false;
        }

        let p0 = a[0] % 10;
        let p1 = a[1] % 10;

        if p0 == p1
        {
            return false;
        }

        let diff = a[0].abs_diff(a[1]);

        if diff > 15 && diff < 25
        {
            let sum = p0 + p1;

            return (p0 <= 4 && p1 <= 4 && sum != 7) || (p0 >= 5 && p1 >= 5 && sum != 11);
        }

        return false;
    }

    #[inline]
    pub fn is_bad_scissor(a: &mut [u8]) -> bool
    {
        if Self::all_equal(a)
        {
            return false;
        }

        let p0 = a[0] % 10;
        let p1 = a[1] % 10;

        if p0 == p1
        {
            return false;
        }

        let diff = a[0].abs_diff(a[1]);

        if diff > 15 && diff < 25
        {
            let sum = p0 + p1;

            return (p0 <= 4 && p1 <= 4 && sum != 7) || (p0 >= 5 && p1 >= 5 && sum != 11);
        }

        return false;
    }

    #[inline]
    pub fn is_lsb(a: &mut [u8]) -> bool
    {
        if Self::all_equal(a)
        {
            return false;
        }

        let p0 = a[0] % 10;
        let p1 = a[1] % 10;

        if p0 == p1
        {
            return false;
        }

        let min = if p0 > p1 { p1 } else { p0 };
        let max = if p0 > p1 { p0 } else { p1 };

        return (max == 4 && (min <= 2)) || (min == 5 && (max >= 7));
    }

    #[inline]
    pub fn is_inroll(a: &mut [u8]) -> bool
    {
        if !Self::unique(a)
        {
            return false;
        }

        Self::mod_all(a);

        return a.windows(2).all(|x| x[0] <= x[1]);
    }

    #[inline]
    pub fn is_outroll(a: &mut [u8]) -> bool
    {
        if !Self::unique(a)
        {
            return false;
        }

        Self::mod_all(a);

        return a.windows(2).all(|x| x[0] >= x[1]);
    }

    #[inline]
    pub fn is_alternate(a: &mut [u8]) -> bool
    {
        Self::mod_all(a);

        return a.windows(2).all(|x| {
            match x[0]
            {
                | ..= 4 => x[1] >= 5,
                | _ => x[1] <= 4,
            }
        });
    }

    #[inline]
    pub fn is_redirect(a: &mut [u8]) -> bool
    {
        Self::mod_all(a);

        if !a.windows(2).all(|x| x[0] != x[1])
        {
            return false;
        }

        if a.iter().all(|x| *x <= 4) || a.iter().all(|x| *x >= 5)
        {
            return !a.windows(2).all(|x| x[0] > x[1]) && !a.windows(2).all(|x| x[0] < x[1]);
        }

        return false;
    }

    #[inline(always)]
    pub fn is_left_hand(i: &usize) -> bool
    {
        return (i / 5) % 2 == 0;
    }

    #[inline]
    pub fn geometric_mean(a: &[f32]) -> f32
    {
        let mut s = 1.;
        let mut len = a.len();

        a.iter().for_each(|x| {
            if *x == 0.
            {
                len -= 1;
            }
            else
            {
                s *= x;
            }
        });

        return s.powf((len as f32).recip());
    }

    #[inline]
    pub fn root_square_mean(a: &[f32]) -> f32
    {
        let mut s = 0.;
        let mut len = a.len();

        a.iter().for_each(|x| {
            if *x == 0.
            {
                len -= 1;
            }
            else
            {
                s += x.powf(2.);
            }
        });

        return ((len as f32).recip() * s).sqrt();
    }

    #[inline]
    pub fn arithmatic_mean(a: &[f32]) -> f32
    {
        let v: f32 = a.iter().sum();

        return v / (a.len() as f32);
    }
}

pub trait Stats
{
    fn total_score(&self) -> f32;
    fn arithmatic_mean(&self) -> f32;
    fn geometric_mean(&self) -> f32;
    fn root_square_mean(&self) -> f32;
}
