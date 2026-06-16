use crate::DIGIT_CHARS;
use aho_corasick::AhoCorasick;
use aho_corasick::FindIter;
use core::ops::Range;

#[derive(Debug)]
pub struct SplitKeepWithPos<'a> {
    haystack: &'a str,
    finder: FindIter<'a, 'a>,
    last: usize,
    /// Next Aho-Corasick match we've already pulled but haven't processed yet.
    pending: Option<(usize, usize)>,
    /// Cached next item that `next()` will return (enables `peek()`).
    peeked: Option<(&'a str, Range<usize>)>,
}

impl<'a> SplitKeepWithPos<'a> {
    #[inline]
    pub fn new(ac: &'a AhoCorasick, haystack: &'a str) -> Self {
        Self {
            haystack,
            finder: ac.find_iter(haystack),
            last: 0,
            pending: None,
            peeked: None,
        }
    }

    /// Peek at the next item without advancing the iterator.
    ///
    /// Returns `Some(&item)` if there is a next item, or `None` if exhausted.
    /// Repeated calls return the same value until `next()` is called.
    pub fn peek(&mut self) -> Option<&<Self as Iterator>::Item> {
        if self.peeked.is_none() {
            self.peeked = self.advance();
        }
        self.peeked.as_ref()
    }

    /// Core splitting logic. Called by both `peek()` and `next()`.
    fn advance(&mut self) -> Option<(&'a str, Range<usize>)> {
        if self.last >= self.haystack.len() {
            return None;
        }

        // 1. Handle a pending match we already pulled from Aho-Corasick
        if let Some((mstart, mend)) = self.pending.take() {
            if mstart > self.last {
                // Yield gap first, keep the match pending
                let gap = &self.haystack[self.last..mstart];
                let range = self.last..mstart;
                self.last = mstart;
                self.pending = Some((mstart, mend));
                return Some((gap, range));
            } else {
                // Yield the match itself
                self.last = mend;
                return Some((&self.haystack[mstart..mend], mstart..mend));
            }
        }

        // 2. Pull next match from Aho-Corasick
        if let Some(m) = self.finder.next() {
            self.pending = Some((m.start(), m.end()));
            return self.advance(); // single level of recursion (depth = 1)
        }

        // 3. Final tail segment
        let start = self.last;
        let end = self.haystack.len();
        self.last = end;
        Some((&self.haystack[start..end], start..end))
    }
}

impl<'a> Iterator for SplitKeepWithPos<'a> {
    type Item = (&'a str, Range<usize>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.peeked.take() {
            return Some(item);
        }
        self.advance()
    }
}

impl<'a> core::iter::FusedIterator for SplitKeepWithPos<'a> {}

macro_rules! define_ends_with_methods {
    ($($method_name:ident => $byte:literal),* $(,)?) => {
        pub(crate) trait EndsWithExt {
            $(fn $method_name(&self) -> bool;)*
            fn ends_with_ascii_digit(&self) -> bool;
        }

        impl EndsWithExt for str {
            $(#[inline]
            fn $method_name(&self) -> bool {
                self.as_bytes().last() == Some(&$byte)
            })*

            #[inline]
            fn ends_with_ascii_digit(&self) -> bool {
                matches!(self.as_bytes().last(), Some(b'0'..=b'9'))
                // or: self.as_bytes().last().copied().map_or(false, u8::is_ascii_digit)
            }
        }
    };
}

define_ends_with_methods! {
    ends_with_space         => b' ',
    ends_with_dot           => b'.',
    ends_with_comma         => b',',
    ends_with_slash         => b'/',
    ends_with_colon         => b':',
    ends_with_plus          => b'+',
    ends_with_minus         => b'-',
    ends_with_lbracket      => b'[',
    ends_with_rbracket      => b']',
}

/// Converts common Unicode decimal digit (ASCII + full-width + others)
/// to the corresponding ASCII digit string.
pub(crate) fn to_ascii_digit(ch: char) -> Option<char> {
    if let Some(d) = ch.to_digit(10) {
        return Some(DIGIT_CHARS[d as usize]);
    }
    let codepoint = ch as u32;
    let idx = match codepoint {
        0xFF10..=0xFF19 => codepoint - 0xFF10, // ０-９ Full-width (Japan, China, Korea, Taiwan, Hong Kong)
        0x0660..=0x0669 => codepoint - 0x0660, // ٠-٩ Arabic-Indic
        0x06F0..=0x06F9 => codepoint - 0x06F0, // ۰-۹ Eastern Arabic / Persian / Urdu
        0x0966..=0x096F => codepoint - 0x0966, // ०-९ Devanagari (Hindi, Nepali, Marathi…)
        0x09E6..=0x09EF => codepoint - 0x09E6, // ০-৯ Bengali (Bangla, Assamese)
        0x0A66..=0x0A6F => codepoint - 0x0A66, // ੦-੯ Gurmukhi (Punjabi)
        0x0AE6..=0x0AEF => codepoint - 0x0AE6, // ૦-૯ Gujarati
        0x0B66..=0x0B6F => codepoint - 0x0B66, // ୦-୯ Oriya / Odia
        0x0BE6..=0x0BEF => codepoint - 0x0BE6, // ௦-௯ Tamil
        0x0C66..=0x0C6F => codepoint - 0x0C66, // ౦-౯ Telugu
        0x0CE6..=0x0CEF => codepoint - 0x0CE6, // ೦-೯ Kannada
        0x0D66..=0x0D6F => codepoint - 0x0D66, // ൦-൯ Malayalam
        0x0DE6..=0x0DEF => codepoint - 0x0DE6, // ෦-෯ Sinhala (Sri Lanka)
        0x0E50..=0x0E59 => codepoint - 0x0E50, // ๐-๙ Thai
        0x0ED0..=0x0ED9 => codepoint - 0x0ED0, // ໐-໙ Lao
        0x1040..=0x1049 => codepoint - 0x1040, // ၀-၉ Myanmar (Burmese)
        0x17E0..=0x17E9 => codepoint - 0x17E0, // ០-៩ Khmer (Cambodian)
        0x07C0..=0x07C9 => codepoint - 0x07C0, // ߀-߉ N'Ko (Guinea, West Africa)
        0x0F20..=0x0F29 => codepoint - 0x0F20, // ༠-༩ Tibetan
        0x1810..=0x1819 => codepoint - 0x1810, // ᠐-᠙ Mongolian
        0x19D0..=0x19D9 => codepoint - 0x19D0, // ᧐-᧙ New Tai Lue
        0x1A80..=0x1A89 => codepoint - 0x1A80, // ᪀-᪉ Tai Tham (low)
        0x1A90..=0x1A99 => codepoint - 0x1A90, // ᪐-᪙ Tai Tham (high)
        0x1B50..=0x1B59 => codepoint - 0x1B50, // ᭐-᭙ Balinese
        0x1BB0..=0x1BB9 => codepoint - 0x1BB0, // ᮰-᮹ Sundanese
        0xA9D0..=0xA9D9 => codepoint - 0xA9D0, // ꧐-꧙ Javanese
        0xAA50..=0xAA59 => codepoint - 0xAA50, // ꩐-꩙ Cham
        0xABF0..=0xABF9 => codepoint - 0xABF0, // ꯰-꯹ Meetei Mayek (Manipuri)
        _ => return None,
    };

    Some(DIGIT_CHARS[idx as usize])
}
