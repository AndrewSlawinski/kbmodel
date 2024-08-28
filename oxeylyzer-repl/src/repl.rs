use crate::flags::{
    Analyze,
    Compare,
    Language,
    Load,
    Ngram,
    Sfbs,
};

use itertools::Itertools;
use oxeylyzer_core::config::config::Config;
use oxeylyzer_core::corpus_transposition::CorpusConfig;
use oxeylyzer_core::data_dir::DataFetch;
use oxeylyzer_core::language::language_data::LanguageData;
use oxeylyzer_core::layout::layout::Layout;
use oxeylyzer_core::stats::layout_stats::LayoutStats;
use oxeylyzer_core::translation::Translator;
use oxeylyzer_core::type_def::Fixed;
use oxeylyzer_core::utility::converter::Converter;
use oxeylyzer_core::utility::pair::Pair;
use std::collections::HashMap;
use std::fs::ReadDir;
use std::io::{
    Read,
    Write,
};

pub struct Repl
{
    config: Config,
    language_data: LanguageData,

    converter: Converter,

    layouts: HashMap<String, Layout>,
}

impl Repl
{
    pub fn new() -> Self
    {
        let config = Config::default();

        let mut converter = Converter::default();
        let language_data = Self::load_language(&config.info.language, &mut converter);

        let fetch = DataFetch::layout_files_in_language(config.info.language.as_str());
        let layouts = Self::load_layouts(&mut converter, fetch);

        return Self {
            layouts,
            config,
            language_data,
            converter,
        };
    }

    fn load_layouts(converter: &mut Converter, fetch: ReadDir) -> HashMap<String, Layout>
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

