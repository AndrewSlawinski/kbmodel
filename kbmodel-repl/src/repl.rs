use crate::flags::{
    Analyze,
    Compare,
    Ngram,
    Rank,
    Sfbs,
    Sfts,
};
use futures::executor::block_on;
use itertools::Itertools;
use kbmodel_core::config::config::Config;
use kbmodel_core::data_dir::DataFetch;
use kbmodel_core::language_data::LanguageData;
use kbmodel_core::layout::layout::Layout;
use kbmodel_core::stats::alt_stats::AltStats;
use kbmodel_core::stats::alt_stats::DataSet::{
    Bigram,
    Disjoint,
};
use kbmodel_core::stats::predicates::Predicates;
use kbmodel_core::stats::stat_matrices::StatMatrices;
use kbmodel_core::stats::stat_type::StatType::SameFinger;
use kbmodel_core::type_def::Fixed;
use std::collections::HashMap;
use std::io;

pub struct Repl
{
    config: Config,
    language_data: LanguageData,

    layouts: HashMap<String, Layout>,
}

impl Repl
{
    pub fn new() -> Self
    {
        let config = Config::default();

        let language_data = Self::load_language(&config.info.language);

        let fetch = DataFetch::layout_files_in_language(config.info.language.as_str());
        let layouts = DataFetch::load_layouts(fetch);

        return Self {
            layouts,
            config,
            language_data,
        };
    }

    pub fn readline() -> io::Result<String>
    {
        use std::io::Write;

        let mut buf = String::new();

        write!(io::stdout(), "> ")?;

        io::stdout().flush()?;
        io::stdin().read_line(&mut buf)?;

        return Ok(buf);
    }

