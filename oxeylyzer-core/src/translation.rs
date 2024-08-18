use std::collections::HashMap;

use smartstring::{
    Compact,
    LazyCompact,
    SmartString,
};

#[derive(Clone)]
pub struct Translator {
    pub table: HashMap<char, SmartString<Compact>>,
    pub is_raw: bool,
    pub is_empty: bool,
}

impl Default for Translator {
    fn default() -> Self {
        let mut translator = Translator::new();

        return translator.default_formatting().build();
    }
}

impl std::ops::Add for Translator {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.is_raw |= rhs.is_raw;
        self.is_empty &= rhs.is_empty;

        return if !self.is_empty
        {
            let base = &SmartString::<Compact>::from(" ");

            for (from, to) in rhs.table {
                let original = self.table.get(&from);

                if original.is_none() || original == Some(base)
                {
                    self.table.insert(from, to);
                }
            }

            self
        } else {
            self.table = rhs.table;

            self
        };
    }
}

impl Translator {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> TranslatorBuilder {
        return TranslatorBuilder {
            table: HashMap::new(),
            is_raw: false,
        };
    }

    #[allow(dead_code)]
    pub fn language(language: &str) -> Self {
        return Self::new().language(language).build();
    }

    #[allow(dead_code)]
    pub fn language_or_raw(language: &str) -> Self {
        return Self::language(language);
    }

    pub fn raw(unshift_chars: bool) -> Self {
        return Translator::new().raw(unshift_chars).build();
    }

    pub fn translate(&self, s: &str) -> SmartString<LazyCompact> {
        let mut res = SmartString::<LazyCompact>::new();

        for c in s.chars() {
            match self.table.get(&c) {
                | Some(replacement) => {
                    res.push_str(replacement);
                }
                | None => {
                    res.push(' ');
                }
            }
        }

        return res;
    }
}

pub struct TranslatorBuilder {
    table: HashMap<char, SmartString<Compact>>,
    is_raw: bool,
}

impl TranslatorBuilder {
    pub fn as_space(&mut self, to_string: &str) -> &mut Self {
        for c in to_string.chars() {
            self.table.insert(c, SmartString::<Compact>::from(" "));
        }

        return self;
    }

    pub fn many_different_to_one(&mut self, domain: &str, codomain: char) -> &mut Self {
        for c in domain.chars() {
            self.table.insert(c, SmartString::<Compact>::from(codomain));
        }

        return self;
    }

    pub fn keep_one(&mut self, keep: char) -> &mut Self {
        self.table.insert(keep, SmartString::<Compact>::from(keep));

        return self;
    }

    pub fn keep(&mut self, keep: &str) -> &mut Self {
        for c in keep.chars() {
            self.table.insert(c, SmartString::<Compact>::from(c));
        }

        return self;
    }

    pub fn one_to_one(&mut self, domain: &str, codomain: &str) -> &mut Self {
        assert_eq!(domain.chars().count(), codomain.chars().count());

        for (domain, codomain) in domain.chars().zip(codomain.chars()) {
            self.table.insert(domain, SmartString::<Compact>::from(codomain));
        }

        return self;
    }

    pub fn one_multiple(&mut self, domain: char, codomain: &str) -> &mut Self {
        self.table.insert(domain, SmartString::<Compact>::from(codomain));

        return self;
    }

    #[inline(always)]
    fn one_multiple_smartstr(&mut self, domain: char, codomain: SmartString<Compact>) -> &mut Self {
        self.table.insert(domain, codomain);

        return self;
    }

    pub fn as_multiple(&mut self, trans: &[(char, &str)]) -> &mut Self {
        for (domain, codomain) in trans {
            self.table.insert(
                domain.clone(),
                SmartString::<Compact>::from(codomain.clone()),
            );
        }

        return self;
    }

    pub fn as_multiple_string(&mut self, trans: &Vec<(char, String)>) -> &mut Self {
        for (domain, codomain) in trans {
            self.table.insert(domain.clone(), SmartString::<Compact>::from(codomain));
        }

        return self;
    }

