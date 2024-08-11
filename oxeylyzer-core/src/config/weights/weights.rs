use crate::config::weights::alternates::Alternates;
use crate::config::weights::bigrams::Bigrams;
use crate::config::weights::fingers::Fingers;
use crate::config::weights::redirects::Redirects;
use crate::config::weights::rolls::Rolls;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Weights {
    pub alternates: Alternates,
    pub bigrams: Bigrams,
    pub redirects: Redirects,
    pub rolls: Rolls,
    pub fingers: Fingers,
}

impl Default for Weights {
    fn default() -> Self {
        return Self {
            alternates: Default::default(),
            bigrams: Default::default(),
            redirects: Default::default(),
            rolls: Default::default(),
            fingers: Default::default(),
        };
    }
}
