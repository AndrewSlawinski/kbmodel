#![allow(deprecated)]

use ansi_rgb::{
    rgb,
    Colorable,
};

use crate::n_gram::n_gram::NGram;
use crate::{
    language::language_data::LanguageData,
    layout::layout::FastLayout,
};

pub struct Heatmap {
    string: String,
}

pub fn heatmap_heat(data: &LanguageData, c: char) -> String {
    let complement = f64::MAX / *data.characters.get(&NGram::from(&[c])).unwrap_or(&0.0);
    let complement = u8::MAX ^ complement as u8;

    let heat = rgb(215, complement, complement);

    let formatted = c.to_string().fg(heat);

    return format!("{formatted}");
}

pub fn heatmap_string(data: &LanguageData, layout: &FastLayout) -> String {
    let mut print_str = String::new();

    for (i, c) in layout.matrix.iter().enumerate() {
        if i % 10 == 0 && i > 0
        {
            print_str.push('\n');
        }

        if (i + 5) % 10 == 0
        {
            print_str.push(' ');
        }

        print_str.push_str(heatmap_heat(data, *c).as_str());
        print_str.push(' ');
    }

    return print_str;
}
