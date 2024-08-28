use crate::language::text_ngrams::TextNgrams;
use crate::translation::Translator;
use indexmap::IndexMap;
use serde::{
    Deserialize,
    Serialize,
};
use smartstring::{
    LazyCompact,
    SmartString,
};
use std::path::PathBuf;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct TextData
{
    language: String,
    characters: IndexMap<char, f64>,

    bigrams: IndexMap<SmartString<LazyCompact>, f64>,
    skipgrams: IndexMap<SmartString<LazyCompact>, f64>,
    skipgrams2: IndexMap<SmartString<LazyCompact>, f64>,
    skipgrams3: IndexMap<SmartString<LazyCompact>, f64>,
    trigrams: IndexMap<SmartString<LazyCompact>, f64>,

    #[serde(skip)]
    char_sum: f64,
    #[serde(skip)]
    bigram_sum: f64,
    #[serde(skip)]
    skipgram_sum: f64,
    #[serde(skip)]
    skipgram2_sum: f64,
    #[serde(skip)]
    skipgram3_sum: f64,
    #[serde(skip)]
    trigram_sum: f64,
}

impl std::fmt::Display for TextData
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(
            f,
            "{{\
                \"language\": {},\
                \"characters\": {:#?},\
                \"bigrams\": {:#?},\
                \"skipgrams\": {:#?},\
                \"skipgrams2\": {:#?},\
                \"skipgrams3\": {:#?},\
                \"trigrams\": {:#?}\
            }}",
            self.language,
            self.characters,
            self.bigrams,
            self.skipgrams,
            self.skipgrams2,
            self.skipgrams3,
            self.trigrams
        )
    }
}

impl TextData
{
    pub fn new(language: &str) -> Self
    {
        return Self {
            language: language.replace(' ', "_").to_lowercase().to_string(),
            ..Default::default()
        };
    }

    fn add_n_subsequent(&mut self, n: usize, ngram: &str, freq: f64)
    {
        let mut chars = ngram.chars();
        let c1 = chars.next().unwrap();

        if n > 0 && c1 != ' '
        {
            self.add_character(c1, freq);

            // take first, first 2 etc chars of the trigram every time for the appropriate stat
            // as long as they don't contain spaces. return `c2` so I don't iter.next() too much

            let c2 = match chars.next()
            {
                | Some(c2) if n > 1 && c2 != ' ' =>
                {
                    self.add_bigram([c1, c2], freq);
                    c2
                },
                | _ => ' ',
            };

            // c1 and c3 for skipgrams
            let c3 = chars.next().unwrap();

            if n > 2 && c3 != ' '
            {
                self.add_skipgram([c1, c3], freq);

                if c2 != ' '
                {
                    self.add_trigram([c1, c2, c3], freq);
                }

                let c4 = chars.next().unwrap();

                if n > 3 && c4 != ' '
                {
                    self.add_skipgram2([c1, c4], freq);

                    let c5 = chars.next().unwrap();

                    if n > 4 && c5 != ' '
                    {
                        self.add_skipgram3([c1, c5], freq);
                    }
                }
            }
        }
    }

    fn add_character(&mut self, c: char, freq: f64)
    {
        self.characters
            .entry(c)
            .and_modify(|e| *e += freq)
            .or_insert(freq);

        self.char_sum += freq;
    }

    fn add_bigram(&mut self, bigram: [char; 2], freq: f64)
    {
        self.bigrams
            .entry(SmartString::from_iter(bigram))
            .and_modify(|e| *e += freq)
            .or_insert(freq);

        self.bigram_sum += freq;
    }

    fn add_skipgram(&mut self, skipgram: [char; 2], freq: f64)
    {
        self.skipgrams
            .entry(SmartString::from_iter(skipgram))
            .and_modify(|e| *e += freq)
            .or_insert(freq);

        self.skipgram_sum += freq;
    }

    fn add_skipgram2(&mut self, skipgram: [char; 2], freq: f64)
    {
        self.skipgrams2
            .entry(SmartString::from_iter(skipgram))
            .and_modify(|e| *e += freq)
            .or_insert(freq);

        self.skipgram2_sum += freq;
    }

