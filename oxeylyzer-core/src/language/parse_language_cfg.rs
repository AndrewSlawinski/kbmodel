use std::collections::HashMap;
use std::io::Read;

pub struct LangsChars {
    pub languages: Vec<String>,
    pub chars: String,
}

impl LangsChars {
    pub fn new(languages: Vec<String>, chars: String) -> Self {
        return Self { languages, chars };
    }
}

pub fn chars_in_language_default() -> HashMap<String, String> {
    let mut cfg = HashMap::new();

    match std::fs::File::open("languages_default.cfg") {
        | Ok(mut f) => {
            let mut file_contents = String::new();

            f.read_to_string(&mut file_contents).unwrap();

            parse_lines(&mut cfg, &mut file_contents);
        }
        | Err(_) => {
            println!("No 'languages_default.cfg' file found in the root folder.");
        }
    }

    return cfg;
}

fn parse_lines(cfg: &mut HashMap<String, String>, file_contents: &mut String) {
    for line in file_contents.lines() {
        match parse_line(line) {
            | Ok(lang_chars) => {
                for lang in lang_chars.languages {
                    cfg.insert(lang, lang_chars.chars.clone());
                }
            }
            | Err(error_msg) => {
                println!("{error_msg}")
            }
        }
    }
}

fn parse_line(line: &str) -> Result<LangsChars, String> {
    let line_content = line.split('#').collect::<Vec<&str>>();
    let split_langs_chars = line_content[0].split(':').collect::<Vec<&str>>();

    if split_langs_chars.len() != 2
    {
        return Err("Either the characters or language is missing".to_owned());
    }

    let langs = split_langs_chars[0].trim().split(',').map(|s| s.trim().to_owned()).collect::<Vec<String>>();

    if langs.is_empty()
    {
        return Err("No specified language".to_owned());
    }

    let chars = split_langs_chars[1].trim();
    let char_count = chars.chars().count();

    if char_count != 30
    {
        return Err(format!(
            "{char_count} characters in {langs:?}, 30 required."
        ));
    }

    return Ok(LangsChars::new(langs, chars.to_string()));
}