            layouts.insert(name, converter.parse_layout(&string.unwrap().as_str()));
        }

        return layouts;
    }

    fn load_language(language: &str, converter: &mut Converter) -> LanguageData
    {
        let mut file = DataFetch::language_data_file(language);
        let mut contents = String::new();

        file.read_to_string(&mut contents).unwrap();

        let language_data = converter.language_data(contents.as_str());

        return language_data;
    }

    pub fn run(&mut self)
    {
        loop
        {
            let line = Repl::readline().unwrap();
            let line = line.trim();

            if line.is_empty()
            {
                continue;
            }

            match self.respond(line)
            {
                | Ok(true) =>
                {
                    println!("Exiting analyzer...");

                    break;
                },
                | Ok(false) => continue,
                | Err(err) =>
                {
                    println!("{err}");
                },
            }
        }
    }

    fn respond(&mut self, line: &str) -> Result<bool, String>
    {
        use crate::flags::Repl;
        use crate::flags::ReplCmd::*;

        let args = shlex::split(line)
            .ok_or("Invalid quotations")?
            .into_iter()
            .map(std::ffi::OsString::from)
            .collect_vec();

        let flags = Repl::from_vec(args).map_err(|e| e.to_string())?;

        let response: String = match flags.subcommand
        {
            | Analyze(o) => self.analyze(o),
            | Compare(o) => self.compare(o),
            | Rank(_) => self.rank(),
            | Sfbs(o) => self.sfbs(o),
            | Language(o) => self.language(o),
            | Languages(_) => Self::languages(),
            | Load(o) => self.load(o),
            | Ngram(o) => self.ngram(o),
            | Reload(_) =>
            {
                self.reload();

                "Model has been reloaded.".to_string()
            },
            | Quit(_) =>
            {
                return Ok(true);
            },
        };

        println!("{response}");

        return Ok(false);
    }

    fn load(&mut self, o: Load) -> String
    {
        return match (o.all, o.raw)
        {
            | (true, true) => "You can't currently generate all corpora as raw".to_string(),
            | (true, false) =>
            {
                let mut r = "".to_string();

                for (language, config) in CorpusConfig::all()
                {
                    r.push_str(format!("loading data for language: {language}...").as_str());

                    DataFetch::load_data(language.as_str(), &config.translator());
                }

                r
            },
            | (false, true) =>
            {
                DataFetch::load_data(o.language.to_str().unwrap(), &Translator::raw(true));

                format!("loading raw data for language: {}...", o.language.display())
            },
            | (false, false) =>
            {
                let language = o
                    .language
                    .to_str()
                    .ok_or_else(|| format!("Language is invalid utf8: {:?}", o.language))
                    .unwrap();

                let translator = CorpusConfig::new_translator(language, None);

                println!("loading data for {}...", &language);

                DataFetch::load_data(language, &translator);

                if translator.is_raw
                {
                    String::new()
                }
                else
                {
                    self.config.info.language = language.to_string();
                    self.language_data = Self::load_language(language, &mut self.converter);

                    let files = DataFetch::layout_files_in_language(language);
                    self.layouts = Self::load_layouts(&mut self.converter, files);

                    format!("Set language to {language}. Sfr: {:.2}%", self.sfr_freq())
                }
            },
        };
    }

    fn language(&mut self, language: Language) -> String
    {
        let language = language.language.expect("Language not found.");

        let language = language
            .to_str()
            .expect(format!("Language is invalid utf8: {:?}", language).as_str());

        self.language_data = Self::load_language(language, &mut self.converter);
        self.config.info.language = language.to_string();

        return format!("Set language to {language}. Sfr: {:.2}%", self.sfr_freq());
    }

    fn reload(&mut self)
    {
        self.config = Config::default();

        self.language_data =
            Self::load_language(self.config.info.language.as_str(), &mut self.converter);

        let files = DataFetch::layout_files_in_language(&self.config.info.language);
        self.layouts = Self::load_layouts(&mut self.converter, files)
    }

    fn languages() -> String
    {
        let mut r = String::new();
        let files = DataFetch::files_in(vec!["language_data"]);

        files.for_each(|p| {
            let name = p
                .unwrap()
                .file_name()
                .to_string_lossy()
                .replace('_', " ")
                .replace(".json", "");

            if name != "test"
            {
                r.push_str(format!("{name}\n").as_str());
            }
        });

        return r;
    }

    fn compare(&mut self, o: Compare) -> String
    {
        let name0 = o.name1;
        let name1 = o.name2;

        let mut result = format!("\n{name0:31}{name1}");

        let l0 = self.layout_by_name(name0.as_str());
        let l1 = self.layout_by_name(name1.as_str());

        let heatmap0 = Self::heatmap(&self.language_data.characters, &l0.matrix, &self.converter);
        let heatmap1 = Self::heatmap(&self.language_data.characters, &l1.matrix, &self.converter);

        let compare_map = heatmap0
            .into_iter()
            .zip(heatmap1)
            .map(move |(mut a, b)| {
                a.push_str(b.as_str());

                return a;
            })
            .collect_vec()
            .join("\n");

        let s0 = LayoutStats::new(&self.language_data, l0);
        let s1 = LayoutStats::new(&self.language_data, l1);

        result.push_str(compare_map.as_str());
        result.push_str(format!("{}", s0.bigram_stats).as_str());
        result.push_str(format!("{}", s1.bigram_stats).as_str());

        return result;
    }

    fn analyze(&mut self, o: Analyze) -> String
    {
        let name = o.name_or_number.as_str();
        let layout = self.layout_by_name(name);

        let stats = LayoutStats::new(&self.language_data, layout);

        let layout_str = Self::heatmap(
            &self.language_data.characters,
            &layout.matrix,
            &self.converter,
        )
        .join("\n");

        return format!(
            "{layout_str}\n\n{}\nScore: {:.3}",
            stats.bigram_stats,
            stats.bigram_stats.total_score()
        );
    }

    pub fn rank(&self) -> String
    {
        todo!()
    }

    pub fn layout_by_name(&self, name: &str) -> &Layout
    {
        return self
            .layouts
            .get(name)
            .expect(format!("'{name}' does not exist!").as_str());
    }

    pub fn sfr_freq(&self) -> f64
    {
        let len = self.language_data.characters.len();
        let chars = 0 .. len;

        return chars
            .clone()
            .cartesian_product(chars)
            .filter(|(i1, i2)| i1 == i2)
            .map(|(c1, c2)| {
                self.language_data
                    .bigrams
                    .get(c1 * len + c2)
                    .unwrap_or(&0.0)
            })
            .sum();
    }

    fn sfbs(&self, o: Sfbs) -> String
    {
        let layout = self.layout_by_name(o.name.as_str());

        let top_n = o.count.unwrap_or(10).min(48);

        let mut response = format!("top {top_n} sfbs for {}:", o.name);

        let len = self.language_data.characters.len();

        for i in 0 .. len
        {
            for j in 0 .. len
            {
                if Pair(i, j).is_sfb()
                {
                    let c0 = layout.matrix[i];
                    let c1 = layout.matrix[j];

                    let bigram = self.converter.as_string(&[c0, c1]);
                    let freq = self.language_data.bigrams[i * len + j];

                    response.push_str(format!("{bigram}: {:.3}%\n", freq).as_str());
                }
            }
        }

        return response;
    }

    pub fn readline() -> std::io::Result<String>
    {
        let mut buf = String::new();

        write!(std::io::stdout(), "> ")?;

        std::io::stdout().flush()?;
        std::io::stdin().read_line(&mut buf)?;

        return Ok(buf);
    }

    pub fn ngram(&mut self, ngram: Ngram) -> String
    {
        let ngram = ngram.ngram;

        return match ngram.chars().count()
        {
            | 1 =>
            {
                let c = ngram.chars().next().unwrap();
                let i = self.converter.char_to_vec_index(c) as usize;

                let occ = self.language_data.characters[i];

                format!("{ngram}: {occ:.3}%")
            },
            | 2 =>
            {
                let bigram: [char; 2] = ngram.chars().collect_vec().try_into().unwrap();

                let indices = self.converter.ngram_to_indices(&bigram);

                let b0 = indices[0] * self.language_data.characters.len() as u8 + indices[1];
                let b1 = indices[1] * self.language_data.characters.len() as u8 + indices[0];

                let rev = bigram.into_iter().rev().collect::<String>();

                let occ_b1 = self.language_data.bigrams.get(b0 as usize).unwrap_or(&0.0);
                let occ_b2 = self.language_data.bigrams.get(b1 as usize).unwrap_or(&0.0);

                let occ_s = self
                    .language_data
                    .skip_grams
                    .get(b0 as usize)
                    .unwrap_or(&0.0);
                let occ_s2 = self
                    .language_data
                    .skip2_grams
                    .get(b1 as usize)
                    .unwrap_or(&0.0);

                format!(
                    "{ngram} + {rev}: {:.3}%,\n\t{ngram}: {occ_b1:.3}%\n\t{rev}: {occ_b2:.3}%\n\
                {ngram} + {rev} (skipgram): {:.3}%,\n\t{ngram}: {occ_s:.3}%\n\t{rev}: {occ_s2:.3}%",
                    occ_b1 + occ_b2,
                    occ_s + occ_s2
                )
            },
            | 3 =>
            {
                todo!();

                // let trigram: Trigram<char> = ngram.chars().collect_vec().try_into().unwrap();
                //
                // let trigram: NGram<u8, 3> = Trigram();
                //
                // let &(_, occ) = self
                //     .language_data
                //     .trigrams
                //     .iter()
                //     .find(|&&(tf, _)| tf == t)
                //     .unwrap_or(&(t, 0.0));
                //
                // format!("{ngram}: {:.3}%", occ)
            },
            | _ =>
            {
                // Skill issue honestly...
                "Invalid ngram! It must be 1, 2 or 3 chars long. ".to_string()
            },
        };
    }

    pub fn heat(c: char, p: f64) -> String
    {
        use ansi_rgb::{
            rgb,
            Colorable,
        };

        let complement = u8::MAX as f64 * p;
        let complement = u8::MAX ^ complement as u8;

        let heat = rgb(192, complement, complement);

        let formatted = c.to_string().fg(heat);

        return format!("{formatted}");
    }

    pub fn heatmap(data: &Vec<f64>, char_indices: &Fixed<u8>, converter: &Converter)
    -> Vec<String>
    {
        let mut map = Vec::new();
        let mut print_str = String::new();

        for (i, c) in char_indices.iter().enumerate()
        {
            if i % 10 == 0 && i != 0
            {
                map.push(print_str.clone());

                print_str = String::new();
            }

            if (i + 5) % 10 == 0
            {
                print_str.push(' ');
            }

            let p = *data.get(*c as usize).unwrap_or(&0.0);
            let c = converter.index_char(*c);

            let heat = Self::heat(c, p);

            print_str.push_str(heat.as_str());
            print_str.push(' ');
        }

        map.push(print_str.clone());

        return map;
    }
}