    pub fn letter_to_lowercase(&mut self, letter: char) -> &mut Self {
        self.table.insert(letter, SmartString::<Compact>::from(letter));

        let mut upper_string = letter.to_uppercase();

        if upper_string.clone().count() == 1
        {
            let uppercase_letter = upper_string.next().unwrap();
            let shifted = SmartString::<Compact>::from_iter([' ', letter]);

            self.one_multiple_smartstr(uppercase_letter, shifted);
        }

        return self;
    }

    pub fn letters_to_lowercase(&mut self, letters: &str) -> &mut Self {
        for letter in letters.chars() {
            self.letter_to_lowercase(letter);
        }

        return self;
    }

    pub fn raw(&mut self, unshift_chars: bool) -> &mut Self {
        self.is_raw = true;
        self.normalize_punct();

        return if unshift_chars
        {
            for i in 128..75000 {
                if let Some(c) = char::from_u32(i)
                {
                    if c.is_alphabetic()
                    {
                        if c.is_lowercase()
                        {
                            self.letter_to_lowercase(c);
                        } else {
                            self.keep_one(c);
                        }
                    } else if !c.is_control()
                    {
                        self.keep_one(c);
                    }
                }
            }

            self.ascii_lower()
        } else {
            for i in 0..75000 {
                if let Some(c) = char::from_u32(i)
                {
                    if !c.is_control()
                    {
                        self.keep_one(c);
                    }
                }
            }

            self
        };
    }

    pub fn custom_unshift(&mut self, upper_version: &str, lower_version: &str) -> &mut Self {
        for (upper, lower) in upper_version.chars().zip(lower_version.chars()) {
            let shifted = SmartString::<Compact>::from_iter([' ', lower]);

            self.one_multiple_smartstr(upper, shifted);
        }

        return self.keep(lower_version);
    }

    pub fn punct_lower(&mut self) -> &mut Self {
        for (upper, lower) in "{}?+_|\"<>:~".chars().zip("[]/=-\\',.;`".chars()) {
            let shifted = String::from_iter([' ', lower]);

            self.one_multiple(upper, shifted.as_str());
        }

        return self.keep("[]/=-\\',.;`");
    }

    pub fn alphabet_lower(&mut self) -> &mut Self {
        return self.letters_to_lowercase("abcdefghijklmnopqrstuvwxyz");
    }

    pub fn ascii_lower(&mut self) -> &mut Self {
        return self.punct_lower().alphabet_lower();
    }

    pub fn normalize_punct(&mut self) -> &mut Self {
        return self.one_to_one("«´»÷‘“”’–ʹ͵", "'''/''''-''").one_multiple('…', "...");
    }

    pub fn default_formatting(&mut self) -> &mut Self {
        return self.ascii_lower().normalize_punct();
    }

    pub fn language(&mut self, language: &str) -> &mut Self {
        self.default_formatting();

        return match language.to_lowercase().as_str() {
            | "akl" | "english" | "english2" | "toki_pona" | "indonesian" | "reddit" => self,
            | "albanian" => self.letters_to_lowercase("çë"),
            | "bokmal" | "nynorsk" | "danish" => self.letters_to_lowercase("åøæ"),
            | "czech" => self.as_multiple(&CZECH).letters_to_lowercase("áíě"),
            | "dan-en70-30" => self.letters_to_lowercase("åøæ"),
            | "dan-en70-30a" => self.as_multiple(&DANISH),
            | "dutch" => self.letters_to_lowercase("áèéçëíîó"),
            | "dutch_repeat" => self.letters_to_lowercase("áèéçëíîó@"),
            | "english_repeat" => self.keep("@"),
            | "english_th" => self.letters_to_lowercase("þ"),
            | "esperanto" => self.letters_to_lowercase("ŝĝĉŭĵĥ"),
            | "finnish" => self.letters_to_lowercase("åäö"),
            | "finnish_repeat" => self.letters_to_lowercase("åäö@"),
            | "french" | "french_qu" | "test" => {
                self.as_multiple(&FRENCH).letters_to_lowercase("éà")
            }
            | "german" => self.letters_to_lowercase("äöüß"),
            | "hungarian" => self.as_multiple(&HUNGARIAN).letters_to_lowercase("áéöóő"),
            | "italian" => self.as_multiple(&ITALIAN),
            | "korean" => {
                self.as_space("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ").keep("ㅣㅡㅜㅏㅊㅈㅅㅂㅁㄹㄷㄴㄱㅇㅋㅌㅍㅐㅑㅓㅕㅗㅎㅔㅛㅠ").one_to_one("ㄲㄸㅆㅃㅉㅒㅖ", "ㄱㄷㅅㅂㅈㅐㅔ").as_multiple(&KOREAN)
            }
            | "luxembourgish" => self.as_multiple(&LUXEMBOURGISH),
            | "polish" => self.as_multiple(&POLISH).letters_to_lowercase("łęż"),
            | "russian" => {
                self.letters_to_lowercase("абвгдеёжзийклмнопрстуфхцчшщъыьэюя").as_space("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ")
            }
            | "spanish" => self.as_multiple(&SPANISH),
            | "swedish" => self.letters_to_lowercase("äåö"),
            | "welsh" => self.as_multiple(&WELSH).letters_to_lowercase("ΔⳐ"),
            | "welsh_pure" => self.as_multiple(&WELSH),
            | _ => {
                panic!("This language is not available.");
            }
        };
    }

