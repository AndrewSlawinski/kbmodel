use std::fs::File;
use std::time::Instant;

use crate::language::text_data::TextData;
use crate::language::text_n_grams::TextNgrams;
use crate::{
    data_dir::files_in,
    translation::Translator,
};
use file_chunker::FileChunker;
use itertools::Itertools;
use rayon::iter::{
    IntoParallelRefIterator,
    ParallelBridge,
    ParallelIterator,
};
use smartstring::{
    LazyCompact,
    SmartString,
};

pub fn load_data(language: &str, translator: &Translator) {
    let start_total = Instant::now();

    let files = files_in(vec!["text", language]);

    let files = files
        .par_bridge()
        .filter_map(Result::ok).flat_map(|path| File::open(path.path())).map(|file| chunk(&file)).collect::<Vec<(FileChunker, usize)>>();

    let time = Instant::now();
    let mut now = (time - start_total).as_millis();

    println!("Prepared text files in {now}ms", );

    let strings = files
        .par_iter()
        .flat_map(|(chunker, count)| chunker.chunks(*count, Some(' ')).unwrap())
        .map(|chunk| {
            std::str::from_utf8(chunk).expect(
                "one of the files provided is not encoded as utf-8.\
                Make sure all files in the directory are valid utf-8.",
            )
        })
        .map(|s| {
            let mut last_chars = SmartString::<LazyCompact>::new();
            let mut inter = [' '; 5];
            s.chars()
                .rev()
                .take(5)
                .enumerate()
                .for_each(|(i, c)| unsafe { *inter.get_unchecked_mut(4 - i) = c });

            inter.into_iter().for_each(|c| last_chars.push(c));
            last_chars.push_str("     ");

            return (s, last_chars);
        })
        .collect::<Vec<_>>();

    now = (Instant::now() - time).as_millis();
    println!("Converted to UTF8 in {now}ms", );

    let quingrams = strings
        .par_iter().map(|(s, last)| TextNgrams::new(s, last)).reduce(TextNgrams::default, |accum, new| accum.combine_with(new));

    let text_data = TextData::from((&quingrams, language, translator));

    text_data.save(&translator.is_raw);

    println!(
        "loading {language} took {}ms",
        (Instant::now() - start_total).as_millis()
    );
}

fn chunk(file: &File) -> (FileChunker, usize) {
    let len = file.metadata().unwrap().len() + 1;
    let count = (len / (1024 * 1024 * 4)).max(1);

    return (FileChunker::new(&file).unwrap(), count as usize);
}
