use std::path::PathBuf;

use xflags::xflags;

xflags! {
    cmd repl {
        /// Analyze a layout. Specify a number to select a layout generated in this session.
        cmd analyze a view layout {
            required name_or_number: String
        }

        /// Compare two layouts.
        cmd compare c comp cmp {
            required name1: String
            required name2: String
        }

        /// Rank all layouts for the loaded language.
        cmd rank {}

        /// Generate n layouts and display the top 10.
        cmd generate gen g {
            required count: usize
        }

        /// Improves a layout while pinning the keys specified in `config.toml`.
        cmd improve i optimize {
            required name: String
            required count: usize
        }

        /// Saves the nth generated layout.
        ///
        /// `<nth> [ -n <name> ]`
        cmd save s {
            required nth: usize
            optional name: String
        }

        /// Shows the top n same finger bigrams in a layout.
        ///
        /// [ n || 10 ]
        cmd sfbs {
            required name: String
            optional -c, --count count: usize
        }

        /// Set a language to be used for analysis. Tries to load corpus when not present.
        cmd language l lang {
            optional language: PathBuf
        }

        /// Include layouts stored under a different language.
        cmd include {
            required language: String
        }

        /// Lists all currently available languages.
        cmd languages langs {}

        /// Loads a corpus for a certain language.
        cmd load {
            required language: PathBuf
            optional -a, --all
            optional -r, --raw
        }

        /// Gives information about a certain n-gram.
        ///
        /// For bigrams, skipgram info will be provided.
        cmd ngram n occ freq {
            required ngram: String
        }

        /// Refreshes the config and default characters for the analyzer, retaining generated layouts.
        cmd reload r {}

        /// Quit.
        cmd quit q exit {}
    }
}