    fn add_skipgram3(&mut self, skipgram: [char; 2], freq: f64)
    {
        self.skipgrams3
            .entry(SmartString::from_iter(skipgram))
            .and_modify(|e| *e += freq)
            .or_insert(freq);

        self.skipgram3_sum += freq;
    }

    fn add_trigram(&mut self, trigram: [char; 3], freq: f64)
    {
        self.trigrams
            .entry(SmartString::from_iter(trigram))
            .and_modify(|e| *e += freq)
            .or_insert(freq);

        self.trigram_sum += freq;
    }

    pub fn save(&self, pass: &bool)
    {
        use std::fs::OpenOptions;
        use std::io::Write;

        let buf = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"\t");
        let mut serializer = serde_json::Serializer::with_formatter(buf, formatter);

        self.serialize(&mut serializer).unwrap();

        let data_dir_str = if *pass
        {
            "static/language_data_raw"
        }
        else
        {
            "static/language_data"
        };

        let data_dir = &PathBuf::from(data_dir_str);

        if data_dir.exists()
        {
            println!("Why create it if it already exists?")
            // std::fs::create_dir_all(data_dir).unwrap();
        }

        let file_name = format!("{}/{}.json", data_dir.to_str().unwrap(), self.language);

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(file_name);

        file.unwrap()
            .write_all(serializer.into_inner().as_slice())
            .unwrap();
    }
}

impl<'a> From<(&TextNgrams<'a, 5>, &str, &Translator)> for TextData
{
    fn from((ngrams, language, translator): (&TextNgrams<5>, &str, &Translator)) -> Self
    {
        let mut res = TextData::new(language);

        for (ngram, freq) in ngrams.ngrams.iter()
        {
            let first = ngram.chars().next().unwrap();

            if first != ' '
            {
                let first_t = translator.table.get(&first).unwrap();

                if first_t != " "
                {
                    let mut trans = translator.translate(ngram);
                    let count = trans.chars().count();

                    match count
                    {
                        | 5 .. =>
                        {
                            trans.push(' ');

                            let first_t_len = count.max(1);

                            let it1 = 0 .. count.min(first_t_len);
                            // trans.char_indices().map(|(i, _)| i).take(first_t_len);

                            let it2 = 5 .. count.min(5 + first_t_len);
                            // trans
                            // .char_indices()
                            // .map(|(i, _)| i)
                            // .skip(5)
                            // .take(first_t_len);

                            it1.zip(it2)
                                .map(|(i1, i2)| &trans[i1 .. i2])
                                .for_each(|ngram| res.add_n_subsequent(5, ngram, *freq as f64));
                        },
                        | 4 | 3 | 2 | 1 => res.add_n_subsequent(count, &trans, *freq as f64),

                        | _ =>
                        {},
                    }
                }
            }
        }

        res.characters
            .iter_mut()
            .for_each(|(_, f)| *f /= res.char_sum);

        res.bigrams
            .iter_mut()
            .for_each(|(_, f)| *f /= res.bigram_sum);

        res.skipgrams
            .iter_mut()
            .for_each(|(_, f)| *f /= res.skipgram_sum);

        res.skipgrams2
            .iter_mut()
            .for_each(|(_, f)| *f /= res.skipgram2_sum);

        res.skipgrams3
            .iter_mut()
            .for_each(|(_, f)| *f /= res.skipgram3_sum);

        res.trigrams
            .iter_mut()
            .for_each(|(_, f)| *f /= res.trigram_sum);

        res.characters
            .sort_by(|_, f1, _, f2| f2.partial_cmp(f1).unwrap());

        res.bigrams
            .sort_by(|_, f1, _, f2| f2.partial_cmp(f1).unwrap());

        res.skipgrams
            .sort_by(|_, f1, _, f2| f2.partial_cmp(f1).unwrap());

        res.skipgrams2
            .sort_by(|_, f1, _, f2| f2.partial_cmp(f1).unwrap());

        res.skipgrams3
            .sort_by(|_, f1, _, f2| f2.partial_cmp(f1).unwrap());

        res.trigrams
            .sort_by(|_, f1, _, f2| f2.partial_cmp(f1).unwrap());

        return res;
    }
}
