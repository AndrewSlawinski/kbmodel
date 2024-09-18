use crate::language_data::LanguageData;
use crate::layout::layout::Layout;
use crate::stats::bigram_stats::{
    BStats,
    BType,
};
use crate::stats::character_stats::{
    CStats,
    CType,
};
use crate::stats::disjoint_stats::{
    D1Stats,
    DType,
};
use crate::stats::skip_stats::{
    S1Stats,
    S1Type,
    S2Stats,
    S2Type,
    S3Stats,
    S3Type,
};
use crate::stats::stat::Stat;
use crate::stats::trigram_stats::{
    TStats,
    TType,
};
use futures::executor::block_on;
use futures::join;
use std::ops::Index;

#[derive(Default, Clone)]
pub struct LayoutStats
{
    pub character_stats: Stat<CType>,
    pub bigram_stats: Stat<BType>,
    pub trigram_stats: Stat<TType>,
    pub disjoint_stats: Stat<DType>,
    pub skip1_stats: Stat<S1Type>,
    pub skip2_stats: Stat<S2Type>,
    pub skip3_stats: Stat<S3Type>,
}

impl Index<&BType> for LayoutStats
{
    type Output = f32;

    fn index(&self, index: &BType) -> &Self::Output
    {
        return &self.bigram_stats[index];
    }
}

impl Index<&TType> for LayoutStats
{
    type Output = f32;

    fn index(&self, index: &TType) -> &Self::Output
    {
        return &self.trigram_stats[index];
    }
}

impl Index<&DType> for LayoutStats
{
    type Output = f32;

    fn index(&self, index: &DType) -> &Self::Output
    {
        return &self.disjoint_stats[index];
    }
}

impl LayoutStats
{
    #[inline]
    pub fn new(language_data: &LanguageData, layout: &Layout) -> Self
    {
        let j = block_on(Self::f_generic(language_data, layout));

        return Self {
            character_stats: j.0,
            bigram_stats: j.1,
            trigram_stats: j.2,
            disjoint_stats: j.3,
            skip1_stats: j.4,
            skip2_stats: j.5,
            skip3_stats: j.6,
        };
    }

    #[allow(unused)]
    async fn f(
        language_data: &LanguageData,
        layout: &Layout,
    ) -> (CStats, BStats, TStats, D1Stats, S1Stats, S2Stats, S3Stats)
    {
        let character_stats = CStats::new(&layout.matrix, language_data, None);
        let bigram_stats = BStats::new(&layout.matrix, language_data, None);
        let trigram_stats = TStats::new(&layout.matrix, language_data, None);
        let disjoint_stats = D1Stats::new(&layout.matrix, language_data, None);

        let skip1_stats = S1Stats::new(&layout.matrix, language_data, None);
        let skip2_stats = S2Stats::new(&layout.matrix, language_data, None);
        let skip3_stats = S3Stats::new(&layout.matrix, language_data, None);

        let j = join!(
            character_stats,
            bigram_stats,
            trigram_stats,
            disjoint_stats,
            skip1_stats,
            skip2_stats,
            skip3_stats
        );

        return j;
    }

    async fn f_generic(
        language_data: &LanguageData,
        layout: &Layout,
    ) -> (
        Stat<CType>,
        Stat<BType>,
        Stat<TType>,
        Stat<DType>,
        Stat<S1Type>,
        Stat<S2Type>,
        Stat<S3Type>,
    )
    {
        let character_stats = Stat::new_c(&layout.matrix, language_data, None);
        let bigram_stats = Stat::new_b(&layout.matrix, language_data, None);
        let trigram_stats = Stat::new_t(&layout.matrix, language_data, None);
        let disjoint_stats = Stat::new_d(&layout.matrix, language_data, None);

        let skip1_stats = Stat::new_s1(&layout.matrix, language_data, None);
        let skip2_stats = Stat::new_s2(&layout.matrix, language_data, None);
        let skip3_stats = Stat::new_s3(&layout.matrix, language_data, None);

        let j = join!(
            character_stats,
            bigram_stats,
            trigram_stats,
            disjoint_stats,
            skip1_stats,
            skip2_stats,
            skip3_stats
        );

        return j;
    }

    #[allow(unused)]
    pub async fn with(
        language_data: &LanguageData,
        layout: &Layout,
        c: Option<&[CType]>,
        b: Option<&[BType]>,
        t: Option<&[TType]>,
        d: Option<&[DType]>,
        s1: Option<&[S1Type]>,
        s2: Option<&[S2Type]>,
        s3: Option<&[S3Type]>,
    ) -> Self
    {
        todo!();

        // let character_stats = match c
        // {
        //     | None => CStats::default(),
        //     | Some(s) => CStats::new(&layout.matrix, language_data, Some(s)).await,
        // };
        //
        // let bigram_stats = match b
        // {
        //     | None => BStats::default(),
        //     | Some(s) => BStats::new(&layout.matrix, language_data, Some(s)).await,
        // };
        //
        // let trigram_stats = match t
        // {
        //     | None => TStats::default(),
        //     | Some(s) => TStats::new(&layout.matrix, language_data, Some(s)).await,
        // };
        //
        // let disjoint_stats = match d
        // {
        //     | None => D1Stats::default(),
        //     | Some(s) => D1Stats::new(&layout.matrix, language_data, Some(s)).await,
        // };
        //
        // let skip1_stats = match s1
        // {
        //     | None => S1Stats::default(),
        //     | Some(s) => S1Stats::new(&layout.matrix, language_data, Some(s)).await,
        // };
        //
        // let skip2_stats = match s2
        // {
        //     | None => S2Stats::default(),
        //     | Some(s) => S2Stats::new(&layout.matrix, language_data, Some(s)).await,
        // };
        //
        // let skip3_stats = match s3
        // {
        //     | None => S3Stats::default(),
        //     | Some(s) => S3Stats::new(&layout.matrix, language_data, Some(s)).await,
        // };

        // return Self {
        //     character_stats,
        //     bigram_stats,
        //     trigram_stats,
        //     disjoint_stats,
        //     skip1_stats,
        //     skip2_stats,
        //     skip3_stats,
        // };
    }
}

impl LayoutStats
{
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

    #[inline(always)]
    pub fn arithmatic_mean(a: &[f32]) -> f32
    {
        let v: f32 = a.iter().sum();

        return v / (a.len() as f32);
    }
}
