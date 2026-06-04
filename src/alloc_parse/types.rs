use crate::Dt;
use alloc::string::String;
use alloc::vec::Vec;

/// Used by [`ParseCfg`] in
/// [`Dt::from_str_parse`](../struct.Dt.html#method.from_str_parse).
///
/// Controls how ambiguous numeric dates (e.g. `01/02/03`).
///
/// The default `Smart` variant applies a practical heuristic that prefers
/// year-first for compact formats and uses numeric plausibility checks
/// for other cases. The other variants force a specific ordering.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub enum Order {
    /// Heuristic for **mixed data**. Uses the following rules, in this order:
    ///
    /// 1. **Pure-numeric compact formats** (≥ 6 digits with no separators,
    ///    e.g. `240314153045`, `20240315`, `YYMMDDHHMMSS`):
    ///    treated as **Year-first** (`%Y%m%d` / `%y%m%d`).
    ///    These are overwhelmingly used in logs, filenames, databases, APIs,
    ///    configs, and JSON for sortability.
    ///
    /// 2. **Delimited formats that start with a plausible 4-digit year**
    ///    (1900–2100): treated as **Year-first**.
    ///
    /// 3. **Numeric plausibility check** (strongest universal signal):
    ///    - First number is 13–31 → **Day-first** (international/European style).
    ///    - First number is 1–12 **and** second number is 13–31 → **Month-first**
    ///      (US style).
    ///
    /// 4. **Strong ISO 8601 / timestamp markers** (`T` connector, `Z`, numeric
    ///    offsets, or IANA timezone names) → **Year-first**.
    ///
    /// 5. **Fallback**:
    ///    - With the `locale` feature enabled: respects the system locale
    ///      preference (Day-first in most of the world).
    ///    - Without the `locale` feature: **Day-first** (global majority).
    ///
    /// The `/` separator is deliberately ignored in the plausibility step
    /// because it is culturally ambiguous.
    ///
    /// Once the preferred ordering is determined, the parser tries the
    /// corresponding ambiguous candidate formats (Year-first → Day-first →
    /// Month-first, or the reverse, depending on the detected order) and falls
    /// back gracefully.
    #[default]
    Smart,
    /// Force **Year-first** only (YYYY/MM/DD or YY/MM/DD)
    Year,
    /// Force **Day-first** only (DD/MM/YYYY)
    Day,
    /// Force **Month-first** only (MM/DD/YYYY)
    Month,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
/// Only relevant for purely numeric dates.
pub enum Mode {
    /// **Default mode** — Smart heuristic:
    /// - 5/7-digit pure-numeric inside `LEGACY_ORDINAL_YEAR_RANGE` → treated as business ordinal (YYYYDDD / YYDDD)
    /// - Outside that range or invalid ordinal → treated as MJD or JD
    #[default]
    Auto,
    /// When combined with a provided Vec of formats in parse no other formats are tried.
    Explicit,
    /// It's some sort of unix timestamp
    UnixTimestamp,
    /// Business/legacy-only mode:
    /// Only accepts ordinal dates (YYYYDDD / YYDDD). No astronomy (JD/MJD) support.
    /// Strict and predictable for ERP/mainframe data.
    Legacy,
    /// Scientific / astronomy-first mode:
    /// Prioritizes MJD (5-digit) and JD (7-digit). Ordinals are only fallback.
    /// Use this when parsing data from astronomy tools or large numeric epochs.
    Scientific,
}

/// Configuration options for
/// [`Dt::from_str_parse`](../struct.Dt.html#method.from_str_parse).
///
/// Controls language, ambiguous date order, numeric parsing mode,
/// explicit `strptime` formats, relative-date support, and reference time.
///
/// These settings will not persist between parse calls and have to be used
/// as an arg every time you want them.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[derive(Clone, Debug, PartialEq)]
pub struct ParseCfg {
    /// Explicit list of formats to try **in the exact order given**.
    ///
    /// If this is provided and the vec is non-empty and the mode is Explicit
    /// then only these formats are tried and `mode` and `order` are ignored.
    ///
    /// If the mode is not Explicit then after trying the formats in parse the
    /// rest of the parser will continue as normal, using `mode` and `order`.
    ///
    /// Example:
    /// ```js
    /// parse: ["%Y-%m-%d", "%d/%m/%Y", "%m/%d/%Y", "%d.%m.%Y"]
    /// ```
    #[cfg_attr(feature = "serde", serde(default))]
    pub parse: Option<Vec<String>>,

