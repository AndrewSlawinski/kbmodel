#[derive(Clone, Default)]
pub struct TrigramStats {
    pub alternates: f64,
    pub alternates_same_finger_skipgrams: f64,

    pub inrolls: f64,
    pub outrolls: f64,

    pub one_hands: f64,

    pub redirects: f64,
    pub redirects_same_finger_skipgrams: f64,

    pub bad_redirects: f64,
    pub bad_redirects_same_finger_skipgrams: f64,

    pub same_finger_bigrams: f64,
    pub bad_same_finger_bigrams: f64,

    pub same_finger_trigrams: f64,

    pub other: f64,
    pub invalid: f64,
}

impl std::fmt::Display for TrigramStats {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            formatter,
            "Inrolls: {:.3}%\n\
			Outrolls: {:.3}%\n\
			Total Rolls: {:.3}%\n\
			Onehands: {:.3}%\n\n\
			Alternates: {:.3}%\n\
			Alternates (sfs): {:.3}%\n\
			Total Alternates: {:.3}%\n\n\
			Redirects: {:.3}%\n\
			Redirects Sfs: {:.3}%\n\
			Bad Redirects: {:.3}%\n\
			Bad Redirects Sfs: {:.3}%\n\
			Total Redirects: {:.3}%\n\n\
			Bad Sfbs: {:.3}%\n\
			Sft: {:.3}%\n",
            self.inrolls,
            self.outrolls,
            self.inrolls + self.outrolls,
            self.one_hands,
            self.alternates,
            self.alternates_same_finger_skipgrams,
            self.alternates + self.alternates_same_finger_skipgrams,
            self.redirects,
            self.redirects_same_finger_skipgrams,
            self.bad_redirects,
            self.bad_redirects_same_finger_skipgrams,
            self.redirects + self.redirects_same_finger_skipgrams + self.bad_redirects + self.bad_redirects_same_finger_skipgrams,
            self.bad_same_finger_bigrams,
            self.same_finger_trigrams
        );
    }
}
