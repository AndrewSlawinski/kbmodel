use crate::new_stats::alt_stats::DataSet::{
    Bigram,
    Character,
    Disjoint,
    Skip1,
    Skip2,
    Skip3,
};
use crate::new_stats::stat_matrices::StatMatrices;
use crate::new_stats::stat_type::StatType;
use core::fmt::{
    Display,
    Formatter,
};
use indexmap::IndexMap;
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

        let stat = StatType::default();
        let columns = StatType::columns();

        for d in DataSet::default()
        {
            let mut f = IndexMap::new();

            if d == Character
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
                | Character => k.f()(&stat_matrices.char_map, v),
                | Bigram => k.f()(&stat_matrices.bigram_map, v),
                | Skip1 => k.f()(&stat_matrices.skip1_map, v),
                | Skip2 => k.f()(&stat_matrices.skip2_map, v),
                | Skip3 => k.f()(&stat_matrices.skip2_map, v),
                | Disjoint => k.f()(&stat_matrices.disjoint_map, v),
            }
        }
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
    Character,
    Bigram,
    Skip1,
    Skip2,
    Skip3,
    Disjoint,
}

impl DataSet
{
    pub const fn default() -> [DataSet; 6]
    {
        return [Character, Bigram, Disjoint, Skip1, Skip2, Skip3];
    }

    pub const fn f(&self) -> fn(data: &[f32], indexmap: &mut IndexMap<StatType, f32>)
    {
        return match self
        {
            | Character => Self::character,
            | Bigram => Self::bigram,
            | Disjoint => Self::bigram,
            | Skip1 => Self::bigram,
            | Skip2 => Self::bigram,
            | Skip3 => Self::bigram,
        };
    }

    fn character(data: &[f32], index_map: &mut IndexMap<StatType, f32>)
    {
        for i in 0 .. 30
        {
            for (k, v) in index_map.iter_mut()
            {
                if k.f()(&[i as u8])
                {
                    *v += data[i];
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
                    if k.f()(&[i as u8, j as u8])
                    {
                        let l = i * 30 + j;

                        *v += data[l];
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

        write!(f, "{}", format.clone())
    }
}
