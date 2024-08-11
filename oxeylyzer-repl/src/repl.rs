use crate::flags::{
    Analyze,
    Compare,
    Generate,
    Improve,
    Save,
    Sfbs,
};
use indicatif::{
    ProgressBar,
    ProgressStyle,
};
use itertools::Itertools;
use oxeylyzer_core::config::config::Config;
use oxeylyzer_core::corpus_transposition::CorpusConfig;
use oxeylyzer_core::language::language_data::LanguageData;
use oxeylyzer_core::layout::layout::{
    FastLayout,
    Layout,
};
use oxeylyzer_core::layout::layout_generation::{
    LayoutGenerator,
    Layouts,
};
use oxeylyzer_core::load_text;
use oxeylyzer_core::rayon::iter::IntoParallelIterator;
use oxeylyzer_core::stat::trigram::Trigram;
use oxeylyzer_core::translation::Translator;
use oxeylyzer_core::utility::heat_map::{
    heatmap_heat,
    heatmap_string,
};
use std::ffi::OsString;
use std::fs::read_dir;
use std::io;
use std::io::Write;
use std::rc::Rc;

const ROWS: usize = 3;
const COLUMNS: usize = 10;

pub struct Repl {
    layout_generator: LayoutGenerator,
    layouts: Layouts,
    temp_generated: Layouts,
    config: Config,
    language_data: LanguageData,
}

impl Repl {
    pub fn new() -> Self {
        let config = Config::new();
        let language_data = LanguageData::new();

        let mut generator = LayoutGenerator::new(Rc::new(&config));

        return Self {
            layouts: generator.load_layouts(config.defaults.language.as_str()),
            layout_generator: generator,
            temp_generated: Vec::new(),
            config,
        };
    }

    pub unsafe fn run(&mut self) {
        loop {
            let line: &str = self.readline().map_err(|e| e.to_string()).unwrap().trim();

            if line.is_empty()
            {
                continue;
            }

            match self.respond(line) {
                | Ok(true) => {
                    println!("Exiting analyzer...");

                    break;
                },
                | Ok(false) => continue,
                | Err(err) => {
                    println!("{err}");
                },
            }
        }
    }

    unsafe fn respond(&mut self, line: &str) -> Result<bool, String> {
        use crate::flags::Repl;
        use crate::flags::ReplCmd::*;

        let args = shlex::split(line).ok_or("Invalid quotations")?.into_iter().map(std::ffi::OsString::from).collect::<Vec<OsString>>();

        let flags = Repl::from_vec(args).map_err(|e| e.to_string())?;

        let reponse: String = match flags.subcommand {
            | Analyze(o) => self.analyze(&o),
            | Compare(o) => self.compare(&o),
            | Rank(_) => self.rank(),
            | Generate(o) => self.generate(&o),
            | Improve(o) => self.improve(&o),
            | Save(o) => self.save(&o),
            | Sfbs(o) => self.sfbs(&o),
            | Language(o) => {
                match o.language {
                    | Some(l) => {
                        let config = Config::from(l);

                        let language = l.to_str().ok_or_else(|| format!("Language is invalid utf8: {:?}", l))?;

                        println!("{language:?}");

                        match LayoutGenerator::new(language, config) {
                            | Ok(generator) => {
                                self.layout_generator = generator;
                                self.layouts = self.layout_generator.load_layouts(language);
                                self.config.defaults.language = language.to_string();

                                println!(
                                    "Set language to {}. Sfr: {:.2}%",
                                    &language,
                                    self.sfr_freq() * 100.0
                                );
                            },
                            | Err(..) => {
                                return Err(format!("Could not load data for {}", language));
                            },
                        }
                    },
                    | None => println!("Current language: {}", self.config.defaults.language),
                }
            },
            | Include(o) => {
                self.layout_generator.load_layouts(&o.language).into_iter().for_each(|(name, layout)| {
                    self.layouts.insert(name, layout);
                });

                self.layouts.sort_by(|_, a, _, b| a.score.partial_cmp(&b.score).unwrap());
            },
            | Languages(_) => Self::languages(),
            | Load(o) => {
                match (o.all, o.raw) {
                    | (true, true) => {
                        return Err("You can't currently generate all corpora as raw".into());
                    },
                    | (true, _) => {
                        for (language, config) in CorpusConfig::all() {
                            println!("loading data for language: {language}...");

                            load_text::load_data(language.as_str(), config.translator());
                        }
                    },
                    | (false, true) => {
                        println!("loading raw data for language: {}...", o.language.display());

                        load_text::load_data(o.language, Translator::raw(true));
                    },
                    | (false, false) => {
                        let language = o.language.to_str().ok_or_else(|| format!("Language is invalid utf8: {:?}", o.language))?;

                        let translator = CorpusConfig::new_translator(language, None);
                        let is_raw_translator = translator.is_raw;

                        println!("loading data for {}...", &language);

                        load_text::load_data(language, translator).map_err(|e| e.to_string())?;

                        if !is_raw_translator
                        {
                            let config = Config::try_from();

                            match LayoutGenerator::new(language, config) {
                                | Ok(generator) => {
                                    self.config.defaults.language = language.into();
                                    self.layout_generator = generator;

                                    self.layouts = self.layout_generator.load_layouts(language);

                                    println!(
                                        "Set language to {}. Sfr: {:.2}%",
                                        language,
                                        self.sfr_freq() * 100.0
                                    );
                                },
                                | Err(e) => return Err(e.to_string()),
                            }
                        }
                    },
                }
            },
            | Ngram(o) => {
                println!("{}", get_ngram_info(&mut language_data, &o.ngram))
            },
            | Reload(_) => {
                let config = Config::try_from();

                self.config.pins.clone_from(&config.pins);

                match LayoutGenerator::new(&self.config.defaults.language, config) {
                    | Ok(generator) => {
                        self.layout_generator = generator;
                        self.layouts = self.layout_generator.load_layouts(&self.config.defaults.language);
                    },
                    | Err(e) => return Err(e.to_string()),
                }
            },
            | Quit(_) => {
                return Ok(true);
            },
        };

        println!("{response}");

        return Ok(false);
    }

