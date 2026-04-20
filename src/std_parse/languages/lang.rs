use crate::DateToken;
use aho_corasick::AhoCorasick;
use std::{collections::HashMap, sync::LazyLock};

#[derive(Debug, Clone, Copy)]
pub(crate) struct LangData {
    /// Master lookup table: short/long term → (canonical form, token)
    pub map: &'static LazyLock<HashMap<&'static str, (&'static str, DateToken)>>,
    /// Aho-Corasick for full date parsing
    pub date_ac: &'static LazyLock<AhoCorasick>,
    /// Aho-Corasick that only contains relatives + durations
    pub duration_ac: &'static LazyLock<AhoCorasick>,
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

impl std::fmt::Display for Lang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
