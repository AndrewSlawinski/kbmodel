use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use itertools::Itertools;
use serde::Deserialize;
use serde_json;

use crate::stat::trigram::Trigram;
use crate::utility::converter::Converter;

pub type CharacterData = Vec<f64>;
pub type BigramData = Vec<f64>;
pub type TrigramData = HashMap<Trigram<char>, f64>;

#[derive(Deserialize)]
struct LanguageDataIntermediate {
    pub language: String,
    pub characters: HashMap<char, f64>,
    pub bigrams: HashMap<String, f64>,
    pub skipgrams: HashMap<String, f64>,
    pub skipgrams2: HashMap<String, f64>,
    pub skipgrams3: HashMap<String, f64>,
    pub trigrams: HashMap<String, f64>,
}

pub struct LanguageData {
    pub language: String,
    pub characters: CharacterData,
    pub bigrams: BigramData,
    pub skipgrams: BigramData,
    pub skipgrams2: BigramData,
    pub skipgrams3: BigramData,
    pub trigrams: TrigramData,
    pub converter: Converter,
}

impl LanguageData {
    pub fn new(language: &str) -> LanguageData {
        let file_path = format!("static/language_data/{}.json", language.to_lowercase());
        let mut file = File::open(file_path).unwrap();

        let mut contents = String::new();

        file.read_to_string(&mut contents).unwrap();

        let data: LanguageDataIntermediate = serde_json::from_str(contents.as_str()).unwrap();
        let res = LanguageData::from(data);

        return res;
    }
}

impl From<LanguageDataIntermediate> for LanguageData {
    fn from(mut data: LanguageDataIntermediate) -> Self {
        let mut convert_u8 = Converter::default();

        for c in ['\'', ',', '.', ';', '/', '~'] {
            data.characters.entry(c).or_insert(0.0);
        }

        let characters = get_char_data(&data.characters, &mut convert_u8);

        let bigrams = get_bigram_data(&data.bigrams, &mut convert_u8);
        let skipgrams = get_bigram_data(&data.skipgrams, &mut convert_u8);
        let skipgrams2 = get_bigram_data(&data.skipgrams2, &mut convert_u8);
        let skipgrams3 = get_bigram_data(&data.skipgrams3, &mut convert_u8);

        let trigrams = get_trigram_data(&data.trigrams, &mut convert_u8);

        return Self {
            characters,
            bigrams,
            skipgrams,
            skipgrams2,
            skipgrams3,
            trigrams,
            language: data.language,
            converter: convert_u8,
        };
    }
}

fn get_char_data(data: &HashMap<char, f64>, converter: &mut Converter) -> CharacterData {
    let mut res = CharacterData::new();

    for (c, f) in data.into_iter() {
        converter.insert_single(c.clone());
        res.push(f.clone());
    }

    return res;
}

fn get_bigram_data(data: &HashMap<String, f64>, converter: &mut Converter) -> BigramData {
    return (0..converter.len()).cartesian_product(0..converter.len())
        // vec![(0, '0'), (0, '1'), (1, '0'), (1, '1')]
        .map(|(c1, c2)| converter.as_string(&[c1, c2])).map(|bigram| data.get(&bigram).cloned().unwrap_or(0.0)).collect::<BigramData>();
}

fn get_trigram_data(data: &HashMap<String, f64>, converter: &mut Converter) -> TrigramData {
    let mut res = TrigramData::new();

    for (trigram, freq) in data {
        let trigram_vec = trigram.chars().collect::<Vec<char>>();
        let tv_u8 = converter.to(trigram_vec);

        if tv_u8[0] != tv_u8[1] && tv_u8[1] != tv_u8[2]
        {
            let new_trigram = Trigram::<u8>(tv_u8[0], tv_u8[1], tv_u8[2]);

            res.insert(new_trigram, freq.clone());
        }
    }

    return res;
}
