use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct LanguageData
{
    pub language: String,
    pub characters: HashMap<char, f32>,
    pub bigrams: HashMap<String, f32>,
    pub skipgrams: HashMap<String, f32>,
    pub skipgrams2: HashMap<String, f32>,
    pub skipgrams3: HashMap<String, f32>,
    pub trigrams: HashMap<String, f32>,
}
