use crate::language::text_data::TextData;
use crate::language::text_n_grams::TextNgrams;
use crate::layout::layout::FastLayout;
use crate::translation::Translator;
use crate::type_def::Fixed;
use file_chunker::FileChunker;
use fs::read_to_string;
use itertools::Itertools;
use rayon::iter::{
    IntoParallelRefIterator,
    ParallelBridge,
    ParallelIterator,
};
use smartstring::{
    LazyCompact,
    SmartString,
};
use std::collections::HashMap;
use std::fs;
use std::fs::{
    read_dir,
    DirEntry,
    File,
    ReadDir,
};
use std::io::Read;
use std::path::PathBuf;
use std::time::Instant;

const ROOT: &str = "static";
pub struct DataFetch {}

impl DataFetch {
    pub fn files_in(dirs: Vec<&str>) -> ReadDir {
        let mut path_buf: PathBuf = PathBuf::from(ROOT);

        dirs.iter().for_each(|d| path_buf.push(d));

        match read_dir(path_buf.as_path()) {
            | Ok(f) => f,
            | Err(e) => {
                panic!("{e}")
            }
        }
    }

    pub fn load_layouts_in_language(language_name: &str) -> Vec<FastLayout> {
        let mut layouts: Vec<FastLayout> = Vec::new();

        let paths: Vec<&str> = Vec::from(["layouts", language_name]);

        for entry in Self::files_in(paths).flatten().into_iter() {
            if entry.path().extension().unwrap() == "kb"
            {
                continue;
            }

            layouts.push(Self::parse_layout(&entry));
        }

        return layouts;
    }

    pub fn get_language_data_file(language_name: &str) -> File {
        let file_path = format!(
            "{}/language_data/{}.json",
            ROOT,
            language_name.to_lowercase()
        );

        return File::open(file_path).unwrap();
    }

    fn format_layout_str(layout_str: &str) -> Vec<char> {
        let str = layout_str.to_string().chars().filter(|x| !x.is_whitespace()).collect_vec();

        assert_eq!(str.len(), 30);

        return str;
    }

    fn get_layout_name(entry: &DirEntry) -> String {
        return entry.path().file_stem().unwrap().to_str().unwrap().to_string();
    }

    fn parse_layout(layout_path: &DirEntry) -> FastLayout {
        let content = read_to_string(layout_path.path()).unwrap();

        let layout_chars = Self::format_layout_str(&content);
        // let layout_bytes = converter.to(layout_chars);

        return FastLayout::try_from(layout_chars.as_slice()).unwrap();
    }

    pub fn chars_in_languages_default() -> HashMap<String, Fixed<char>> {
        let mut f = File::open("languages_default.cfg").expect("No 'languages_default.cfg' file found in the root folder.");

        let mut file_contents = String::new();
        f.read_to_string(&mut file_contents).unwrap();

        return Self::parse_lines(&mut file_contents);
    }

    fn parse_lines(file_contents: &mut String) -> HashMap<String, Fixed<char>> {
        let mut parsed = HashMap::new();

        for line in file_contents.lines() {
            let c = line.chars().next().unwrap();

            if c.is_whitespace() || c == '#'
            {
                continue;
            }

            Self::insert_from_line(&mut parsed, line);
        }

        return parsed.try_into().unwrap();
    }

    fn insert_from_line(hash_map: &mut HashMap<String, Fixed<char>>, line: &str) {
        let split = line.split(':').collect_vec();

        if split.len() != 2
        {
            panic!("Either the characters or language are missing");
        }

        let languages = split[0].split(',').map(|s| s.trim().to_owned()).collect_vec();

        if languages.is_empty()
        {
            panic!("No specified language");
        }

        let chars: Fixed<char> = split[1].trim().chars().collect_vec().try_into().unwrap();

        if chars.len() != 30
        {
            panic!("{} characters in {languages:?}, 30 required.", chars.len());
        }

        for language in languages {
            hash_map.entry(language).or_insert(chars.clone());
        }
    }

    pub fn load_data(language: &str, translator: &Translator) {
        let start_total = Instant::now();

        let files = DataFetch::files_in(vec!["text", language]);

        let files = files.par_bridge().flat_map(|path| File::open(path.unwrap().path())).map(|file| Self::chunk(&file)).collect::<Vec<_>>();

        let time = Instant::now();
        let mut now = (time - start_total).as_millis();

        println!("Prepared text files in {now}ms", );

        let strings = files.par_iter().flat_map(|(chunker, count)| chunker.chunks(*count, Some(' ')).unwrap()).map(|chunk| {
            std::str::from_utf8(chunk).expect(
                "one of the files provided is not encoded as utf-8.\
                Make sure all files in the directory are valid utf-8.",
            )
        }).map(|s| {
            let mut last_chars = SmartString::<LazyCompact>::new();
            let mut inter = [' '; 5];

            s.chars().rev().take(5).enumerate().for_each(|(i, c)| *inter.get_mut(4 - i).unwrap() = c);

            inter.into_iter().for_each(|c| last_chars.push(c));
            last_chars.push_str("     ");

            return (s, last_chars);
        }).collect::<Vec<_>>();

        now = (Instant::now() - time).as_millis();
        println!("Converted to UTF8 in {now}ms", );

        let quingrams = strings.par_iter().map(|(s, last)| TextNgrams::new(s, last)).reduce(TextNgrams::default, |accum, new| accum.combine_with(new));

        let text_data = TextData::from((&quingrams, language, translator));

        text_data.save(&translator.is_raw);

        println!(
            "loading {language} took {}ms",
            (Instant::now() - start_total).as_millis()
        );
    }

    fn chunk(file: &File) -> (FileChunker, usize) {
        let len = file.metadata().unwrap().len() + 1;
        let count = (len / (1024 * 1024 * 4)).max(1);

        return (FileChunker::new(&file).unwrap(), count as usize);
    }
}
