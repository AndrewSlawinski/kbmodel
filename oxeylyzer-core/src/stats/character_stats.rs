use crate::language_data::LanguageData;
use crate::stats::character_stats::CType::*;
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
pub enum CType
{
    LP,
    LR,
    LM,
    LI,
    RP,
    RR,
    RM,
    RI,
}

impl CType
{
    #[inline]
    pub const fn f(&self) -> fn(a: &u8) -> bool
    {
        return match self
        {
            | LP => |a| a % 10 == 0,
            | LR => |a| a % 10 == 1,
            | LM => |a| a % 10 == 2,
            | LI => |a| a % 10 == 3 || a % 10 == 4,
            | RP => |a| a % 10 == 9,
            | RR => |a| a % 10 == 8,
            | RM => |a| a % 10 == 7,
            | RI => |a| a % 10 == 6 || a % 10 == 5,
        };
    }

    pub const fn default() -> [CType; 8]
    {
        return [LP, LR, LM, LI, RI, RM, RR, RP];
    }

    pub const fn source(language_data: &LanguageData) -> &HashMap<char, f32>
    {
        return &language_data.characters;
    }
}

#[derive(Default, Clone)]
pub struct CStats
{
    pub inner: IndexMap<CType, f32>,
}

impl Index<CType> for CStats
{
    type Output = f32;

    fn index(&self, index: CType) -> &Self::Output
    {
        return &self.inner[&index];
    }
}

impl CStats
{
    #[inline]
    pub async fn new(chars: &Fixed<char>, language_data: &LanguageData, a: Option<&[CType]>)
    -> Self
    {
        let mut stats = IndexMap::new();

        Self::p2(
            chars,
            CType::source(language_data),
            &mut stats,
            a.unwrap_or(&CType::default()),
        );

        return Self { inner: stats };
    }

    pub(crate) fn p2(
        chars: &Fixed<char>,
        data: &HashMap<char, f32>,
        map: &mut IndexMap<CType, f32>,
        a: &[CType],
    )
    {
        for t in a
        {
            map.insert(*t, 0.);
        }

        for i in 0 .. 30
        {
            for (key, value) in map.iter_mut()
            {
                let c0 = chars[i as usize];

                if char::is_ascii_punctuation(&c0)
                {
                    continue;
                }

                if key.f()(&i)
                {
                    let p = data.get(&c0).unwrap_or(&0.0);

                    *value += *p;
                }
            }
        }

        map.values_mut().for_each(|x| *x *= 100.);
    }
}

impl Display for CStats
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        let mut format = "Columns:\n".to_string();

        for i in 0 .. self.inner.len() / 2
        {
            let (key0, value0) = self.inner.get_index(i).unwrap();
            let (key1, value1) = self.inner.get_index(self.inner.len() - (i + 1)).unwrap();

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

        write!(f, "{}", format.clone())
    }
}
