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
        /// -c: 1, 2, 3, 4
        cmd rank {
           optional -a, --asc order: bool
           optional -c, --columns columns: String
        }

        /// Shows the top n same finger bigrams in a layout.
        cmd sfbs {
            required name: String
            optional -c, --count count: usize
        }

        /// Shows the top n same finger trigrams in a layout.
        cmd sfts {
            required name: String
            optional -c, --count count: usize
        }

        /// Gives information about a certain n-gram.
        /// For bigrams, skipgram info will be provided.
        cmd ngram n occ freq {
            required ngram: String
        }

        /// Quit.
        cmd quit q exit {}
    }
}
