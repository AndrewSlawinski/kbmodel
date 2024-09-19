use crate::stats::alt_stats::DataSet::{
    Bigram,
    Column,
    Disjoint,
    Skip1,
    Skip2,
    Skip3,
    Trigram,
};
use crate::stats::stat_matrices::StatMatrices;
use crate::stats::stat_type::StatType;
use core::fmt::{
    Display,
    Formatter,
};
use indexmap::IndexMap;
use itertools::Itertools;
use std::fmt;
use std::ops::Index;

pub struct AltStats
{
    map: IndexMap<DataSet, IndexMap<StatType, f32>>,
}

impl AltStats
{
    pub fn new() -> Self
    {
        let mut map = IndexMap::new();

        let stat = StatType::bigram();
        let columns = StatType::character();

        for d in DataSet::default()
        {
            let mut f = IndexMap::new();

            if d == Column
            {
                for s in columns
                {
                    f.insert(s, 0.0);
                }
            }
            else
            {
                for s in stat
                {
                    f.insert(s, 0.0);
                }
            }
            map.insert(d, f);
        }

        return Self { map };
    }

    pub async fn compute(&mut self, stat_matrices: &StatMatrices)
    {
        for (k, v) in self.map.iter_mut()
        {
            match k
            {
                | Column => k.f()(&stat_matrices.char_map, v),
                | Bigram => k.f()(&stat_matrices.bigram_map, v),
                | Skip1 => k.f()(&stat_matrices.skip1_map, v),
                | Skip2 => k.f()(&stat_matrices.skip2_map, v),
                | Skip3 => k.f()(&stat_matrices.skip2_map, v),
                | Disjoint => k.f()(&stat_matrices.disjoint_map, v),
                | Trigram => k.f()(&stat_matrices.trigram_map, v),
            }
        }
    }

    pub fn table_format(&self) -> String
    {
        let mut format = String::new();

        format.push_str(&format!("\t{:17}Left:{:6}Right:{:5}Total:\n", "", "", ""));

        self.map.iter().for_each(|(k0, v0)| {
            format.push_str(&format!("{:?}:\n", k0));

            for (a, b) in v0.iter().tuples()
            {
                let k1 = format!("{:?}", a.0);

                let av = format!("{:.3}%", a.1);
                let bv = format!("{:.3}%", b.1);
                let tv = format!("{:.3}%", a.1 + b.1);

                let s = format!(
                    "\t{:14}{:3}{av: >7}{:4}{bv: >7}{:4}{tv: >7}\n",
                    &k1[1 ..],
                    "",
                    "",
                    "",
                );

                format.push_str(s.as_str());
            }

            format.push_str("\n");
        });

        return format[0 .. (format.len() - 1)].to_string();
    }
}

impl Index<&DataSet> for AltStats
{
    type Output = IndexMap<StatType, f32>;

    fn index(&self, index: &DataSet) -> &Self::Output
    {
        return &self.map[index];
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum DataSet
{
    Column,
    Bigram,
    Skip1,
    Skip2,
    Skip3,
    Disjoint,
    Trigram,
}

impl DataSet
{
    pub const fn default() -> [DataSet; 7]
    {
        return [Column, Bigram, Disjoint, Skip1, Skip2, Skip3, Trigram];
    }

    pub const fn f(&self) -> fn(data: &[f32], indexmap: &mut IndexMap<StatType, f32>)
    {
        return match self
        {
            | Column => Self::character,
            | Bigram => Self::bigram,
            | Disjoint => Self::bigram,
            | Skip1 => Self::bigram,
            | Skip2 => Self::bigram,
            | Skip3 => Self::bigram,
            | Trigram => Self::trigram,
        };
    }

    fn character(data: &[f32], index_map: &mut IndexMap<StatType, f32>)
    {
        for i in 0 .. 30
        {
            for (k, v) in index_map.iter_mut()
            {
                if k.f()(&[i])
                {
                    *v += data[i as usize];
                }
            }
        }
    }

    fn bigram(data: &[f32], index_map: &mut IndexMap<StatType, f32>)
    {
        for i in 0 .. 30
        {
            for j in 0 .. 30
            {
                for (k, v) in index_map.iter_mut()
                {
                    if k.f()(&[i, j])
                    {
                        *v += data[i as usize * 30 + j as usize];
                    }
                }
            }
        }
    }

    fn trigram(data: &[f32], index_map: &mut IndexMap<StatType, f32>)
    {
        for i in 0 .. 30
        {
            for j in 0 .. 30
            {
                for l in 0 .. 30
                {
                    for (k, v) in index_map.iter_mut()
                    {
                        if k.f()(&[i, j, l])
                        {
                            *v += data[i as usize * 300 + j as usize * 30 + l as usize];
                        }
                    }
                }
            }
        }
    }
}

impl Display for AltStats
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        let mut format = String::new();

        self.map.iter().for_each(|(k0, v0)| {
            format.push_str(&format!("{:?}:\n", k0));

            v0.iter().for_each(|(k1, v1)| {
                let k1 = format!("{:?}", k1);
                let v1 = format!("{:.3}%", v1);

                let s = format!("{k1:16}{:5}{v1:0>7}\n", "");

                format.push_str(s.as_str());
            });

            format.push_str("\n");
        });

        write!(f, "{}", format)
    }
}