    /// Controls which preset format sets are used (astronomy/scientific formats,
    /// legacy business rules, etc.).
    #[cfg_attr(feature = "serde", serde(default))]
    pub mode: Mode,

    /// Controls ambiguous numeric dates.
    #[cfg_attr(feature = "serde", serde(default))]
    pub order: Order,

    /// Sets language to use for a particular parse call.
    #[cfg_attr(feature = "serde", serde(default))]
    pub lang: Lang,

    /// Whether to lowercase the input:
    /// ONLY set to `false` if the &str is already lowercase.
    #[cfg_attr(feature = "serde", serde(default = "default_true"))]
    pub to_lower: bool,

    /// Whether to parse relative dates as well as normal dates.
    #[cfg_attr(feature = "serde", serde(default = "default_true"))]
    pub relative: bool,

    /// **Reference ("current") time** used for relative expressions:
    /// - "tomorrow", "next Friday", "in 3 days", "next week"
    /// - If `Some`, this `Dt` is used as "now" (overrides everything).
    /// - If `None` + `std` feature enabled: automatically uses real system time.
    /// - If `None` + no `std`: parsing relative dates will fail with a clear error.
    #[cfg_attr(feature = "serde", serde(default))]
    pub ref_time: Option<Dt>,
}

#[cfg(feature = "serde")]
fn default_true() -> bool {
    true
}

