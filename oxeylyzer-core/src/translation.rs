use std::collections::HashMap;

use smartstring::{
    Compact,
    LazyCompact,
    SmartString,
};

#[derive(Clone)]
pub struct Translator
{
    pub table: HashMap<char, SmartString<Compact>>,
    pub is_raw: bool,
    pub is_empty: bool,
}

impl Default for Translator
{
    fn default() -> Self
    {
        let mut translator = Translator::new();

        return translator.default_formatting().build();
    }
}

impl std::ops::Add for Translator
{
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output
    {
        self.is_raw |= rhs.is_raw;
        self.is_empty &= rhs.is_empty;

        return if !self.is_empty
        {
            let base = &SmartString::<Compact>::from(" ");

            for (from, to) in rhs.table
            {
                let original = self.table.get(&from);

                if original.is_none() || original == Some(base)
                {
                    self.table.insert(from, to);
                }
            }

            self
        }
        else
        {
            self.table = rhs.table;

            self
        };
    }
}

impl Translator
{
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> TranslatorBuilder
    {
        return TranslatorBuilder {
            table: HashMap::new(),
            is_raw: false,
        };
    }

    #[allow(dead_code)]
    pub fn language(language: &str) -> Self
    {
        return Self::new().language(language).build();
    }

    #[allow(dead_code)]
    pub fn language_or_raw(language: &str) -> Self
    {
        return Self::language(language);
    }

    pub fn raw(unshift_chars: bool) -> Self
    {
        return Translator::new().raw(unshift_chars).build();
    }

    pub fn translate(&self, s: &str) -> SmartString<LazyCompact>
    {
        let mut res = SmartString::<LazyCompact>::new();

        for c in s.chars()
        {
            match self.table.get(&c)
            {
                | Some(replacement) =>
                {
                    res.push_str(replacement);
                },
                | None =>
                {
                    res.push(' ');
                },
            }
        }

        return res;
    }
}

pub struct TranslatorBuilder
{
    table: HashMap<char, SmartString<Compact>>,
    is_raw: bool,
}

impl TranslatorBuilder
{
    pub fn as_space(&mut self, to_string: &str) -> &mut Self
    {
        for c in to_string.chars()
        {
            self.table.insert(c, SmartString::<Compact>::from(" "));
        }

        return self;
    }

    pub fn many_different_to_one(&mut self, domain: &str, codomain: char) -> &mut Self
    {
        for c in domain.chars()
        {
            self.table.insert(c, SmartString::<Compact>::from(codomain));
        }

        return self;
    }

    pub fn keep_one(&mut self, keep: char) -> &mut Self
    {
        self.table.insert(keep, SmartString::<Compact>::from(keep));

        return self;
    }

    pub fn keep(&mut self, keep: &str) -> &mut Self
    {
        for c in keep.chars()
        {
            self.table.insert(c, SmartString::<Compact>::from(c));
        }

        return self;
    }

    pub fn one_to_one(&mut self, domain: &str, codomain: &str) -> &mut Self
    {
        assert_eq!(domain.chars().count(), codomain.chars().count());

        for (domain, codomain) in domain.chars().zip(codomain.chars())
        {
            self.table
                .insert(domain, SmartString::<Compact>::from(codomain));
        }

        return self;
    }

    pub fn one_multiple(&mut self, domain: char, codomain: &str) -> &mut Self
    {
        self.table
            .insert(domain, SmartString::<Compact>::from(codomain));

        return self;
    }

    #[inline(always)]
    fn one_multiple_smartstr(&mut self, domain: char, codomain: SmartString<Compact>) -> &mut Self
    {
        self.table.insert(domain, codomain);

        return self;
    }

    pub fn as_multiple(&mut self, trans: &[(char, &str)]) -> &mut Self
    {
        for (domain, codomain) in trans
        {
            self.table.insert(
                domain.clone(),
                SmartString::<Compact>::from(codomain.clone()),
            );
        }

        return self;
    }

    pub fn as_multiple_string(&mut self, trans: &Vec<(char, String)>) -> &mut Self
    {
        for (domain, codomain) in trans
        {
            self.table
                .insert(domain.clone(), SmartString::<Compact>::from(codomain));
        }

        return self;
    }

    pub fn letter_to_lowercase(&mut self, letter: char) -> &mut Self
    {
        self.table
            .insert(letter, SmartString::<Compact>::from(letter));

        let mut upper_string = letter.to_uppercase();

        if upper_string.clone().count() == 1
        {
            let uppercase_letter = upper_string.next().unwrap();
            let shifted = SmartString::<Compact>::from_iter([' ', letter]);

            self.one_multiple_smartstr(uppercase_letter, shifted);
        }

        return self;
    }

    pub fn letters_to_lowercase(&mut self, letters: &str) -> &mut Self
    {
        for letter in letters.chars()
        {
            self.letter_to_lowercase(letter);
        }

        return self;
    }

    pub fn raw(&mut self, unshift_chars: bool) -> &mut Self
    {
        self.is_raw = true;
        self.normalize_punct();

        return if unshift_chars
        {
            for i in 128 .. 75000
            {
                if let Some(c) = char::from_u32(i)
                {
                    if c.is_alphabetic()
                    {
                        if c.is_lowercase()
                        {
                            self.letter_to_lowercase(c);
                        }
                        else
                        {
                            self.keep_one(c);
                        }
                    }
                    else if !c.is_control()
                    {
                        self.keep_one(c);
                    }
                }
            }

            self.ascii_lower()
        }
        else
        {
            for i in 0 .. 75000
            {
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

    pub fn custom_unshift(&mut self, upper_version: &str, lower_version: &str) -> &mut Self
    {
        for (upper, lower) in upper_version.chars().zip(lower_version.chars())
        {
            let shifted = SmartString::<Compact>::from_iter([' ', lower]);

            self.one_multiple_smartstr(upper, shifted);
        }

        return self.keep(lower_version);
    }

    pub fn punct_lower(&mut self) -> &mut Self
    {
        for (upper, lower) in "{}?+_|\"<>:~".chars().zip("[]/=-\\',.;`".chars())
        {
            let shifted = String::from_iter([' ', lower]);

            self.one_multiple(upper, shifted.as_str());
        }

        return self.keep("[]/=-\\',.;`");
    }

    pub fn alphabet_lower(&mut self) -> &mut Self
    {
        return self.letters_to_lowercase("abcdefghijklmnopqrstuvwxyz");
    }

    pub fn ascii_lower(&mut self) -> &mut Self
    {
        return self.punct_lower().alphabet_lower();
    }

    pub fn normalize_punct(&mut self) -> &mut Self
    {
        return self
            .one_to_one("«´»÷‘“”’–ʹ͵", "'''/''''-''")
            .one_multiple('…', "...");
    }

    pub fn default_formatting(&mut self) -> &mut Self
    {
        return self.ascii_lower().normalize_punct();
    }

    pub fn language(&mut self, language: &str) -> &mut Self
    {
        self.default_formatting();

        return match language.to_lowercase().as_str()
        {
            | "english" | "english2" => self,
            | "english_repeat" => self.keep("@"),
            | "english_th" => self.letters_to_lowercase("þ"),
            | _ =>
            {
                panic!("This language is not available.");
            },
        };
    }

    pub fn build(&mut self) -> Translator
    {
        return Translator {
            is_empty: self.table.is_empty(),
            table: std::mem::take(&mut self.table),
            is_raw: self.is_raw,
        };
    }
}
