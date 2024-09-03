use crate::type_def::Fixed;
use itertools::Itertools;

use crate::layout::layout::Layout;
use std::collections::HashMap;
use std::fs::{
    File,
    ReadDir,
};
use std::io::Read;
use std::path::PathBuf;

const ROOT: &str = "static";
pub struct DataFetch {}

impl DataFetch
{
    pub fn files_in(dirs: Vec<&str>) -> ReadDir
    {
        let mut path_buf: PathBuf = PathBuf::from(ROOT);

        dirs.iter().for_each(|d| path_buf.push(d));

        match std::fs::read_dir(path_buf.as_path())
        {
            | Ok(f) => f,
            | Err(e) =>
            {
                panic!("{e}")
            },
        }
    }

    pub fn layout_files_in_language(language: &str) -> ReadDir
    {
        let paths: Vec<&str> = Vec::from(["layouts", language]);

        return Self::files_in(paths);
    }

    pub fn load_layouts(fetch: ReadDir) -> HashMap<String, Layout>
    {
        use std::fs::read_to_string;

        let mut layouts = HashMap::new();

        for entry in fetch.flatten().into_iter()
        {
            if entry.path().extension().unwrap() != "kb"
            {
                continue;
            }

            let string = read_to_string(entry.path());

            let name = entry.file_name().to_str().unwrap().to_string();
            let name = name[.. name.len() - 3].to_string();

            layouts.insert(name, Self::parse_layout(&string.unwrap().as_str()));
        }

        return layouts;
    }

    pub fn parse_layout(string: &str) -> Layout
    {
        let mut layout = [' '; 30];

        string
            .to_string()
            .chars()
            .filter(|x| !x.is_whitespace())
            .into_iter()
            .enumerate()
            .for_each(|(i, c)| layout[i] = c);

        return Layout::from(layout);
    }

    pub fn language_data_file(language_name: &str) -> File
    {
        let file_path = format!(
            "{}/language_data/{}.json",
            ROOT,
            language_name.to_lowercase()
        );

        return File::open(file_path).unwrap();
    }

    pub fn chars_in_languages_default() -> HashMap<String, Fixed<char>>
    {
        let mut f = File::open("languages_default.cfg")
            .expect("No 'languages_default.cfg' file found in the root folder.");

        let mut file_contents = String::new();
        f.read_to_string(&mut file_contents).unwrap();

        return Self::parse_lines(&mut file_contents);
    }

    fn parse_lines(file_contents: &mut String) -> HashMap<String, Fixed<char>>
    {
        let mut parsed = HashMap::new();

        for line in file_contents.lines()
        {
            let c = line.chars().next().unwrap();

            if c.is_whitespace() || c == '#'
            {
                continue;
            }

            Self::insert_from_line(&mut parsed, line);
        }

        return parsed.try_into().unwrap();
    }

    fn insert_from_line(hash_map: &mut HashMap<String, Fixed<char>>, line: &str)
    {
        let split = line.split(':').collect_vec();

        if split.len() != 2
        {
            panic!("Either the characters or language are missing");
        }

        let languages = split[0]
            .split(',')
            .map(|s| s.trim().to_owned())
            .collect_vec();

        if languages.is_empty()
        {
            panic!("No specified language");
        }

        let chars: Fixed<char> = split[1].trim().chars().collect_vec().try_into().unwrap();

        if chars.len() != 30
        {
            panic!("{} characters in {languages:?}, 30 required.", chars.len());
        }

        for language in languages
        {
            hash_map.entry(language).or_insert(chars.clone());
        }
    }
}