    unsafe fn languages() -> String {
        let mut r = "";

        read_dir("static/language_data").unwrap().for_each(|p| {
            let name = p.unwrap().file_name().to_string_lossy().replace('_', " ").replace(".json", "");

            if name != "test"
            {
                r = concat!(r, name);
            }
        });

        r.to_string()
    }

    unsafe fn save(&mut self, o: &Save) -> String {
        match self.get_nth(o.nth) {
            | Some(layout) => {
                self.save_layout(layout.clone(), o.name.clone()).unwrap();

                "".to_string()
            },
            | None => {
                format!(
                    "Index '{}' provided is out of bounds for {} generated layouts",
                    o.nth,
                    self.temp_generated.len()
                )
            },
        }
    }

    unsafe fn improve(&mut self, o: &Improve) -> String {
        let layout = self.layout_by_name(&o.name);

        let layouts = self.layout_generator.generate_n_with_pins_iter(&layout, o.count).collect::<Vec<Layouts>>();

        // self.temp_generated =
        //     self.layout_generator
        //         .generate_n_with_pins(&layout, o.count, layout.clone())

        "".to_string()
    }

    fn compare(&mut self, o: &Compare) -> String {
        return self.compare_name(&o.name1, &o.name2).to_string();
    }

    fn analyze(&mut self, o: &Analyze) -> String {
        match o.name_or_number.parse::<usize>() {
            | Ok(number) => {
                match self.temp_generated.get(number) {
                    | Some(layout) => self.analyze_layout(layout).to_string(),
                    | None => {
                        format!(
                            "Index '{}' provided is out of bounds for {} generated layouts",
                            o.name_or_number,
                            self.temp_generated.len()
                        )
                    },
                }
            },
            | Err(_) => self.analyze_name(&o.name_or_number).unwrap().to_string(),
        }
    }

    pub fn rank(&self) -> String {
        let mut result: String = String::new();

        for layout in self.layouts.iter() {
            // result.push_str(
            //     format!("{:10}{}", format!("{:.3}:", layout.score), layout.name).as_str(),
            // );
        }

        return result;
    }

    pub fn layout_by_name(&self, name: &str) -> &FastLayout {
        return self.layouts.get(name).expect(format!("'{name}' does not exist!").as_str());
    }

    pub fn analyze_name(&self, name: &str) -> Result<String, String> {
        let layout = match self.layout_by_name(name) {
            | Some(l) => l,
            | None => {
                return Err(format!("layout {} does not exist!", name));
            },
        };

        println!("{}", name);

        return Ok(self.analyze_layout(layout));
    }

