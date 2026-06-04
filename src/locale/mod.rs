pub mod en;
use en::*;

#[cfg(feature = "parse")]
pub mod lang_data;
#[cfg(feature = "parse")]
pub mod lang_map;

#[cfg(feature = "multi-lang")]
pub mod multi_lang;
#[cfg(feature = "multi-lang")]
use multi_lang::*;

pub(crate) struct FormatNames {
    pub weekdays_full: &'static [&'static [u8]; 7],
    pub weekdays_abbr: &'static [&'static [u8]; 7],
    pub months_full: &'static [&'static [u8]; 12],
    pub months_abbr: &'static [&'static [u8]; 12],
}

impl Lang {
    #[inline]
    pub(crate) const fn names(self) -> &'static FormatNames {
        match self {
            Lang::En => &EN_NAMES,
            #[cfg(feature = "fr")]
            Lang::Fr => &FR_NAMES,
            #[cfg(feature = "de")]
            Lang::De => &DE_NAMES,
            #[cfg(feature = "es")]
            Lang::Es => &ES_NAMES,
        }
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

#[cfg(feature = "alloc")]
impl alloc::fmt::Display for Lang {
    fn fmt(&self, f: &mut alloc::fmt::Formatter<'_>) -> alloc::fmt::Result {
        write!(f, "{:?}", self)
    }
}