impl Default for ParseCfg {
    fn default() -> ParseCfg {
        Self {
            parse: None,
            mode: Mode::default(),
            order: Order::default(),
            lang: Lang::default(),
            to_lower: true,
            relative: true,
            ref_time: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OrderFirst {
    /// Year-Month-Day ordering (ISO 8601 style, `YYYY-MM-DD`, `20240315`, etc.)
    Year,
    /// Month-Day-Year ordering (US / some English locales, `MM/DD/YYYY`)
    Month,
    /// Day-Month-Year ordering (most of the world, `DD/MM/YYYY`, `DD.MM.YYYY`)
    Day,
}

#[derive(Clone)]
pub(crate) struct AmBuilder {
    pub pieces: Vec<&'static str>,
    pub seen_year: bool,
    pub seen_month: bool,
    pub seen_day: bool,
}

#[inline]
pub(crate) fn append_to_all(builders: &mut Vec<AmBuilder>, s: &'static str) {
    for b in builders {
        b.pieces.push(s);
    }
}

/// Language codes following ISO 639-1 standard (two-letter codes).
/// Default is En (English)
#[allow(dead_code)]
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Lang {
    /// English (default) — always available
    #[default]
    En,

    // /// Afar
    // #[cfg(feature = "aa")]
    // Aa,
    // /// Abkhazian
    // #[cfg(feature = "ab")]
    // Ab,
    // /// Avestan
    // #[cfg(feature = "ae")]
    // Ae,
    // /// Afrikaans
    // #[cfg(feature = "af")]
    // Af,
    // /// Akan
    // #[cfg(feature = "ak")]
    // Ak,
    // /// Amharic
    // #[cfg(feature = "am")]
    // Am,
    // /// Aragonese
    // #[cfg(feature = "an")]
    // An,
    // /// Arabic
    // #[cfg(feature = "ar")]
    // Ar,
    // /// Assamese
    // #[cfg(feature = "as")]
    // As,
    // /// Avaric
    // #[cfg(feature = "av")]
    // Av,
    // /// Aymara
    // #[cfg(feature = "ay")]
    // Ay,
    // /// Azerbaijani
    // #[cfg(feature = "az")]
    // Az,
    // /// Bashkir
    // #[cfg(feature = "ba")]
    // Ba,
    // /// Belarusian
    // #[cfg(feature = "be")]
    // Be,
    // /// Bulgarian
    // #[cfg(feature = "bg")]
    // Bg,
    // /// Bihari languages
    // #[cfg(feature = "bh")]
    // Bh,
    // /// Bislama
    // #[cfg(feature = "bi")]
    // Bi,
    // /// Bambara
    // #[cfg(feature = "bm")]
    // Bm,
    // /// Bengali
    // #[cfg(feature = "bn")]
    // Bn,
    // /// Tibetan
    // #[cfg(feature = "bo")]
    // Bo,
    // /// Breton
    // #[cfg(feature = "br")]
    // Br,
    // /// Bosnian
    // #[cfg(feature = "bs")]
    // Bs,
    // /// Catalan
    // #[cfg(feature = "ca")]
    // Ca,
    // /// Chechen
    // #[cfg(feature = "ce")]
    // Ce,
    // /// Chamorro
    // #[cfg(feature = "ch")]
    // Ch,
    // /// Corsican
    // #[cfg(feature = "co")]
    // Co,
    // /// Cree
    // #[cfg(feature = "cr")]
    // Cr,
    // /// Czech
    // #[cfg(feature = "cs")]
    // Cs,
    // /// Church Slavic
    // #[cfg(feature = "cu")]
    // Cu,
    // /// Chuvash
    // #[cfg(feature = "cv")]
    // Cv,
    // /// Welsh
    // #[cfg(feature = "cy")]
    // Cy,
    // /// Danish
    // #[cfg(feature = "da")]
    // Da,
    /// German
    #[cfg(feature = "de")]
    De,
    // /// Divehi
    // #[cfg(feature = "dv")]
    // Dv,
    // /// Dzongkha
    // #[cfg(feature = "dz")]
    // Dz,
    // /// Ewe
    // #[cfg(feature = "ee")]
    // Ee,
    // /// Greek
    // #[cfg(feature = "el")]
    // El,
    // /// Esperanto
    // #[cfg(feature = "eo")]
    // Eo,
    /// Spanish
    #[cfg(feature = "es")]
    Es,
    // /// Estonian
    // #[cfg(feature = "et")]
    // Et,
    // /// Basque
    // #[cfg(feature = "eu")]
    // Eu,
    // /// Persian
    // #[cfg(feature = "fa")]
    // Fa,
    // /// Fulah
    // #[cfg(feature = "ff")]
    // Ff,
    // /// Finnish
    // #[cfg(feature = "fi")]
    // Fi,
    // /// Fijian
    // #[cfg(feature = "fj")]
    // Fj,
    // /// Faroese
    // #[cfg(feature = "fo")]
    // Fo,
    /// French
    #[cfg(feature = "fr")]
    Fr,
    // /// Western Frisian
    // #[cfg(feature = "fy")]
    // Fy,
    // /// Irish
    // #[cfg(feature = "ga")]
    // Ga,
    // /// Scottish Gaelic
    // #[cfg(feature = "gd")]
    // Gd,
    // /// Galician
    // #[cfg(feature = "gl")]
    // Gl,
    // /// Guarani
    // #[cfg(feature = "gn")]
    // Gn,
    // /// Gujarati
    // #[cfg(feature = "gu")]
    // Gu,
    // /// Manx
    // #[cfg(feature = "gv")]
    // Gv,
    // /// Hausa
    // #[cfg(feature = "ha")]
    // Ha,
    // /// Hebrew
    // #[cfg(feature = "he")]
    // He,
    // /// Hindi
    // #[cfg(feature = "hi")]
    // Hi,
    // /// Hiri Motu
    // #[cfg(feature = "ho")]
    // Ho,
    // /// Croatian
    // #[cfg(feature = "hr")]
    // Hr,
    // /// Haitian Creole
    // #[cfg(feature = "ht")]
    // Ht,
    // /// Hungarian
    // #[cfg(feature = "hu")]
    // Hu,
    // /// Armenian
    // #[cfg(feature = "hy")]
    // Hy,
    // /// Herero
    // #[cfg(feature = "hz")]
    // Hz,
    // /// Interlingua
    // #[cfg(feature = "ia")]
    // Ia,
    // /// Indonesian
    // #[cfg(feature = "id")]
    // Id,
    // /// Interlingue
    // #[cfg(feature = "ie")]
    // Ie,
    // /// Igbo
    // #[cfg(feature = "ig")]
    // Ig,
    // /// Sichuan Yi
    // #[cfg(feature = "ii")]
    // Ii,
    // /// Inupiaq
    // #[cfg(feature = "ik")]
    // Ik,
    // /// Ido
    // #[cfg(feature = "io")]
    // Io,
    // /// Icelandic
    // #[cfg(feature = "is")]
    // Is,
    // /// Italian
    // #[cfg(feature = "it")]
    // It,
    // /// Inuktitut
    // #[cfg(feature = "iu")]
    // Iu,
    // /// Japanese
    // #[cfg(feature = "ja")]
    // Ja,
    // /// Javanese
    // #[cfg(feature = "jv")]
    // Jv,
    // /// Georgian
    // #[cfg(feature = "ka")]
    // Ka,
    // /// Kongo
    // #[cfg(feature = "kg")]
    // Kg,
    // /// Kikuyu
    // #[cfg(feature = "ki")]
    // Ki,
    // /// Kuanyama
    // #[cfg(feature = "kj")]
    // Kj,
    // /// Kazakh
    // #[cfg(feature = "kk")]
    // Kk,
    // /// Greenlandic
    // #[cfg(feature = "kl")]
    // Kl,
    // /// Khmer
    // #[cfg(feature = "km")]
    // Km,
    // /// Kannada
    // #[cfg(feature = "kn")]
    // Kn,
    // /// Korean
    // #[cfg(feature = "ko")]
    // Ko,
    // /// Kanuri
    // #[cfg(feature = "kr")]
    // Kr,
    // /// Kashmiri
    // #[cfg(feature = "ks")]
    // Ks,
    // /// Kurdish
    // #[cfg(feature = "ku")]
    // Ku,
    // /// Komi
    // #[cfg(feature = "kv")]
    // Kv,
    // /// Cornish
    // #[cfg(feature = "kw")]
    // Kw,
    // /// Kyrgyz
    // #[cfg(feature = "ky")]
    // Ky,
    // /// Latin
    // #[cfg(feature = "la")]
    // La,
    // /// Luxembourgish
    // #[cfg(feature = "lb")]
    // Lb,
    // /// Ganda
    // #[cfg(feature = "lg")]
    // Lg,
    // /// Limburgish
    // #[cfg(feature = "li")]
    // Li,
    // /// Lingala
    // #[cfg(feature = "ln")]
    // Ln,
    // /// Lao
    // #[cfg(feature = "lo")]
    // Lo,
    // /// Lithuanian
    // #[cfg(feature = "lt")]
    // Lt,
    // /// Luba-Katanga
    // #[cfg(feature = "lu")]
    // Lu,
    // /// Latvian
    // #[cfg(feature = "lv")]
    // Lv,
    // /// Malagasy
    // #[cfg(feature = "mg")]
    // Mg,
    // /// Marshallese
    // #[cfg(feature = "mh")]
    // Mh,
    // /// Maori
    // #[cfg(feature = "mi")]
    // Mi,
    // /// Macedonian
    // #[cfg(feature = "mk")]
    // Mk,
    // /// Malayalam
    // #[cfg(feature = "ml")]
    // Ml,
    // /// Mongolian
    // #[cfg(feature = "mn")]
    // Mn,
    // /// Marathi
    // #[cfg(feature = "mr")]
    // Mr,
    // /// Malay
    // #[cfg(feature = "ms")]
    // Ms,
    // /// Maltese
    // #[cfg(feature = "mt")]
    // Mt,
    // /// Burmese
    // #[cfg(feature = "my")]
    // My,
    // /// Nauru
    // #[cfg(feature = "na")]
    // Na,
    // /// Norwegian Bokmål
    // #[cfg(feature = "nb")]
    // Nb,
    // /// North Ndebele
    // #[cfg(feature = "nd")]
    // Nd,
    // /// Nepali
    // #[cfg(feature = "ne")]
    // Ne,
    // /// Ndonga
    // #[cfg(feature = "ng")]
    // Ng,
    // /// Dutch
    // #[cfg(feature = "nl")]
    // Nl,
    // /// Norwegian Nynorsk
    // #[cfg(feature = "nn")]
    // Nn,
    // /// Norwegian
    // #[cfg(feature = "no")]
    // No,
    // /// South Ndebele
    // #[cfg(feature = "nr")]
    // Nr,
    // /// Navajo
    // #[cfg(feature = "nv")]
    // Nv,
    // /// Chichewa
    // #[cfg(feature = "ny")]
    // Ny,
    // /// Occitan
    // #[cfg(feature = "oc")]
    // Oc,
    // /// Ojibwa
    // #[cfg(feature = "oj")]
    // Oj,
    // /// Oromo
    // #[cfg(feature = "om")]
    // Om,
    // /// Oriya
    // #[cfg(feature = "or")]
    // Or,
    // /// Ossetian
    // #[cfg(feature = "os")]
    // Os,
    // /// Punjabi
    // #[cfg(feature = "pa")]
    // Pa,
    // /// Pali
    // #[cfg(feature = "pi")]
    // Pi,
    // /// Polish
    // #[cfg(feature = "pl")]
    // Pl,
    // /// Pashto
    // #[cfg(feature = "ps")]
    // Ps,
    // /// Portuguese
    // #[cfg(feature = "pt")]
    // Pt,
    // /// Quechua
    // #[cfg(feature = "qu")]
    // Qu,
    // /// Romansh
    // #[cfg(feature = "rm")]
    // Rm,
    // /// Rundi
    // #[cfg(feature = "rn")]
    // Rn,
    // /// Romanian
    // #[cfg(feature = "ro")]
    // Ro,
    // /// Russian
    // #[cfg(feature = "ru")]
    // Ru,
    // /// Kinyarwanda
    // #[cfg(feature = "rw")]
    // Rw,
    // /// Sanskrit
    // #[cfg(feature = "sa")]
    // Sa,
    // /// Sardinian
    // #[cfg(feature = "sc")]
    // Sc,
    // /// Sindhi
    // #[cfg(feature = "sd")]
    // Sd,
    // /// Northern Sami
    // #[cfg(feature = "se")]
    // Se,
    // /// Sango
    // #[cfg(feature = "sg")]
    // Sg,
    // /// Sinhala
    // #[cfg(feature = "si")]
    // Si,
    // /// Slovak
    // #[cfg(feature = "sk")]
    // Sk,
    // /// Slovenian
    // #[cfg(feature = "sl")]
    // Sl,
    // /// Samoan
    // #[cfg(feature = "sm")]
    // Sm,
    // /// Shona
    // #[cfg(feature = "sn")]
    // Sn,
    // /// Somali
    // #[cfg(feature = "so")]
    // So,
    // /// Albanian
    // #[cfg(feature = "sq")]
    // Sq,
    // /// Serbian
    // #[cfg(feature = "sr")]
    // Sr,
    // /// Swati
    // #[cfg(feature = "ss")]
    // Ss,
    // /// Southern Sotho
    // #[cfg(feature = "st")]
    // St,
    // /// Sundanese
    // #[cfg(feature = "su")]
    // Su,
    // /// Swedish
    // #[cfg(feature = "sv")]
    // Sv,
    // /// Swahili
    // #[cfg(feature = "sw")]
    // Sw,
    // /// Tamil
    // #[cfg(feature = "ta")]
    // Ta,
    // /// Telugu
    // #[cfg(feature = "te")]
    // Te,
    // /// Tajik
    // #[cfg(feature = "tg")]
    // Tg,
    // /// Thai
    // #[cfg(feature = "th")]
    // Th,
    // /// Tigrinya
    // #[cfg(feature = "ti")]
    // Ti,
    // /// Turkmen
    // #[cfg(feature = "tk")]
    // Tk,
    // /// Tagalog
    // #[cfg(feature = "tl")]
    // Tl,
    // /// Tswana
    // #[cfg(feature = "tn")]
    // Tn,
    // /// Tonga
    // #[cfg(feature = "to")]
    // To,
    // /// Turkish
    // #[cfg(feature = "tr")]
    // Tr,
    // /// Tsonga
    // #[cfg(feature = "ts")]
    // Ts,
    // /// Tatar
    // #[cfg(feature = "tt")]
    // Tt,
    // /// Twi
    // #[cfg(feature = "tw")]
    // Tw,
    // /// Tahitian
    // #[cfg(feature = "ty")]
    // Ty,
    // /// Uyghur
    // #[cfg(feature = "ug")]
    // Ug,
    // /// Ukrainian
    // #[cfg(feature = "uk")]
    // Uk,
    // /// Urdu
    // #[cfg(feature = "ur")]
    // Ur,
    // /// Uzbek
    // #[cfg(feature = "uz")]
    // Uz,
    // /// Venda
    // #[cfg(feature = "ve")]
    // Ve,
    // /// Vietnamese
    // #[cfg(feature = "vi")]
    // Vi,
    // /// Volapük
    // #[cfg(feature = "vo")]
    // Vo,
    // /// Walloon
    // #[cfg(feature = "wa")]
    // Wa,
    // /// Wolof
    // #[cfg(feature = "wo")]
    // Wo,
    // /// Xhosa
    // #[cfg(feature = "xh")]
    // Xh,
    // /// Yiddish
    // #[cfg(feature = "yi")]
    // Yi,
    // /// Yoruba
    // #[cfg(feature = "yo")]
    // Yo,
    // /// Zhuang
    // #[cfg(feature = "za")]
    // Za,
    // /// Chinese
    // #[cfg(feature = "zh")]
    // Zh,
    // /// Zulu
    // #[cfg(feature = "zu")]
    // Zu,
}

impl alloc::fmt::Display for Lang {
    fn fmt(&self, f: &mut alloc::fmt::Formatter<'_>) -> alloc::fmt::Result {
        write!(f, "{:?}", self)
    }
}