    fn placeholder_name(&self, layout: &FastLayout) -> Result<String, String> {
        for i in 1..1000 {
            let new_name_bytes = layout.matrix[10..14].to_vec();
            let mut new_name = self.language_data.converter.as_string(new_name_bytes.as_slice());

            new_name.push_str(format!("{}", i).as_str());

            if !self.layouts[i](&new_name)
            {
                return Ok(new_name);
            }
        }

        return Err("Could not find a good placeholder name for the layout.".to_string());
    }

    pub fn save_layout(&mut self, mut layout: &FastLayout, name: Option<String>) -> String {
        let new_name = match name {
            | Some(n) => n.replace(' ', "_"),
            | None => self.placeholder_name(&layout).unwrap(),
        };

        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true).open(format!(
            "static/layouts/{}/{}.kb",
            self.config.info.language, new_name
        )).map_err(|e| e.to_string()).unwrap();

        let layout_formatted = layout.formatted_string(&self.language_data.converter);

        println!("saved {new_name}\n{layout_formatted}");

        f.write_all(layout_formatted.as_bytes()).unwrap();

        // layout.score = self.layout_generator.score(&layout);

        self.layouts.push(layout.clone());

        // self.layouts
        //     .sort_by(|_, a, _, b| a.score.partial_cmp(&b.score).unwrap());