    pub fn run(&mut self) -> io::Result<()>
    {
        loop
        {
            let line = Repl::readline()?;
            let line = line.trim();

            if line.is_empty()
            {
                continue;
            }

            match self.respond(line)
            {
                | Ok(false) => continue,
                | Ok(true) =>
                {
                    println!("Exiting analyzer...");

                    return Ok(());
                },
                | Err(err) => println!("{err}"),
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

        match flags.subcommand
        {
            | Analyze(o) => self.analyze(o),
            | Compare(o) => self.compare(o),
            | Rank(o) => self.rank(o),
            | Sfbs(o) => self.sfbs(o),
            | Sfts(o) => self.sfts(o),
            | Ngram(o) => self.ngram(o),
            | Quit(_) =>
            {
                return Ok(true);
            },
        };

        return Ok(false);
    }

    fn compare(&mut self, o: Compare)
    {
        let name0 = o.name1;
        let name1 = o.name2;

        let mut result = format!("\n{name0:44}{name1}\n");

        let l0 = self.layout_by_name(name0.as_str());
        let l1 = self.layout_by_name(name1.as_str());

        if l0.is_none() || l1.is_none()
        {
            return;
        }

        let s0 = self.analyze_preformat(&l0.unwrap());
        let s1 = self.analyze_preformat(&l1.unwrap());

        let heatmap =
            s0.1.map
                .iter()
                .zip(&s1.1.map)
                .map(|(a, b)| format!("{a}{:23}{b}", " "))
                .collect_vec()
                .join("\n");

        let st0 = format!("\n\n{}", s0.0);
        let st1 = format!("\n\n{}", s1.0);

        let stats = st0
            .split("\n")
            .zip(st1.split("\n"))
            .map(|(a, b)| format!("{a:44}{b}"))
            .collect_vec()
            .join("\n");

        result.push_str(heatmap.as_str());
        result.push_str(stats.as_str());

        println!("{result}");
    }

    fn analyze_preformat(&self, layout: &Layout) -> (AltStats, HeatMap)
    {
        use futures::executor::block_on;

        let mut t = StatMatrices::new();
        block_on(t.compute(&layout.matrix, &self.language_data));

        let mut alt = AltStats::new();
        block_on(alt.compute(&t));

        // let stats = LayoutStats::new(&self.language_data, &layout);
        let heatmap = HeatMap::new(&self.language_data.characters, &layout.matrix);

        return (alt, heatmap);
    }

    fn analyze(&mut self, o: Analyze)
    {
        let name = o.name_or_number.as_str();
        let layout = self.layout_by_name(name);

        if layout.is_none()
        {
            return;
        }

        let layout = layout.unwrap();

        let l = self.analyze_preformat(&layout);

        let stats = l.0;

        let layout_str = l.1.map.join("\n");

        println!("{layout_str}\n\n{stats}");
    }

    pub fn rank(&self, rank: Rank)
    {
        use rayon::iter::*;

        let mut v = self
            .layouts
            .iter()
            .par_bridge()
            .map(|(name, layout)| {
                let mut matrices = StatMatrices::new();
                block_on(matrices.compute(&layout.matrix, &self.language_data));

                let mut stats = AltStats::new();
                block_on(stats.compute(&matrices));

                let a = [stats[&Bigram][&SameFinger], stats[&Disjoint][&SameFinger]];

                let metric = a.into_iter().fold(0., |c, x| c + x);

                return (name, metric);
            })
            .collect::<Vec<_>>();

        v.sort_by(|(_, s0), (_, s1)| {
            if rank.asc.unwrap_or(true)
            {
                s1.partial_cmp(s0).unwrap()
            }
            else
            {
                s0.partial_cmp(s1).unwrap()
            }
        });

        v.iter().for_each(|(n, s)| println!("{n:24} {s:.5}"));
    }

    fn sfbs(&self, o: Sfbs)
    {
        match self.layout_by_name(o.name.as_str())
        {
            | None =>
            {
                println!("Layout \"{}\" does not exist.", o.name)
            },
            | Some(layout) =>
            {
                let top_n = o.count.unwrap_or(10).min(48);

                let mut v = Vec::new();

                for i in 0 .. 30
                {
                    for j in 0 .. 30
                    {
                        if Predicates::is_sf(&mut [i as u8, j as u8])
                        {
                            let c0 = layout.matrix[i];
                            let c1 = layout.matrix[j];

                            if [c0, c1].iter().any(char::is_ascii_punctuation)
                            {
                                continue;
                            }

                            let bigram = format!("{c0}{c1}");
                            let freq = self.language_data.bigrams.get(&bigram).unwrap_or(&0.);

                            v.push((bigram, freq));
                        }
                    }
                }

                v.sort_by(|(_, f0), (_, f1)| f1.partial_cmp(f0).unwrap());

                println!("Top {top_n} SFBs for {}:\n", o.name);

                v.iter()
                    .take(top_n)
                    .for_each(|(s, f)| println!("{} {:.5}\n", s, *f * 100.));
            },
        };
    }

    fn sfts(&self, o: Sfts)
    {
        return match self.layout_by_name(o.name.as_str())
        {
            | None =>
            {
                println!("Layout \"{}\" does not exist.", o.name)
            },
            | Some(layout) =>
            {
                let top_n = o.count.unwrap_or(10).min(48);

                let mut v = Vec::new();

                for i in 0 .. 30
                {
                    for j in 0 .. 30
                    {
                        for k in 0 .. 30
                        {
                            if Predicates::is_sf(&mut [i as u8, j as u8, k as u8])
                            {
                                let c0 = layout.matrix[i];
                                let c1 = layout.matrix[j];
                                let c2 = layout.matrix[k];

                                if [c0, c1, c2].iter().any(char::is_ascii_punctuation)
                                {
                                    continue;
                                }

                                let trigram = format!("{}{}{}", c0, c1, c2);
                                let freq = self.language_data.trigrams.get(&trigram).unwrap_or(&0.);

                                v.push((trigram, freq));
                            }
                        }
                    }
                }

                v.sort_by(|(_, f0), (_, f1)| f1.partial_cmp(f0).unwrap());

                println!("Top {top_n} SFTs for {}:\n", o.name);

                v.iter()
                    .take(top_n)
                    .for_each(|(s, f)| println!("{} {:.5}\n", s, *f * 100.));
            },
        };
    }

    pub fn ngram(&mut self, ngram: Ngram)
    {
        let ngram = ngram.ngram;

        return match ngram.chars().count()
        {
            | 1 =>
            {
                let c = ngram.chars().next().unwrap();
                let p = self
                    .language_data
                    .characters
                    .get(&format!("{c}"))
                    .unwrap_or(&0.)
                    * 100.;

                println!("{ngram}: {p:.3}%")
            },
            | 2 =>
            {
                let chars = ngram.chars().collect_vec();
                let p0 = self.language_data.bigrams.get(&ngram).unwrap_or(&0.0) * 100.;

                let s01 = self.language_data.skipgrams.get(&ngram).unwrap_or(&0.0) * 100.;
                let s02 = self.language_data.skipgrams2.get(&ngram).unwrap_or(&0.0) * 100.;
                let s03 = self.language_data.skipgrams3.get(&ngram).unwrap_or(&0.0) * 100.;

                let mut d0 = 0.;

                for c in self.language_data.characters.keys()
                {
                    d0 += self
                        .language_data
                        .trigrams
                        .get(&format!("{}{}{}", chars[0], c, chars[1]))
                        .unwrap_or(&0.0)
                        * 100.;
                }

                if chars[0] == chars[1]
                {
                    println!(
                        "[bigram]: {p0:.3}%\n\
                    [disjoint]: {d0:.3}%\n\
                    [skip1]: {s01:.3}%\n\
                    [skip2]: {s02:.3}%\n\
                    [skip3]: {s03:.3}%",
                    )
                }
                else
                {
                    let b1: String = ngram.chars().rev().collect();
                    let p1 = self.language_data.bigrams.get(&b1).unwrap_or(&0.0) * 100.;

                    let s11 = self.language_data.skipgrams.get(&b1).unwrap_or(&0.0) * 100.;
                    let s12 = self.language_data.skipgrams2.get(&b1).unwrap_or(&0.0) * 100.;
                    let s13 = self.language_data.skipgrams3.get(&b1).unwrap_or(&0.0) * 100.;

                    let mut d1 = 0.;

                    for c in self.language_data.characters.keys()
                    {
                        d1 += self
                            .language_data
                            .trigrams
                            .get(&format!("{}{}{}", chars[1], c, chars[0]))
                            .unwrap_or(&0.0)
                            * 100.;
                    }

                    println!(
                        "[bigram]: {:.5}%\n\
                        \t{ngram}: {p0:.5}%\n\
                        \t{b1}: {p1:.5}%\n\
                        [disjoint]: {:.5}%\n\
                        \t{ngram}: {d0:.5}%\n\
                        \t{b1}: {d1:.5}%\n\
                        [skip1]: {:.5}%\n\
                        \t{ngram}: {s01:.5}%\n\
                        \t{b1}: {s11:.5}%\n\
                        [skip2]: {:.5}%\n\
                        \t{ngram}: {s02:.5}%\n\
                        \t{b1}: {s12:.5}%\n\
                        [skip3]: {:.5}%\n\
                        \t{ngram}: {s03:.5}%\n\
                        \t{b1}: {s13:.5}%",
                        p0 + p1,
                        d0 + d1,
                        s01 + s11,
                        s02 + s12,
                        s03 + s13
                    )
                };
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
                println!("Invalid ngram! It must be 1, 2 or 3 chars long. ");
            },
        };
    }
}

impl Repl
{
    fn layout_by_name(&self, name: &str) -> Option<Layout>
    {
        let l = self.layouts.get(name);

        if l.is_none()
        {
            println!("Layout \"{name}\" does not exist.")
        }

        return self.layouts.get(name).cloned();
    }

    fn load_language(language: &str) -> LanguageData
    {
        use std::io::Read;
        let mut file = DataFetch::language_data_file(language);
        let mut contents = String::new();

        file.read_to_string(&mut contents).unwrap();

        let language_data = serde_json::from_str(contents.as_str()).unwrap();

        return language_data;
    }
}

struct HeatMap
{
    pub map: Vec<String>,
}

impl HeatMap
{
    pub fn heat(c: char, p: f32) -> String
    {
        use ansi_rgb::{
            rgb,
            Colorable,
        };

        let complement = 192. - p * 1720.;
        let complement = complement.max(0.) as u8;

        let heat = rgb(192, complement, complement);

        let formatted = c.to_string().fg(heat);

        return format!("{formatted}");
    }

    pub fn new(data: &HashMap<String, f32>, chars: &Fixed<char>) -> Self
    {
        let mut maps = Vec::new();
        let mut row = String::new();

        for (i, c) in chars.iter().enumerate()
        {
            if i % 10 == 0 && i != 0
            {
                maps.push(row);

                row = String::new();
            }

            if (i + 5) % 10 == 0
            {
                row.push(' ');
            }

            let p = data.get(&format!("{c}")).unwrap_or(&0.0);

            let heat = Self::heat(*c, *p);

            row.push_str(heat.as_str());
            row.push(' ');
        }

        maps.push(row);

        return Self { map: maps };
    }
}
