use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use glob::glob;
use itertools::Itertools;
use serde::Deserialize;

use crate::translation::*;

#[derive(Deserialize, Default)]
struct Multiple
{
    #[serde(default)]
    uppercase_versions: bool,
    list: Vec<[String; 2]>,
}

#[derive(Deserialize, Default)]
struct Bijection(String, String);

impl std::ops::Add for Bijection
{
    type Output = Bijection;

    fn add(mut self, rhs: Self) -> Self::Output
    {
        self.0.push_str(&rhs.0);
        self.1.push_str(&rhs.1);

        return self;
    }
}

#[derive(Deserialize)]
struct CorpusConfigLoad
{
    #[serde(default)]
    inherits: Vec<String>,
    #[serde(default)]
    letters_to_lowercase: String,
    #[serde(default)]
    keep: String,
    #[serde(default)]
    multiple: Multiple,
    #[serde(default)]
    one_to_one: Bijection,
    #[serde(default)]
    punct_unshifted: Bijection,
}

impl CorpusConfigLoad
{
    fn check_for_language(language: &str) -> Result<PathBuf, String>
    {
        let try_find_path = glob("static/corpus_configs/*/*.toml")
            .unwrap()
            .flatten()
            .find(|stem| {
                return language == stem.file_stem().unwrap_or_else(|| std::ffi::OsStr::new(""));
            });

        return match try_find_path
        {
            | Some(path) =>
            {
                let res = path
                    .as_path()
                    .parent()
                    .unwrap()
                    .components()
                    .last()
                    .unwrap()
                    .as_os_str();

                Ok(PathBuf::from(res))
            },
            | None => Err("Could not find a fitting config".to_string()),
        };
    }

    pub fn new(language: &str, preferred_folder: Option<&str>) -> Result<Self, String>
    {
        let preferred_folder: Result<PathBuf, String> = match preferred_folder
        {
            | Some(folder) => Ok(PathBuf::from(folder)),
            | None => Self::check_for_language(language),
        };

        return match preferred_folder
        {
            | Ok(preferred_folder) =>
            {
                let file_name = format!("{language}.toml");

                let path = PathBuf::from("../../static")
                    .join("corpus_configs")
                    .join(preferred_folder)
                    .join(file_name.clone());

                let mut f = File::open(path).map_err(|_| {
                    format!(
                        "Couldn't open {} because it does not exist.",
                        file_name.clone()
                    )
                    .to_string()
                })?;

                let mut buf = String::new();

                f.read_to_string(&mut buf)
                    .map_err(|_| "Toml contains non-utf8 characters, aborting...".to_string())?;

                toml::from_str(buf.as_str()).map_err(|_| {
                    "Toml contains invalid elements. Check the readme for what is allowed."
                        .to_string()
                })
            },
            | Err(..) => Err("No config file found!".to_string()),
        };
    }
}

pub struct CorpusConfig
{
    inherits: Vec<String>,
    letters_to_lowercase: String,
    punct_unshifted: Bijection,
    keep: String,
    to_multiple: Vec<(char, String)>,
    one_to_one: Bijection,
}

impl CorpusConfig
{
    pub fn new(language: &str, preferred_folder: Option<&str>) -> Result<Self, String>
    {
        let loaded = CorpusConfigLoad::new(language, preferred_folder)?;

        return Ok(Self {
            inherits: loaded.inherits,
            letters_to_lowercase: loaded.letters_to_lowercase,
            punct_unshifted: loaded.punct_unshifted,
            keep: loaded.keep,
            to_multiple: Self::get_to_multiple(loaded.multiple),
            one_to_one: loaded.one_to_one,
        });
    }

    fn get_to_multiple(multiple: Multiple) -> Vec<(char, String)>
    {
        let mut res = Vec::new();

        if multiple.uppercase_versions
        {
            for [from, to] in multiple.list
            {
                if from.chars().count() == 1
                {
                    let c = from.chars().next().unwrap();

                    res.push((c, to.clone()));

                    let mut upper = c.to_uppercase();

                    if upper.clone().count() == 1
                    {
                        let upper_c = upper.next().unwrap();

                        res.push((upper_c, to));
                    }
                }
            }
        }

        return res;
    }

    pub fn all() -> Vec<(String, Self)>
    {
        return glob("static/text/*")
            .unwrap()
            .flatten()
            .filter(|pb| {
                return pb.is_dir();
            })
            .flat_map(|pb| {
                return pb.file_name().unwrap().to_os_string().into_string();
            })
            .map(|l| {
                return (l.clone(), Self::new(&l, None));
            })
            .filter_map(|(l, c)| {
                return match c
                {
                    | Ok(c) => Some((l, c)),
                    | Err(..) => None,
                };
            })
            .collect_vec();
    }

    pub fn new_translator(language: &str, preferred_folder: Option<&str>) -> Translator
    {
        return match Self::new(language, preferred_folder)
        {
            | Ok(config) => config.translator(),
            | Err(error) =>
            {
                println!("{error}\nUsing a raw translator instead.");

                Translator::raw(true)
            },
        };
    }

    pub fn translator(self) -> Translator
    {
        let mut res = Translator::new()
            .letters_to_lowercase(&self.letters_to_lowercase)
            .keep(&self.keep)
            .one_to_one(&self.one_to_one.1, &self.one_to_one.0)
            .custom_unshift(&self.punct_unshifted.1, &self.punct_unshifted.0)
            .as_multiple_string(&self.to_multiple)
            .build();

        for inherits in self.inherits
        {
            match Self::new(&inherits, None)
            {
                | Ok(new) =>
                {
                    res = res + new.translator();
                },
                | Err(..) =>
                {},
            }
        }

        return res;
    }
}
