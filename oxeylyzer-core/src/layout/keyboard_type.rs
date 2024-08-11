use serde::Deserialize;

#[derive(Deserialize, Clone, Default)]
#[serde(from = "String")]
#[serde(rename_all(deserialize = "PascalCase"))]
pub enum KeyboardType {
    #[default]
    AnsiAngle,
    IsoAngle,
    RowstagDefault,
    Ortho,
    Colstag,
}

impl From<String> for KeyboardType {
    fn from(value: String) -> Self {
        let lower = value.to_lowercase();
        let split = lower.split_whitespace().collect::<Vec<&str>>();

        return match split.len() {
            | 1 => {
                match split[0] {
                    | "ortho" => Self::Ortho,
                    | "colstag" => Self::Colstag,
                    | "rowstag" | "iso" | "ansi" | "jis" => Self::RowstagDefault,
                    | _ => Self::default(),
                }
            }
            | 2 => {
                match (split[0], split[1]) {
                    | ("ansi", "angle") => Self::AnsiAngle,
                    | ("iso", "angle") => Self::IsoAngle,
                    | _ => Self::default(),
                }
            }
            | _ => panic!("Couldn't match keyboard type."),
        };
    }
}
