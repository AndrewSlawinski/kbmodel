use std::collections::HashMap;
use std::io::prelude::*;

use crate::data_dir::DataFetch;
use crate::n_gram::n_gram::NGram;
use itertools::Itertools;
use serde::Deserialize;
use serde_json;

pub type NGramDataMap<T, const N: usize> = HashMap<NGram<T, 3>, f64>;

#[derive(Deserialize)]
pub struct LanguageDataIntermediate {
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
    pub characters: HashMap<NGram<char, 1>, f64>,
    pub bigrams: HashMap<NGram<char, 2>, f64>,
    pub skip_grams: HashMap<NGram<char, 2>, f64>,
    pub skip2_grams: HashMap<NGram<char, 2>, f64>,
    pub skip3_grams: HashMap<NGram<char, 2>, f64>,
    pub trigrams: HashMap<NGram<char, 3>, f64>,
}

impl LanguageData {
    pub fn new(language_name: &str) -> LanguageData {
        let mut file = DataFetch::get_language_data_file(language_name);
        let mut contents = String::new();

        file.read_to_string(&mut contents).unwrap();

        let mut data: LanguageDataIntermediate = serde_json::from_str(contents.as_str()).unwrap();
        let res = LanguageData::from(&mut data);

        return res;
    }
}

impl From<&mut LanguageDataIntermediate> for LanguageData {
    fn from(language_data: &mut LanguageDataIntermediate) -> Self {
        // Old code pre-fork which does too much global mutation to be intelligable...
        {
            // let mut convert_u8 = Converter::default();

            // let characters = sus_converter_character_data(&data.characters);
            // let bigrams = sus_converter_get_bigram_data(&data.bigrams);
            // let skipgrams = sus_converter_get_bigram_data(&language_data.skipgrams);
            // let skipgrams2 = sus_converter_get_bigram_data(&language_data.skipgrams2);
            // let skipgrams3 = sus_converter_get_bigram_data(&language_data.skipgrams3);
            // let trigrams = sus_converter_get_bigram_data(&data.bigrams);
        }
        //

        for c in ['\'', ',', '.', ';', '/', '~'] {
            language_data.characters.entry(c).or_insert(0.0);
        }

        let mut a: [char; 1] = [' '; 1];

        let mut characters = HashMap::new();

        language_data.characters.iter().for_each(|(key, value)| {
            a.iter_mut().for_each(|c| *c = *key);

            characters.entry(NGram::from(&a)).or_insert(value.clone());
        });

        let mut a: [char; 2] = [' '; 2];

        let mut bigrams = HashMap::new();

        language_data.bigrams.iter().for_each(|(key, value)| {
            a.iter_mut().enumerate().for_each(|(i, c)| *c = key.chars().collect_vec()[i]);

            let t = NGram::from(&a);

            bigrams.entry(t).or_insert(value.clone());
        });

        let mut skip_grams = HashMap::new();

        language_data.skipgrams.iter().for_each(|(key, value)| {
            a.iter_mut().enumerate().for_each(|(i, c)| *c = key.chars().collect_vec()[i]);

            let t = NGram::from(&a);

            skip_grams.entry(t).or_insert(value.clone());
        });

        let mut skip2_grams = HashMap::new();

        language_data.skipgrams2.iter().for_each(|(key, value)| {
            a.iter_mut().enumerate().for_each(|(i, c)| *c = key.chars().collect_vec()[i]);

            let t = NGram::from(&a);

            skip2_grams.entry(t).or_insert(value.clone());
        });

        let mut skip3_grams = HashMap::new();

        language_data.skipgrams3.iter().for_each(|(key, value)| {
            a.iter_mut().enumerate().for_each(|(i, c)| *c = key.chars().collect_vec()[i]);

            let t = NGram::from(&a);

            skip3_grams.entry(t).or_insert(value.clone());
        });

        let mut a: [char; 3] = [' '; 3];

        let mut trigrams = HashMap::new();

        language_data.trigrams.iter().for_each(|(key, value)| {
            a.iter_mut().enumerate().for_each(|(i, c)| *c = key.chars().collect_vec()[i]);

            let t = NGram::from(&a);

            trigrams.entry(t).or_insert(value.clone());
        });

        return Self {
            characters,
            bigrams,
            skip_grams,
            skip2_grams,
            skip3_grams,
            trigrams,
            language: language_data.language.clone(),
            // converter: convert_u8,
        };
    }
}
