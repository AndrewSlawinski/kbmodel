use crate::{
    config::info::Info,
    config::pins::Pins,
    config::weights::weights::Weights,
};
use serde::Deserialize;
use std::fs::File;
use std::io::Read;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub info: Info,
    pub weights: Weights,
    pub pins: Pins,
}

impl Default for Config {
    fn default() -> Self {
        return Self {
            info: Default::default(),
            weights: Default::default(),
            pins: Default::default(),
        };
    }
}

impl Config {
    pub fn new() -> Self {
        let mut f = File::open("../../../../config.toml").expect("config.toml is missing.");

        let mut buf = String::new();
        f.read_to_string(&mut buf).unwrap();

        let new: Self = toml::from_str(&buf).expect("Failed to parse config.toml.");

        return new;
    }
}

#[test]
fn test() {
    Config::default();
}
