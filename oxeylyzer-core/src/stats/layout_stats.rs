use crate::language::language_data::LanguageData;
use crate::layout::layout::Layout;
use crate::stats::bigram_stats::BigramStats;

#[derive(Default, Clone)]
pub struct LayoutStats
{
    pub bigram_stats: BigramStats,
}

impl LayoutStats
{
    pub fn new(language_data: &LanguageData, layout: &Layout) -> Self
    {
        return Self {
            bigram_stats: BigramStats::new(language_data, &layout.matrix),
        };
    }
}