    pub fn build(&mut self) -> Translator {
        return Translator {
            is_empty: self.table.is_empty(),
            table: std::mem::take(&mut self.table),
            is_raw: self.is_raw,
        };
    }
}

const CZECH: [(char, &str); 24] = [
    ('č', "*c"),
    ('ď', "*d"),
    ('é', "*x"),
    ('ň', "*n"),
    ('ó', "*o"),
    ('ř', "*r"),
    ('š', "*s"),
    ('ť', "*t"),
    ('ů', "*u"),
    ('ú', "*b"),
    ('ý', "*y"),
    ('ž', "*z"),
    ('Č', "*c"),
    ('Ď', "*d"),
    ('É', "*x"),
    ('Ň', "*n"),
    ('Ó', "*o"),
    ('Ř', "*r"),
    ('Š', "*s"),
    ('Ť', "*t"),
    ('Ů', "*u"),
    ('Ú', "*b"),
    ('Ý', "*y"),
    ('Ž', "*z"),
];

const DANISH: [(char, &str); 3] = [('å', "*a"), ('ø', "*o"), ('æ', "*e")];

const FRENCH: [(char, &str); 39] = [
    ('ç', "*c"),
    ('Ç', "*c"),
    ('œ', "oe"),
    ('á', "* a"),
    ('â', "* a"),
    ('è', "* e"),
    ('ê', "* e"),
    ('ì', "* i"),
    ('í', "* i"),
    ('î', "* i"),
    ('ò', "* o"),
    ('ó', "* o"),
    ('ô', "* o"),
    ('ù', "* u"),
    ('ú', "* u"),
    ('û', "* u"),
    ('Á', "* a"),
    ('Â', "* a"),
    ('È', "* e"),
    ('Ê', "* e"),
    ('Ì', "* i"),
    ('Í', "* i"),
    ('Î', "* i"),
    ('Ò', "* o"),
    ('Ó', "* o"),
    ('Ô', "* o"),
    ('Ù', "* u"),
    ('Ú', "* u"),
    ('Û', "* u"),
    ('ä', "* a"),
    ('ë', "* e"),
    ('ï', "* i"),
    ('ö', "* o"),
    ('ü', "* u"),
    ('Ä', "* a"),
    ('Ë', "* e"),
    ('Ï', "* i"),
    ('Ö', "* o"),
    ('Ü', "* u"),
];

const HUNGARIAN: [(char, &str); 8] = [
    ('í', "*i"),
    ('ü', "*u"),
    ('ú', "* u"),
    ('ű', "* u"),
    ('Í', "*i"),
    ('Ü', "*u"),
    ('Ú', "* u"),
    ('Ű', "* u"),
];

const ITALIAN: [(char, &str); 10] = [
    ('à', "*a"),
    ('è', "*e"),
    ('ì', "*i"),
    ('ò', "*o"),
    ('ù', "*u"),
    ('À', "*a"),
    ('È', "*e"),
    ('Ì', "*i"),
    ('Ò', "*o"),
    ('Ù', "*u"),
];

