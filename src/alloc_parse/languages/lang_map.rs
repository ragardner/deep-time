use crate::en_lang_data;
use crate::{Lang, LangData};
use alloc::boxed::Box;
use hashbrown::HashMap;
use once_cell::race::OnceBox;

#[allow(unused_imports)]
use crate::alloc_parse::multi_lang::*;

static LANG_MAP: OnceBox<HashMap<Lang, &'static LangData>> = OnceBox::new();
pub(crate) fn lang_map() -> &'static HashMap<Lang, &'static LangData> {
    LANG_MAP.get_or_init(|| {
        let mut m = HashMap::new();

        m.insert(Lang::En, en_lang_data());

        #[cfg(feature = "fr")]
        {
            m.insert(Lang::Fr, fr::fr_lang_data());
        }

        Box::new(m)
    })
}
