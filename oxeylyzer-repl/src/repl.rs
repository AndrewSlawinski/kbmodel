use crate::flags::ReplCmd::Ngram;
use crate::flags::{
    Analyze,
    Compare,
    Generate,
    Improve,
    Include,
    Load,
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
use oxeylyzer_core::data_dir::DataFetch;
use oxeylyzer_core::language::language_data::{
    LanguageData,
    LanguageDataIntermediate,
};
use oxeylyzer_core::layout::layout::FastLayout;
use oxeylyzer_core::n_gram::n_gram::NGram;
use oxeylyzer_core::stats::layout_stats::NGramType::Trigram;
use oxeylyzer_core::translation::Translator;
use oxeylyzer_core::utility::generator::Generator;
use oxeylyzer_core::utility::heat_map::{
    heatmap_heat,
    heatmap_string,
};
use oxeylyzer_core::utility::scorer::Scorer;
use std::io;
use std::io::{
    Read,
    Write,
};
use std::path::PathBuf;

pub struct Repl<'a> {
    layout_generator: Generator<'a>,
    layouts: Vec<FastLayout>,
    temp_generated: Vec<FastLayout>,
    config: Config,
    language_data: LanguageData,
    scorer: Scorer<'a>,
}

impl<'a> Repl {
    pub fn new() -> Self {
        let config = Config::new();

        let mut file = DataFetch::get_language_data_file(config.info.language.as_str());
        let mut contents = String::new();

        file.read_to_string(&mut contents).unwrap();

        let mut language_data_intermadiate: LanguageDataIntermediate = serde_json::from_str(contents.as_str()).unwrap();

        let language_data = LanguageData::from(&mut language_data_intermadiate);

        let scorer = Scorer::new(&language_data, &config.weights, &config.info.keyboard_type);
        let layout_generator = Generator::new(&language_data, &config, &scorer);

        let layouts = DataFetch::load_layouts_in_language(config.info.language.as_str());

        return Self {
            layouts,
            layout_generator,
            temp_generated: Vec::new(),
            config,
            language_data,
            scorer,
        };
    }

    pub fn run(&mut self) {
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
                }
                | Ok(false) => continue,
                | Err(err) => {
                    println!("{err}");
                }
            }
        }
    }

    fn respond(&mut self, line: &str) -> Result<bool, String> {
        use crate::flags::Repl;
        use crate::flags::ReplCmd::*;

        let args = shlex::split(line).ok_or("Invalid quotations")?.into_iter().map(std::ffi::OsString::from).collect_vec();

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
                        self.language(l);

                        format!("Set language to {l}. Sfr: {:.2}%", self.sfr_freq())
                    }
                    | None => format!("{l} not found."),
                }
            }
            | Include(o) => {
                self.include(&o);

                format!("{} has been included.", o.language)
            }
            | Languages(_) => Self::languages(),
            | Load(o) => self.load(o),
            | Ngram(o) => self.n_gram(&mut self.language_data, &o.ngram),
            | Reload(_) => {
                self.reload();

                "Model has been reloaded.".to_string()
            }
            | Quit(_) => {
                return Ok(true);
            }
        };

        println!("{response}");

        return Ok(false);
    }

    fn load(&mut self, o: Load) -> String {
        let mut r = "";

        r = match (o.all, o.raw) {
            | (true, true) => "You can't currently generate all corpora as raw",
            | (true, false) => {
                for (language, config) in CorpusConfig::all() {
                    r = concat!(r, format!("loading data for language: {language}..."));

                    DataFetch::load_data(language.as_str(), &config.translator());
                }

                r
            }
            | (false, true) => {
                DataFetch::load_data(o.language.to_str().unwrap(), &Translator::raw(true));

                format!("loading raw data for language: {}...", o.language.display()).as_str()
            }
            | (false, false) => {
                let language = o.language.to_str().ok_or_else(|| format!("Language is invalid utf8: {:?}", o.language)).unwrap();

                let translator = CorpusConfig::new_translator(language, None);

                println!("loading data for {}...", &language);

                DataFetch::load_data(language, &translator);

                if translator.is_raw
                {
                    r
                } else {
                    self.language_data = LanguageData::new(language);
                    self.config.info.language = language.to_string();
                    self.layouts = DataFetch::load_layouts_in_language(language);

                    format!(
                        "Set language to {}. Sfr: {:.2}%",
                        language,
                        self.sfr_freq() * 100.0
                    ).as_str()
                }
            }
        };

        return r.to_string();
    }

    fn include(&mut self, o: &Include) {
        DataFetch::load_layouts_in_language(&o.language).into_iter().for_each(|layout| {
            self.layouts.push(layout);
        });

        self.layouts.sort_by(|_, a, _, b| a.score.partial_cmp(&b.score).unwrap());
    }

    fn language(&mut self, l: PathBuf) {
        let language = l.to_str().expect(format!("Language is invalid utf8: {:?}", l).as_str());

        self.layout_generator.language_data = &LanguageData::new(language);
        self.layouts = DataFetch::load_layouts_in_language(language);
        self.config.info.language = language.to_string();
    }

    fn reload(&mut self) {
        self.config = Config::new();
        self.language_data = LanguageData::new(&self.config.info.language);

        self.layouts = DataFetch::load_layouts_in_language(&self.config.info.language);
    }

    fn languages() -> String {
        let mut r = "";

        let files = DataFetch::files_in(vec!["language_data"]);

        files.for_each(|p| {
            let name = p.unwrap().file_name().to_string_lossy().replace('_', " ").replace(".json", "");

            if name != "test"
            {
                r = concat!(r, name);
            }
        });

        r.to_string()
    }

    fn save(&mut self, o: &Save) -> String {
        match self.get_nth(o.nth) {
            | Some(layout) => {
                self.save_layout(layout.clone(), o.name.clone());

                "".to_string()
            }
            | None => {
                format!(
                    "Index '{}' provided is out of bounds for {} generated layouts",
                    o.nth,
                    self.temp_generated.len()
                )
            }
        }
    }

    fn improve(&mut self, o: &Improve) -> String {
        let layout = self.layout_by_name(&o.name);

        let layouts = self.layout_generator.generate_n_with_pins_iter(&layout, o.count).collect_vec();

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
                    }
                }
            }
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
            }
        };

        println!("{}", name);

        return Ok(self.analyze_layout(layout));
    }

    fn placeholder_name(&self, layout: &FastLayout) -> Result<String, String> {
        for i in 1..1000 {
            let new_name_bytes = layout[10..14].to_vec();
            let mut new_name = self.language_data.language.as_string(new_name_bytes.as_slice());

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

        let mut f = std::fs::OpenOptions::new().write(true).create(true).truncate(true).open(format!(
            "static/layouts/{}/{}.kb",
            self.config.info.language, new_name
        )).map_err(|e| e.to_string()).unwrap();

        let layout_formatted = layout.formatted_string();

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

    pub fn compare_name(&self, language_data: &LanguageData, name0: &str, name1: &str) -> String {
        let mut result = String::from(format!("\n{:31}{}", name0, name1));

        let l0 = self.layout_by_name(name0);
        let l1 = self.layout_by_name(name1);

        for y in 0..3 {
            for layout in [l0, l1].into_iter() {
                result.push_str("        ");

                for x in 0..10 {
                    if x == 10 / 2
                    {
                        result.push(' ');
                    }

                    result.push_str(heatmap_heat(&language_data, layout.get(x + 10 * y)).as_str());
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

                for (bigram, freq) in self.layout_generator.same_finger_bigrams(layout, top_n) {
                    response = concat!(response, format!("{bigram}: {:.3}%", freq * 100.0)).to_string()
                }
            }
            | None => {
                format!("layout {name} does not exist!")
            }
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

    pub fn generate(&mut self, generate: &Generate) -> String {
        let amount = generate.count;

        if amount == 0
        {
            return "".to_string();
        }

        let mut response = format!("generating {amount} layouts...");

        let start = std::time::Instant::now();

        if amount == 1
        {
            self.layouts.push(self.layout_generator.generate_layout());
        } else {
            self.layout_generator.generate_layouts(amount).iter().for_each(|x| self.layouts.push(x.clone()));
        }

        let time = start.elapsed().as_secs();

        response = concat!(
        response,
        format!("optimizing {amount} variants took: {time} seconds")
        ).to_string();

        self.layouts.sort_by(|l1, l2| l2.score.partial_cmp(&l1.score).unwrap());

        self.print_message();

        return response;
    }

    pub fn n_gram(&self, language_data: &LanguageData, ngram: &str) -> String {
        return match ngram.chars().count() {
            | 1 => {
                let c = ngram.chars().next().unwrap();
                let occ = language_data.characters.get(&NGram::from(&[c])).unwrap_or(&0.0) * 100.0;

                format!("{ngram}: {occ:.3}%")
            }
            | 2 => {
                let bigram: [char; 2] = ngram.chars().collect_vec().try_into().unwrap();

                let b1 = c1 * language_data.characters.len() + c2;
                let b2 = c2 * language_data.characters.len() + c1;

                let rev = bigram.into_iter().rev().collect::<String>();

                let occ_b1 = language_data.bigrams.get(b1).unwrap_or(&0.0);
                let occ_b2 = language_data.bigrams.get(b2).unwrap_or(&0.0);
                let occ_s = language_data.skipgrams.get(b1).unwrap_or(&0.0);
                let occ_s2 = language_data.skipgrams.get(b2).unwrap_or(&0.0);

                format!(
                    "{ngram} + {rev}: {:.3}%,\n  {ngram}: {occ_b1:.3}%\n  {rev}: {occ_b2:.3}%\n\
                {ngram} + {rev} (skipgram): {:.3}%,\n  {ngram}: {occ_s:.3}%\n  {rev}: {occ_s2:.3}%",
                    occ_b1 + occ_b2,
                    occ_s + occ_s2
                )
            }
            | 3 => {
                let trigram: Trigram<char> = ngram.chars().collect_vec().try_into().unwrap();

                let trigram: NGram<u8, 3> = Trigram();

                let &(_, occ) = language_data.trigrams.iter().find(|&&(tf, _)| tf == t).unwrap_or(&(t, 0.0));

                format!("{ngram}: {:.3}%", occ * 100.0)
            }
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
