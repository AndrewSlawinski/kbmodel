use crate::language_data::LanguageData;
use crate::stats::bigram_stats::BType::*;
use crate::stats::predicates::Predicates;
use crate::type_def::Fixed;
use indexmap::IndexMap;
pub use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{
    Display,
    Formatter,
};
use std::ops::Index;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum BType
{
    SFB,
    LSB,

    IRB,
    ORB,
    LHIRB,
    LHORB,
    RHIRB,
    RHORB,

    URB,
    DRB,
    LHURB,
    LHDRB,
    RHURB,
    RHDRB,

    AB,
    RepB,
    S,
}

impl BType
{
    #[inline]
    pub fn f(&self) -> fn(a: &[u8]) -> bool
    {
        return match self
        {
            | SFB => Predicates::is_sf,
            | LSB => Predicates::is_ls,
            | IRB => Predicates::is_inroll,
            | ORB => Predicates::is_outroll,
            | AB => Predicates::is_alternate,
            | RepB => Predicates::all_equal,
            | S => Predicates::is_scissor,
            | LHIRB => Predicates::is_lh_inroll,
            | LHORB => Predicates::is_lh_outroll,
            | RHIRB => Predicates::is_rh_inroll,
            | RHORB => Predicates::is_rh_outroll,
            | URB => Predicates::is_uproll,
            | DRB => Predicates::is_downroll,
            | LHURB => Predicates::is_lh_uproll,
            | LHDRB => Predicates::is_lh_downroll,
            | RHURB => Predicates::is_rh_uproll,
            | RHDRB => Predicates::is_rh_downroll,
        };
    }

    pub const fn default() -> [BType; 17]
    {
        return [
            SFB, LSB, IRB, ORB, AB, RepB, S, LHIRB, LHORB, RHIRB, RHORB, URB, DRB, LHURB, LHDRB,
            RHURB, RHDRB,
        ];
    }

    pub const fn source(language_data: &LanguageData) -> &HashMap<String, f32>
    {
        return &language_data.bigrams;
    }
}

#[derive(Default, Clone)]
pub struct BStats
{
    pub inner: IndexMap<BType, f32>,
}

impl Index<BType> for BStats
{
    type Output = f32;

    fn index(&self, index: BType) -> &Self::Output
    {
        return &self.inner[&index];
    }
}

impl BStats
{
    #[inline]
    pub async fn new(chars: &Fixed<char>, language_data: &LanguageData, a: Option<&[BType]>)
    -> Self
    {
        let mut stats = IndexMap::new();

        Self::p2(
            chars,
            BType::source(language_data),
            &mut stats,
            a.unwrap_or(&BType::default()),
        );

        return Self { inner: stats };
    }

    fn p2(
        chars: &Fixed<char>,
        data: &HashMap<String, f32>,
        map: &mut IndexMap<BType, f32>,
        a: &[BType],
    )
    {
        for t in a
        {
            map.insert(*t, 0.);
        }

        for i in 0 .. 30
        {
            let c0 = chars[i];

            if char::is_ascii_punctuation(&c0)
            {
                continue;
            }

            for j in 0 .. 30
            {
                let c1 = chars[j];

                if char::is_ascii_punctuation(&c1)
                {
                    continue;
                }

                for (key, value) in map.iter_mut()
                {
                    if key.f()(&mut [i as u8, j as u8])
                    {
                        let p = data.get(&format!("{}{}", c0, c1)).unwrap_or(&0.0);

                        *value += p;
                    }
                }
            }
        }

        map.values_mut().for_each(|x| *x *= 100.);
    }

    pub fn p_par(chars: &Fixed<char>, data: &HashMap<String, f32>, f: fn(&[u8]) -> bool) -> f32
    {
        use rayon::iter::*;

        let a = (0 .. 30).into_par_iter().map(|i| {
            let c0 = chars[i as usize];

            if char::is_ascii_punctuation(&c0)
            {
                return 0.;
            }

            let b = (0 .. 30).into_par_iter().map(|j| {
                let c1 = chars[j as usize];

                if char::is_ascii_punctuation(&c1)
                {
                    return 0.;
                }

                if f(&[i, j])
                {
                    let p = data.get(&format!("{c0}{c1}")).unwrap_or(&0.);

                    return *p;
                }

                return 0.;
            });

            return b.collect::<Vec<f32>>().iter().sum();
        });

        let q: f32 = a.collect::<Vec<f32>>().iter().sum();

        return q * 100.;
    }
}

impl Display for BStats
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        let mut format = "Bigrams:\n".to_string();

        self.inner.iter().for_each(|(key, value)| {
            let key = format!("{:?}", key);
            let value = format!("{:.3}%", value);

            let s = format!("{key:7}{:5}{value:0>7}\n", "");

            format.push_str(s.as_str());
        });

        write!(f, "{}", format.clone())
    }
}
