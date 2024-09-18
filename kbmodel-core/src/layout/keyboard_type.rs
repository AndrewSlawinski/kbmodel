use crate::layout::keyboard_type::KeyboardType::{
    AnsiAngle,
    Colstag,
    IsoAngle,
    Ortho,
    RowstagDefault,
};
use crate::type_def::Fixed;
use itertools::Itertools;
use serde::Deserialize;

#[derive(Deserialize, Clone, Default)]
#[serde(from = "String")]
#[serde(rename_all(deserialize = "PascalCase"))]
pub enum KeyboardType
{
    #[default]
    AnsiAngle,
    IsoAngle,
    RowstagDefault,
    Ortho,
    Colstag,
}

impl KeyboardType
{
    pub const fn get_effort_map(&self) -> Fixed<f32>
    {
        return match self
        {
            | IsoAngle =>
            {
                [
                    3.0, 2.4, 2.0, 2.2, 2.4, 3.3, 2.2, 2.0, 2.4, 3.0, 1.8, 1.3, 1.1, 1.0, 2.6, 2.6,
                    1.0, 1.1, 1.3, 1.8, 3.3, 2.8, 2.4, 1.8, 2.2, 2.2, 1.8, 2.4, 2.8, 3.3,
                ]
            },
            | AnsiAngle =>
            {
                [
                    3.0, 2.4, 2.0, 2.2, 2.4, 3.3, 2.2, 2.0, 2.4, 3.0, 1.8, 1.3, 1.1, 1.0, 2.6, 2.6,
                    1.0, 1.1, 1.3, 1.8, 3.7, 2.8, 2.4, 1.8, 2.2, 2.2, 1.8, 2.4, 2.8, 3.3,
                ]
            },
            | RowstagDefault =>
            {
                [
                    3.0, 2.4, 2.0, 2.2, 2.4, 3.3, 2.2, 2.0, 2.4, 3.0, 1.8, 1.3, 1.1, 1.0, 2.6, 2.6,
                    1.0, 1.1, 1.3, 1.8, 3.5, 3.0, 2.7, 2.3, 3.7, 2.2, 1.8, 2.4, 2.8, 3.3,
                ]
            },
            | Ortho =>
            {
                [
                    3.0, 2.4, 2.0, 2.2, 3.1, 3.1, 2.2, 2.0, 2.4, 3.0, 1.7, 1.3, 1.1, 1.0, 2.6, 2.6,
                    1.0, 1.1, 1.3, 1.7, 3.2, 2.6, 2.3, 1.6, 3.0, 3.0, 1.6, 2.3, 2.6, 3.2,
                ]
            },
            | Colstag =>
            {
                [
                    3.0, 2.4, 2.0, 2.2, 3.1, 3.1, 2.2, 2.0, 2.4, 3.0, 1.7, 1.3, 1.1, 1.0, 2.6, 2.6,
                    1.0, 1.1, 1.3, 1.7, 3.4, 2.6, 2.2, 1.8, 3.2, 3.2, 1.8, 2.2, 2.6, 3.4,
                ]
            },
        };
    }
}

impl From<String> for KeyboardType
{
    fn from(value: String) -> Self
    {
        let lower = value.to_lowercase();
        let split = lower.split_whitespace().collect_vec();

        return match split.len()
        {
            | 1 =>
            {
                match split[0]
                {
                    | "ortho" => Ortho,
                    | "colstag" => Colstag,
                    | "rowstag" | "iso" | "ansi" | "jis" => RowstagDefault,
                    | _ => Self::default(),
                }
            },
            | 2 =>
            {
                match (split[0], split[1])
                {
                    | ("ansi", "angle") => AnsiAngle,
                    | ("iso", "angle") => IsoAngle,
                    | _ => Self::default(),
                }
            },
            | _ => panic!("Couldn't match keyboard type."),
        };
    }
}
