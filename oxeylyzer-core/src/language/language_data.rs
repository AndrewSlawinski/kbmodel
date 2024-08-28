use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct LanguageDataIntermediate
{
    pub language: String,
    pub characters: HashMap<char, f64>,
    pub bigrams: HashMap<String, f64>,
    pub skipgrams: HashMap<String, f64>,
    pub skipgrams2: HashMap<String, f64>,
    pub skipgrams3: HashMap<String, f64>,
    pub trigrams: HashMap<String, f64>,
}

#[derive(Clone)]
pub struct LanguageData
{
    pub language: String,
    pub characters: Vec<f64>,
    pub bigrams: Vec<f64>,
    pub skip_grams: Vec<f64>,
    pub skip2_grams: Vec<f64>,
    pub skip3_grams: Vec<f64>,
    pub trigrams: Vec<f64>,
}
