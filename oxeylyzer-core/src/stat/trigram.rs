use crate::{
    hand::finger::Finger,
    hand::hand::Hand::*,
    language::trigram_patterns::TrigramPattern,
    language::trigram_patterns::TrigramPattern::*,
};

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Trigram<T>(pub T, pub T, pub T);

impl std::fmt::Display for Trigram<Finger> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}, {}, {}", self.0, self.1, self.2);
    }
}

impl Trigram<Finger> {
    pub const fn new(f0: Finger, f1: Finger, f2: Finger) -> Self {
        return Self {
            0: f0,
            1: f1,
            2: f2,
        };
    }

    const fn is_alternate(&self) -> bool {
        return matches!(
            (self.0.hand(), self.1.hand(), self.2.hand()),
            (Left, Right, Left) | (Right, Left, Right)
        );
    }

    const fn is_same_finger_skipgram(&self) -> bool {
        return self.0.eq(self.2);
    }

    const fn get_alternate(&self) -> TrigramPattern {
        return match self.is_same_finger_skipgram() {
            | true => AlternateSfs,
            | false => Alternate,
        };
    }

    const fn is_roll(&self) -> bool {
        return match (self.0.hand(), self.1.hand(), self.2.hand()) {
            | (Left, Left, Right) => true,
            | (Right, Left, Left) => true,
            | (Right, Right, Left) => true,
            | (Left, Right, Right) => true,
            | _ => false,
        };
    }

    const fn is_inroll(&self) -> bool {
        return match (self.0.hand(), self.1.hand(), self.2.hand()) {
            | (Left, Left, Right) => self.0.lt(self.1),
            | (Right, Left, Left) => self.1.lt(self.2),
            | (Right, Right, Left) => self.0.gt(self.1),
            | (Left, Right, Right) => self.1.gt(self.2),
            | _ => unreachable!(),
        };
    }

    const fn get_roll(&self) -> TrigramPattern {
        return match self.is_inroll() {
            | true => Inroll,
            | false => Outroll,
        };
    }

    const fn on_one_hand(&self) -> bool {
        return matches!(
            (self.0.hand(), self.1.hand(), self.2.hand()),
            (Left, Left, Left) | (Right, Right, Right)
        );
    }

    const fn is_redirect(&self) -> bool {
        return (self.0.lt(self.1) == self.1.gt(self.2)) && self.on_one_hand();
    }

    const fn is_bad_redirect(&self) -> bool {
        return self.is_redirect() && self.0.is_bad() && self.1.is_bad() && self.2.is_bad();
    }

    const fn has_same_finger_bigram(&self) -> bool {
        return self.0.eq(self.1) || self.1.eq(self.2);
    }

    const fn is_same_finger_trigarm(&self) -> bool {
        return self.0.eq(self.1) && self.1.eq(self.2);
    }

    const fn get_one_hand(&self) -> TrigramPattern {
        return if self.is_same_finger_trigarm()
        {
            Sft
        } else if self.has_same_finger_bigram()
        {
            BadSfb
        } else if self.is_redirect()
        {
            match (self.is_same_finger_skipgram(), self.is_bad_redirect()) {
                | (false, false) => Redirect,
                | (false, true) => BadRedirect,
                | (true, false) => RedirectSfs,
                | (true, true) => BadRedirectSfs,
            }
        } else {
            Onehand
        };
    }

    pub const fn get_trigram_pattern(&self) -> TrigramPattern {
        return if self.is_alternate()
        {
            self.get_alternate()
        } else if self.on_one_hand()
        {
            self.get_one_hand()
        } else if self.has_same_finger_bigram()
        {
            Sfb
        } else if self.is_roll()
        {
            self.get_roll()
        } else {
            Other
        };
    }
}

impl From<&[char]> for Trigram<char> {
    fn from(value: &[char]) -> Self {
        return Self {
            0: value[0],
            1: value[1],
            2: value[2],
        };
    }
}

impl From<&[u8]> for Trigram<u8> {
    fn from(value: &[u8]) -> Self {
        return Self {
            0: value[0],
            1: value[1],
            2: value[2],
        };
    }
}
