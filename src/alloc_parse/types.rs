use crate::Dt;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub enum DateOrder {
    /// **Default & recommended** — Smart modern heuristic (best of all worlds):
    /// - Tries **Year-first** formats first (modern/tech bias: JSON, logs, APIs, databases)
    /// - Then Day-first (international/European)
    /// - Then Month-first (US fallback)
    ///
    /// This gives the best real-world success rate while remaining predictable.
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
pub enum DateParseMode {
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
    pub mode: DateParseMode,

    /// Controls ambiguous numeric dates.
    #[cfg_attr(feature = "serde", serde(default))]
    pub order: DateOrder,

    /// Sets language to use, not persistent.
    #[cfg_attr(feature = "serde", serde(default))]
    pub lang: Lang,

    /// Whether to lowercase the input:
    /// ONLY set to `false` if the &str is already lowercase.
    #[cfg_attr(feature = "serde", serde(default = "default_true"))]
    pub to_lower: bool,

    /// **Reference ("current") time** used for relative expressions:
    /// - "tomorrow", "next Friday", "in 3 days", "next week"
    /// - dates missing a year/month ("March 15", "15th of next month")
    ///
    /// - If `Some(tp)`, this `Dt` is used as "now" (overrides everything).
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
    fn default() -> Self {
        Self {
            parse: None,
            mode: DateParseMode::default(),
            order: DateOrder::default(),
            lang: Lang::default(),
            to_lower: true,
            ref_time: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DateOrderFirst {
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

/// Language codes following ISO 639-1 standard (two-letter codes)
/// Default is En (English)
#[allow(dead_code)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Lang {
    /// English (default)
    #[default]
    En,
    /// Afar
    Aa,
    /// Abkhazian
    Ab,
    /// Avestan
    Ae,
    /// Afrikaans
    Af,
    /// Akan
    Ak,
    /// Amharic
    Am,
    /// Aragonese
    An,
    /// Arabic
    Ar,
    /// Assamese
    As,
    /// Avaric
    Av,
    /// Aymara
    Ay,
    /// Azerbaijani
    Az,
    /// Bashkir
    Ba,
    /// Belarusian
    Be,
    /// Bulgarian
    Bg,
    /// Bihari languages
    Bh,
    /// Bislama
    Bi,
    /// Bambara
    Bm,
    /// Bengali
    Bn,
    /// Tibetan
    Bo,
    /// Breton
    Br,
    /// Bosnian
    Bs,
    /// Catalan
    Ca,
    /// Chechen
    Ce,
    /// Chamorro
    Ch,
    /// Corsican
    Co,
    /// Cree
    Cr,
    /// Czech
    Cs,
    /// Church Slavic
    Cu,
    /// Chuvash
    Cv,
    /// Welsh
    Cy,
    /// Danish
    Da,
    /// German
    De,
    /// Divehi
    Dv,
    /// Dzongkha
    Dz,
    /// Ewe
    Ee,
    /// Greek
    El,
    /// Esperanto
    Eo,
    /// Spanish
    Es,
    /// Estonian
    Et,
    /// Basque
    Eu,
    /// Persian
    Fa,
    /// Fulah
    Ff,
    /// Finnish
    Fi,
    /// Fijian
    Fj,
    /// Faroese
    Fo,
    /// French
    Fr,
    /// Western Frisian
    Fy,
    /// Irish
    Ga,
    /// Scottish Gaelic
    Gd,
    /// Galician
    Gl,
    /// Guarani
    Gn,
    /// Gujarati
    Gu,
    /// Manx
    Gv,
    /// Hausa
    Ha,
    /// Hebrew
    He,
    /// Hindi
    Hi,
    /// Hiri Motu
    Ho,
    /// Croatian
    Hr,
    /// Haitian Creole
    Ht,
    /// Hungarian
    Hu,
    /// Armenian
    Hy,
    /// Herero
    Hz,
    /// Interlingua
    Ia,
    /// Indonesian
    Id,
    /// Interlingue
    Ie,
    /// Igbo
    Ig,
    /// Sichuan Yi
    Ii,
    /// Inupiaq
    Ik,
    /// Ido
    Io,
    /// Icelandic
    Is,
    /// Italian
    It,
    /// Inuktitut
    Iu,
    /// Japanese
    Ja,
    /// Javanese
    Jv,
    /// Georgian
    Ka,
    /// Kongo
    Kg,
    /// Kikuyu
    Ki,
    /// Kuanyama
    Kj,
    /// Kazakh
    Kk,
    /// Greenlandic
    Kl,
    /// Khmer
    Km,
    /// Kannada
    Kn,
    /// Korean
    Ko,
    /// Kanuri
    Kr,
    /// Kashmiri
    Ks,
    /// Kurdish
    Ku,
    /// Komi
    Kv,
    /// Cornish
    Kw,
    /// Kyrgyz
    Ky,
    /// Latin
    La,
    /// Luxembourgish
    Lb,
    /// Ganda
    Lg,
    /// Limburgish
    Li,
    /// Lingala
    Ln,
    /// Lao
    Lo,
    /// Lithuanian
    Lt,
    /// Luba-Katanga
    Lu,
    /// Latvian
    Lv,
    /// Malagasy
    Mg,
    /// Marshallese
    Mh,
    /// Maori
    Mi,
    /// Macedonian
    Mk,
    /// Malayalam
    Ml,
    /// Mongolian
    Mn,
    /// Marathi
    Mr,
    /// Malay
    Ms,
    /// Maltese
    Mt,
    /// Burmese
    My,
    /// Nauru
    Na,
    /// Norwegian Bokmål
    Nb,
    /// North Ndebele
    Nd,
    /// Nepali
    Ne,
    /// Ndonga
    Ng,
    /// Dutch
    Nl,
    /// Norwegian Nynorsk
    Nn,
    /// Norwegian
    No,
    /// South Ndebele
    Nr,
    /// Navajo
    Nv,
    /// Chichewa
    Ny,
    /// Occitan
    Oc,
    /// Ojibwa
    Oj,
    /// Oromo
    Om,
    /// Oriya
    Or,
    /// Ossetian
    Os,
    /// Punjabi
    Pa,
    /// Pali
    Pi,
    /// Polish
    Pl,
    /// Pashto
    Ps,
    /// Portuguese
    Pt,
    /// Quechua
    Qu,
    /// Romansh
    Rm,
    /// Rundi
    Rn,
    /// Romanian
    Ro,
    /// Russian
    Ru,
    /// Kinyarwanda
    Rw,
    /// Sanskrit
    Sa,
    /// Sardinian
    Sc,
    /// Sindhi
    Sd,
    /// Northern Sami
    Se,
    /// Sango
    Sg,
    /// Sinhala
    Si,
    /// Slovak
    Sk,
    /// Slovenian
    Sl,
    /// Samoan
    Sm,
    /// Shona
    Sn,
    /// Somali
    So,
    /// Albanian
    Sq,
    /// Serbian
    Sr,
    /// Swati
    Ss,
    /// Southern Sotho
    St,
    /// Sundanese
    Su,
    /// Swedish
    Sv,
    /// Swahili
    Sw,
    /// Tamil
    Ta,
    /// Telugu
    Te,
    /// Tajik
    Tg,
    /// Thai
    Th,
    /// Tigrinya
    Ti,
    /// Turkmen
    Tk,
    /// Tagalog
    Tl,
    /// Tswana
    Tn,
    /// Tonga
    To,
    /// Turkish
    Tr,
    /// Tsonga
    Ts,
    /// Tatar
    Tt,
    /// Twi
    Tw,
    /// Tahitian
    Ty,
    /// Uyghur
    Ug,
    /// Ukrainian
    Uk,
    /// Urdu
    Ur,
    /// Uzbek
    Uz,
    /// Venda
    Ve,
    /// Vietnamese
    Vi,
    /// Volapük
    Vo,
    /// Walloon
    Wa,
    /// Wolof
    Wo,
    /// Xhosa
    Xh,
    /// Yiddish
    Yi,
    /// Yoruba
    Yo,
    /// Zhuang
    Za,
    /// Chinese
    Zh,
    /// Zulu
    Zu,
}

impl alloc::fmt::Display for Lang {
    fn fmt(&self, f: &mut alloc::fmt::Formatter<'_>) -> alloc::fmt::Result {
        write!(f, "{:?}", self)
    }
}
