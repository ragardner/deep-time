use crate::en::en_lang_data;
use crate::{Lang, LangData};
use alloc::boxed::Box;
use hashbrown::HashMap;
use once_cell::race::OnceBox;

#[cfg(feature = "multi-lang")]
use super::multi_lang::*;

static LANG_MAP: OnceBox<HashMap<Lang, &'static LangData>> = OnceBox::new();
pub(crate) fn lang_map() -> &'static HashMap<Lang, &'static LangData> {
    LANG_MAP.get_or_init(|| {
        let mut m = HashMap::new();

        m.insert(Lang::En, en_lang_data());

        #[cfg(feature = "de")]
        {
            m.insert(Lang::De, de_alloc::de_lang_data());
        }

        #[cfg(feature = "es")]
        {
            m.insert(Lang::Es, es_alloc::es_lang_data());
        }

        #[cfg(feature = "fr")]
        {
            m.insert(Lang::Fr, fr_alloc::fr_lang_data());
        }

        Box::new(m)
    })
}
