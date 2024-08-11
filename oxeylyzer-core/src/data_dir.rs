use crate::language::language_data::LanguageData;
use crate::layout::layout::FastLayout;
use crate::layout::layout_generation::Layouts;
use crate::utility::converter::Converter;
use fs::read_to_string;
use itertools::Itertools;
use std::fs;
use std::fs::{
    read_dir,
    DirEntry,
    ReadDir,
};
use std::path::PathBuf;

pub fn files_in(dirs: Vec<&str>) -> ReadDir {
    let mut path_buf: PathBuf = PathBuf::from("static");

    dirs.iter().for_each(|d| path_buf.push(d));

    match read_dir(path_buf.as_path()) {
        | Ok(f) => f,
        | Err(e) => {
            panic!("{e}")
        }
    }
}

pub fn load_layouts_in_language(language_data: &mut LanguageData) -> Layouts {
    let mut layouts: Layouts = Vec::new();

    let paths: Vec<&str> = Vec::from(["layouts", language_data.language.as_str()]);

    for entry in files_in(paths).flatten().into_iter() {
        if entry.path().extension().unwrap() == "kb"
        {
            continue;
        }

        layouts.push(parse_layout(&entry, &mut language_data.converter));
    }

    return layouts;
}

fn format_layout_str(layout_str: &str) -> Vec<char> {
    let str = layout_str.to_string().chars().filter(|x| !x.is_whitespace()).collect_vec();

    assert_eq!(str.len(), 30);

    return str;
}

fn get_layout_name(entry: &DirEntry) -> String {
    return entry.path().file_stem().unwrap().to_str().unwrap().to_string();
}

fn parse_layout(layout_path: &DirEntry, converter: &mut Converter) -> FastLayout {
    let content = read_to_string(layout_path.path()).unwrap();

    let layout_chars = format_layout_str(&content);
    let layout_bytes = converter.to(layout_chars);

    return FastLayout::try_from(layout_bytes.as_slice()).unwrap();
}
