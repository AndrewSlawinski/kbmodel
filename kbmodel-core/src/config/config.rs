use crate::{
    config::finger_to_column::FingerToColumn,
    config::info::Info,
};
use serde::Deserialize;
use std::fs::File;
use std::io::Read;

#[derive(Deserialize, Clone)]
pub struct Config
{
    pub info: Info,
    pub finger_to_column: FingerToColumn,
}

impl Default for Config
{
    fn default() -> Self
    {
        return Self {
            info: Default::default(),
            finger_to_column: Default::default(),
        };
    }
}

impl Config
{
    pub fn new() -> Self
    {
        let mut f = File::open("../../../../config.toml").expect("config.toml is missing.");

        let mut buf = String::new();
        f.read_to_string(&mut buf).unwrap();

        let new: Self = toml::from_str(&buf).expect("Failed to parse config.toml.");

        return new;
    }
}

#[test]
fn test()
{
    Config::default();
}
