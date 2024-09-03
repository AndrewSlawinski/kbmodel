use crate::flags::{
    Analyze,
    Compare,
    Ngram,
    Rank,
    Sfbs,
    Sfts,
};
use itertools::Itertools;
use oxeylyzer_core::config::config::Config;
use oxeylyzer_core::data_dir::DataFetch;
use oxeylyzer_core::language_data::LanguageData;
use oxeylyzer_core::layout::layout::Layout;
use oxeylyzer_core::stats::bigram_stats::BType::*;
use oxeylyzer_core::stats::disjoint_stats::DType::*;
use oxeylyzer_core::stats::layout_stats::LayoutStats;
use oxeylyzer_core::stats::trigram_stats::TType::*;
use oxeylyzer_core::type_def::Fixed;
use std::collections::HashMap;

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

    pub fn readline() -> std::io::Result<String>
    {
        use std::io::Write;

        let mut buf = String::new();

        write!(std::io::stdout(), "> ")?;

        std::io::stdout().flush()?;
        std::io::stdin().read_line(&mut buf)?;

        return Ok(buf);
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
                | Ok(false) => continue,
                | Ok(true) =>
                {
                    println!("Exiting analyzer...");

                    break;
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

        let response: String = match flags.subcommand
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

        println!("{response}");

        return Ok(false);
    }

    fn compare(&mut self, o: Compare) -> String
    {
        let name0 = o.name1;
        let name1 = o.name2;

        let mut result = format!("\n{name0:31}{name1}\n");

        let binding = self.analyze(Analyze {
            name_or_number: name0,
        });

        let s0 = binding.split('\n').collect_vec();

        let binding = self.analyze(Analyze {
            name_or_number: name1,
        });

        let s1 = binding.split('\n').collect_vec();

        let compare_map = s0
            .into_iter()
            .zip(s1)
            .map(|(a, b)| {
                return if a.chars().count() > 64
                {
                    format!("{a}{:10}{b}", " ")
                }
                else
                {
                    format!("{a:31}{b}")
                };
            })
            .collect_vec()
            .join("\n");

        result.push_str(compare_map.as_str());

        return result;
    }

    fn analyze(&mut self, o: Analyze) -> String
    {
        let name = o.name_or_number.as_str();
        let layout = self.layout_by_name(name);

        if layout.is_none()
        {
            return format!("'{name}' does not exist!");
        }

        let layout = layout.unwrap();

        let stats = LayoutStats::new(&self.language_data, &layout);

        let layout_str = Self::heatmap(&self.language_data.characters, &layout.matrix).join("\n");

        return format!(
            "{layout_str}\n\n\
            {}\n\
            {}\n\
            {}",
            stats.bigram_stats, stats.trigram_stats, stats.disjoint_stats,
        );
    }

    pub fn rank(&self, rank: Rank) -> String
    {
        use rayon::iter::*;

        let mut v = self
            .layouts
            .iter()
            .par_bridge()
            .map(|(name, layout)| {
                let stats = LayoutStats::new(&self.language_data, &layout);

                let a = [
                    stats[SFB],
                    stats[D1SFB],
                    stats[SFT],
                    stats[LSB],
                    stats[D1LSB],
                ];

                let metric = a.into_iter().fold(0., |c, x| c + x);

                return (name.clone(), metric);
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

        return v
            .iter()
            .map(|(n, s)| format!("{:24} {:.5}", n, s))
            .join("\n");
    }

    fn sfbs(&self, o: Sfbs) -> String
    {
        return match self.layout_by_name(o.name.as_str())
        {
            | None =>
            {
                format!("Layout \"{}\" does not exist.", o.name)
            },
            | Some(layout) =>
            {
                let top_n = o.count.unwrap_or(10).min(48);

                let mut response = format!("Top {top_n} SFBs for {}:\n", o.name);

                let mut v = Vec::new();

                for i in 0 .. 30
                {
                    for j in 0 .. 30
                    {
                        if LayoutStats::is_sf(&mut [i as u8, j as u8])
                        {
                            let c0 = layout.matrix[i];
                            let c1 = layout.matrix[j];

                            if [c0, c1].iter().any(char::is_ascii_punctuation)
                            {
                                continue;
                            }

                            let bigram = format!("{}{}", c0, c1);
                            let freq = self.language_data.bigrams.get(&bigram).unwrap_or(&0.);

                            v.push((bigram, freq));
                        }
                    }
                }

                v.sort_by(|(_, f0), (_, f1)| f1.partial_cmp(f0).unwrap());

                v.iter().take(top_n).for_each(|(s, f)| {
                    response.push_str(format!("{} {:.5}\n", s, *f * 100.).as_str())
                });

                return response;
            },
        };
    }

    fn sfts(&self, o: Sfts) -> String
    {
        return match self.layout_by_name(o.name.as_str())
        {
            | None =>
            {
                format!("Layout \"{}\" does not exist.", o.name)
            },
            | Some(layout) =>
            {
                let top_n = o.count.unwrap_or(10).min(48);

                let mut response = format!("Top {top_n} SFTs for {}:\n", o.name);

                let mut v = Vec::new();

                for i in 0 .. 30
                {
                    for j in 0 .. 30
                    {
                        for k in 0 .. 30
                        {
                            if LayoutStats::is_sf(&mut [i as u8, j as u8, k as u8])
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

                v.iter().take(top_n).for_each(|(s, f)| {
                    response.push_str(format!("{} {:.5}\n", s, *f * 100.).as_str())
                });

                return response;
            },
        };
    }

    pub fn ngram(&mut self, ngram: Ngram) -> String
    {
        let ngram = ngram.ngram;

        return match ngram.chars().count()
        {
            | 1 =>
            {
                let c = ngram.chars().next().unwrap();
                let p = self.language_data.characters.get(&c).unwrap_or(&0.) * 100.;

                format!("{ngram}: {p:.3}%")
            },
            | 2 =>
            {
                let b0 = ngram.clone();
                let p0 = self.language_data.bigrams.get(&b0).unwrap_or(&0.0) * 100.;
                let s0 = self.language_data.skipgrams.get(&b0).unwrap_or(&0.0) * 100.;

                let temp = ngram.chars().collect_vec();

                return if temp[0] == temp[1]
                {
                    format!(
                        "[bigram]:   {:.3}%\n\
                    [skipgram]: {:.3}%",
                        p0, s0
                    )
                }
                else
                {
                    let b1: String = ngram.chars().rev().collect();
                    let p1 = self.language_data.bigrams.get(&b1).unwrap_or(&0.0) * 100.;
                    let s1 = self.language_data.skipgrams2.get(&b1).unwrap_or(&0.0) * 100.;

                    format!(
                        "[bigram]:   {:.5}%\n\
                        \t{b0}: {p0:.5}%\n\
                        \t{b1}: {p1:.5}%\n\
                        [skipgram]: {:.5}%\n\
                        \t{b0}: {s0:.5}%\n\
                        \t{b1}: {s1:.5}%",
                        p0 + p1,
                        s0 + s1
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
                "Invalid ngram! It must be 1, 2 or 3 chars long. ".to_string()
            },
        };
    }
}

impl Repl
{
    fn layout_by_name(&self, name: &str) -> Option<Layout>
    {
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

    pub fn heatmap(data: &HashMap<char, f32>, chars: &Fixed<char>) -> Vec<String>
    {
        let mut map = Vec::new();
        let mut print_str = String::new();

        for (i, c) in chars.iter().enumerate()
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

            let p = *data.get(c).unwrap_or(&0.0);

            let heat = Self::heat(*c, p);

            print_str.push_str(heat.as_str());
            print_str.push(' ');
        }

        map.push(print_str.clone());

        return map;
    }

    fn weight_stats(&self, stats: &mut LayoutStats)
    {
        for (k, mut v) in stats.bigram_stats.inner.iter_mut()
        {
            match k
            {
                | SFB =>
                {},
                | LSB =>
                {},
                | S1SFB =>
                {},
                | S2SFB =>
                {},
                | S3SFB =>
                {},
                | IRB =>
                {},
                | ORB =>
                {},
                | AB =>
                {},
                | Repeat =>
                {},
                | S =>
                {},
            }
        }

        for (k, v) in stats.trigram_stats.inner.iter()
        {
            match k
            {
                | SFT =>
                {},
                | IRT =>
                {},
                | ORT =>
                {},
                | Redirect =>
                {},
                | AT =>
                {},
            }
        }

        for (k, v) in stats.disjoint_stats.inner.iter()
        {
            match k
            {
                | D1S =>
                {},
                | D1IRB =>
                {},
                | D1ORB =>
                {},
                | D1SFB =>
                {},
                | D1LSB =>
                {},
                | D1Repeat =>
                {},
            }
        }
    }
}