const KOREAN: [(char, &str); 49] = [
    ('ㄳ', "ㄱㅅ"),
    ('ㅥ', "ㄴㄴ"),
    ('ㅦ', "ㄴㄷ"),
    ('ㅧ', "ㄴㅅ"),
    ('ㄵ', "ㄴㅈ"),
    ('ㄶ', "ㄴㅎ"),
    ('ㄺ', "ㄹㄱ"),
    ('ㅩ', "ㄹㄱㅅ"),
    ('ㅪ', "ㄹㄷ"),
    ('ㄻ', "ㄹㅁ"),
    ('ㄼ', "ㄹㅂ"),
    ('ㅫ', "ㄹㅂㅅ"),
    ('ㄽ', "ㄹㅅ"),
    ('ㄾ', "ㄹㅌ"),
    ('ㄿ', "ㄹㅍ"),
    ('ㅀ', "ㄹㅎ"),
    ('ㅮ', "ㅁㅂ"),
    ('ㅯ', "ㅁㅅ"),
    ('ㅲ', "ㅂㄱ"),
    ('ㅳ', "ㅂㄷ"),
    ('ㅄ', "ㅂㅅ"),
    ('ㅴ', "ㅂㅅㄱ"),
    ('ㅵ', "ㅂㅅㄷ"),
    ('ㅶ', "ㅂㅈ"),
    ('ㅷ', "ㅂㅌ"),
    ('ㅹ', "ㅂㅂ"),
    ('ㅺ', "ㅅㄱ"),
    ('ㅻ', "ㅅㄴ"),
    ('ㅼ', "ㅅㄷ"),
    ('ㅽ', "ㅅㅂ"),
    ('ㅾ', "ㅅㅈ"),
    ('ㆀ', "ㅇㅇ"),
    ('ㆄ', "ㅍ"),
    ('ㆅ', "ㅎㅎ"),
    ('ㅘ', "ㅗㅏ"),
    ('ㅙ', "ㅗㅐ"),
    ('ㅚ', "ㅗㅣ"),
    ('ㆇ', "ㅛㅑ"),
    ('ㆈ', "ㅛㅐ"),
    ('ㆉ', "ㅛㅣ"),
    ('ㅝ', "ㅜㅓ"),
    ('ㅞ', "ㅜㅔ"),
    ('ㅟ', "ㅜㅣ"),
    ('ㆊ', "ㅠㅖ"),
    ('ㆋ', "ㅠㅖ"),
    ('ㆌ', "ㅠㅣ"),
    ('ㅢ', "ㅡㅣ"),
    ('ㅸ', "ㅂ"),
    ('ㅱ', "ㅁ"),
];

const LUXEMBOURGISH: [(char, &str); 7] = [
    ('œ', " "),
    ('e', " ´"),
    ('u', " ¨"),
    ('i', " ˆ"),
    ('s', " ß"),
    ('d', " ∂"),
    ('c', " ç"),
];
const POLISH: [(char, &str); 6] = [
    ('ą', "*a"),
    ('ó', "*o"),
    ('ź', "*z"),
    ('ś', "*s"),
    ('ć', "*c"),
    ('ń', "*n"),
];

const SPANISH: [(char, &str); 14] = [
    ('á', "*a"),
    ('é', "*e"),
    ('í', "*i"),
    ('ó', "*o"),
    ('ú', "*u"),
    ('ü', "*y"),
    ('Á', "*a"),
    ('É', "*e"),
    ('Í', "*i"),
    ('Ó', "*o"),
    ('Ú', "*u"),
    ('Ü', "*y"),
    ('ñ', "*n"),
    ('Ñ', "*n"),
];

const WELSH: [(char, &str); 14] = [
    ('â', "*a"),
    ('ê', "*e"),
    ('î', "*i"),
    ('ô', "*o"),
    ('û', "*u"),
    ('ŵ', "*w"),
    ('ŷ', "*y"),
    ('Â', "*a"),
    ('Ê', "*e"),
    ('Î', "*i"),
    ('Ô', "*o"),
    ('Û', "*u"),
    ('Ŵ', "*w"),
    ('Ŷ', "*y"),
];
