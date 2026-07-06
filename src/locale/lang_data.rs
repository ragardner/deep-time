use crate::Token;
use aho_corasick::AhoCorasick;
use hashbrown::HashMap;

#[derive(Debug, Clone)]
pub(crate) struct LangData {
    /// Master lookup table: short/long term → (canonical form, token)
    pub map: &'static HashMap<&'static str, (&'static str, Token)>,
    /// Aho-Corasick for full date parsing
    pub date_ac: &'static AhoCorasick,
    /// Aho-Corasick that only contains relatives + durations
    pub duration_ac: &'static AhoCorasick,
    /// Language specific decimals separator
    pub decimal_char: char,
}