        return "".to_string();
    }

    pub fn analyze_layout(&self, layout: &FastLayout) -> String {
        let stats = self.layout_generator.get_layout_stats(layout);

        let score = if layout.score == 0.000
        {
            self.layout_generator.score(layout)
        } else {
            layout.score
        };

        let layout_str = heatmap_string(&self.layout_generator.data, layout);

        return format!("{}\n{}\nScore: {:.3}", layout_str, stats, score);
    }

    pub fn compare_name(&self, name0: &str, name1: &str) -> String {
        let mut result = String::from(format!("\n{:31}{}", name0, name1));

        let l0 = self.layout_by_name(name0);
        let l1 = self.layout_by_name(name1);

        for y in 0..ROWS {
            for (layout) in [l0, l1].into_iter().enumerate() {
                result.push_str("        ");

                for x in 0..COLUMNS {
                    if x == COLUMNS / 2
                    {
                        result.push(' ');
                    }

                    result.push_str(heatmap_heat(&language_data, layout.c(x + 10 * y)).as_str());
                }
            }

            result.push('\n');
        }

        let s0 = self.layout_generator.get_layout_stats(l0);
        let s1 = self.layout_generator.get_layout_stats(l1);

        let ts0 = s0.trigram_stats;
        let ts1 = s1.trigram_stats;

        // abomination
        result.push_str(
            format!(
                include_str!(COMPARE_NAMES_TEMPLATE),
                format!("{:.3}%", 100.0 * s0.same_finger_bigram),
                100.0 * s1.same_finger_bigram,
                format!("{:.3}%", 100.0 * s0.d_same_finger_bigram),
                100.0 * s1.d_same_finger_bigram,
                format!("{:.3}", 10.0 * s0.fspeed),
                10.0 * s1.fspeed,
                format!("{:.3}%", 100.0 * s0.scissors),
                100.0 * s1.scissors,
                format!("{:.3}%", 100.0 * s0.lsbs),
                100.0 * s1.lsbs,
                format!("{:.3}%", 100.0 * s0.pinky_ring),
                100.0 * s1.pinky_ring,
                format!("{:.2}%", 100.0 * ts0.inrolls),
                100.0 * ts1.inrolls,
                format!("{:.2}%", 100.0 * ts0.outrolls),
                100.0 * ts1.outrolls,
                format!("{:.2}%", 100.0 * (ts0.inrolls + ts0.outrolls)),
                100.0 * (ts1.inrolls + ts1.outrolls),
                format!("{:.3}%", 100.0 * ts0.one_hands),
                100.0 * ts1.one_hands,
                format!("{:.2}%", 100.0 * ts0.alternates),
                100.0 * ts1.alternates,
                format!("{:.2}%", 100.0 * ts0.alternates_same_finger_skipgrams),
                100.0 * ts1.alternates_same_finger_skipgrams,
                format!(
                    "{:.2}%",
                    100.0 * (ts0.alternates + ts0.alternates_same_finger_skipgrams)
                ),
                100.0 * (ts1.alternates + ts1.alternates_same_finger_skipgrams),
                format!("{:.3}%", 100.0 * ts0.redirects),
                100.0 * ts1.redirects,
                format!("{:.3}%", 100.0 * ts0.redirects_same_finger_skipgrams),
                100.0 * ts1.redirects_same_finger_skipgrams,
                format!("{:.3}%", 100.0 * ts0.bad_redirects),
                100.0 * ts1.bad_redirects,
                format!("{:.3}%", 100.0 * ts0.bad_redirects_same_finger_skipgrams),
                100.0 * ts1.bad_redirects_same_finger_skipgrams,
                format!(
                    "{:.3}%",
                    100.0 * (ts0.redirects + ts0.redirects_same_finger_skipgrams + ts0.bad_redirects + ts0.bad_redirects_same_finger_skipgrams)
                ),
                100.0 * (ts1.redirects + ts1.redirects_same_finger_skipgrams + ts1.bad_redirects + ts1.bad_redirects_same_finger_skipgrams),
                format!("{:.3}%", 100.0 * ts0.bad_same_finger_bigrams),
                100.0 * ts1.bad_same_finger_bigrams,
                format!("{:.3}%", 100.0 * ts0.same_finger_trigrams),
                100.0 * ts1.same_finger_trigrams,
                format!("{:.3}", l0.score),
                l1.score
            ).as_str(),
        );

        return Ok(result);
    }

    pub fn sfr_freq(&self) -> f64 {
        let len = self.language_data.characters.len();
        let chars = 0..len;

        return chars.clone().cartesian_product(chars).filter(|(i1, i2)| i1 == i2).map(|(c1, c2)| {
            self.language_data.bigrams.get(c1 * len + c2).unwrap_or(&0.0)
        }).sum();
    }

    fn sfbs(&self, o: &Sfbs) -> String {
        return match self.layout_by_name(o.name.as_str()) {
            | Some(layout) => {
                let top_n = o.count.unwrap_or(10).min(48);

                let mut response = format!("top {} sfbs for {name}:", top_n);

                for (bigram, freq) in self.layout_generator.same_finger_bigrams(layout, &self.language_data.converter, top_n) {
                    response = concat!(response, format!("{bigram}: {:.3}%", freq * 100.0)).to_string()
                }
            },
            | None => {
                format!("layout {name} does not exist!")
            },
        };
    }

    pub fn readline() -> io::Result<String> {
        let mut buf = String::new();

        write!(io::stdout(), "> ")?;

        io::stdout().flush()?;
        io::stdin().read_line(&mut buf)?;

        return Ok(buf);
    }

    pub fn generate_n_with_pins(&self, based_on: &FastLayout, amount: usize) {
        if amount == 0
        {}

        // println!(
        // "Optimizing {amount} variants took: {} seconds",
        // start.elapsed().as_secs()
        // );

        // layouts.sort_by(|l0, l1| l1.score.partial_cmp(&l0.score).unwrap());
        // print_message(language_data, &mut layouts);
    }

    pub unsafe fn generate(&mut self, generate: &Generate) -> String {
        let amount = generate.count;

        if amount == 0
        {
            return "".to_string();
        }

        let mut response = format!("generating {amount} layouts...");

        let start = std::time::Instant::now();

        (0..amount).into_par_iter().map(|_| self.layout_generator.generate_layout()).collect::<Layouts>().iter().for_each(|x| self.layouts.push(x.clone()));

        let time = start.elapsed().as_secs();

        response = concat!(
        response,
        format!("optimizing {amount} variants took: {time} seconds")
        ).to_string();

        // layouts.sort_by(|l1, l2| l2.score.partial_cmp(&l1.score).unwrap());

        self.print_message();

        return response;
    }

    pub fn get_ngram_info(&self, data: &LanguageData, ngram: &str) -> String {
        return match ngram.chars().count() {
            | 1 => {
                let c = ngram.chars().next().unwrap();
                let u = data.converter.codomain.single(c);
                let occ = data.characters.get(u as usize).unwrap_or(&0.0) * 100.0;

                format!("{ngram}: {occ:.3}%")
            },
            | 2 => {
                let bigram: [char; 2] = ngram.chars().collect::<Vec<char>>().try_into().unwrap();
                let c1 = data.converter.codomain.single(bigram[0]) as usize;
                let c2 = data.converter.codomain.single(bigram[1]) as usize;

                let b1 = c1 * data.characters.len() + c2;
                let b2 = c2 * data.characters.len() + c1;

                let rev = bigram.into_iter().rev().collect::<String>();

                let occ_b1 = data.bigrams.get(b1).unwrap_or(&0.0) * 100.0;
                let occ_b2 = data.bigrams.get(b2).unwrap_or(&0.0) * 100.0;
                let occ_s = data.skipgrams.get(b1).unwrap_or(&0.0) * 100.0;
                let occ_s2 = data.skipgrams.get(b2).unwrap_or(&0.0) * 100.0;

                format!(
                    "{ngram} + {rev}: {:.3}%,\n  {ngram}: {occ_b1:.3}%\n  {rev}: {occ_b2:.3}%\n\
                {ngram} + {rev} (skipgram): {:.3}%,\n  {ngram}: {occ_s:.3}%\n  {rev}: {occ_s2:.3}%",
                    occ_b1 + occ_b2,
                    occ_s + occ_s2
                )
            },
            | 3 => {
                let trigram: Trigram<char> = ngram.chars().collect::<Vec<char>>().try_into().unwrap();

                let trigram: Trigram<u8> = Trigram(
                    data.converter.codomain.single(trigram[0]),
                    data.converter.codomain.single(trigram[1]),
                    data.converter.codomain.single(trigram[2]),
                );

                let &(_, occ) = data.trigrams.iter().find(|&&(tf, _)| tf == t).unwrap_or(&(t, 0.0));

                format!("{ngram}: {:.3}%", occ * 100.0)
            },
            | _ => "Invalid ngram! It must be 1, 2 or 3 chars long.".to_string(),
        };
    }

    fn print_message(&self) {
        for (i, layout) in self.layouts.iter().enumerate().take(10) {
            let printable = heatmap_string(&self.language_data, layout);

            // println!("#{}, score: {:.5}\n{printable}", i, layout.score);
        }
    }

    fn set_progress_bar(amount: u64) -> ProgressBar {
        let pb = ProgressBar::new(amount);

        pb.set_style(ProgressStyle::default_bar().template("[{elapsed_precise}] [{wide_bar:.white/white}] [eta: {eta:>3}] - {per_sec:>11} {pos:>6}/{len}").expect("couldn't initialize the progress bar template").progress_chars("=>-")
        );

        return pb;
    }
}

