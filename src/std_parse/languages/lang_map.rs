use crate::std_parse::en::*;
use crate::{Lang, LangData};
use std::collections::HashMap;
use std::sync::LazyLock;

#[allow(unused_imports)]
use crate::std_parse::multi_lang::*;

pub(crate) static LANG_MAP: LazyLock<HashMap<Lang, &'static LangData>> = LazyLock::new(|| {
    let mut m = HashMap::new();

    m.insert(Lang::En, &EN_LANG_DATA);

    #[cfg(feature = "fr")]
    {
        m.insert(Lang::Fr, &fr::FR_LANG_DATA);
    }

    m
});
