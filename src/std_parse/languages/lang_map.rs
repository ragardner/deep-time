use crate::Lang;
use crate::LangData;
use crate::std_parse::en::*;
use crate::std_parse::multi_lang::*;
use std::collections::HashMap;
use std::sync::LazyLock;

#[allow(dead_code)]
pub(crate) static LANG_MAP: LazyLock<HashMap<Lang, &'static LangData>> = LazyLock::new(|| {
    let mut m = HashMap::new();

    m.insert(Lang::En, &EN_LANG_DATA);

    #[cfg(feature = "fr")]
    {
        m.insert(Lang::Fr, &FR_LANG_DATA);
    }

    m
});