const COMPARE_NAMES_TEMPLATE: str = *concat!(
"Sfb:                {: <11} Sfb:                {:.3}%\n",
"Dsfb:               {: <11} Dsfb:               {:.3}%\n",
"Finger Speed:       {: <11} Finger Speed:       {:.3}\n",
"Scissors:           {: <11} Scissors:           {:.3}%\n",
"Lsbs:               {: <11} Lsbs:               {:.3}%\n",
"Pinky Ring Bigrams: {: <11} Pinky Ring Bigrams: {:.3}%\n\n",
"Inrolls:            {: <11} Inrolls:            {:.2}%\n",
"Outrolls:           {: <11} Outrolls:           {:.2}%\n",
"Total Rolls:        {: <11} Total Rolls:        {:.2}%\n",
"Onehands:           {: <11} Onehands:           {:.3}%\n\n",
"Alternates:         {: <11} Alternates:         {:.2}%\n",
"Alternates Sfs:     {: <11} Alternates Sfs:     {:.2}%\n",
"Total Alternates:   {: <11} Total Alternates:   {:.2}%\n\n",
"Redirects:          {: <11} Redirects:          {:.3}%\n",
"Redirects Sfs:      {: <11} Redirects Sfs:      {:.3}%\n",
"Bad Redirects:      {: <11} Bad Redirects:      {:.3}%\n",
"Bad Redirects Sfs:  {: <11} Bad Redirects Sfs:  {:.3}%\n",
"Total Redirects:    {: <11} Total Redirects:    {:.3}%\n\n",
"Bad Sfbs:           {: <11} Bad Sfbs:           {:.3}%\n",
"Sft:                {: <11} Sft:                {:.3}%\n\n",
"Score:              {: <11} Score:              {:.3}\n"
);
