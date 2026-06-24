//! Internationalization — string resolution (`tr`/`tr_args`) + the `Lang`
//! enum. The dictionaries live in per-language files; this module
//! dispatches and implements fallback (PtBr → key itself).
//!
//! ## Anti-footgun (BUG-7)
//!
//! `tr` and `tr_args` **require** `Lang`. There is no convenience function
//! with a hardcoded default — every call site must pass the current
//! language explicitly so a language switch re-renders all strings.

mod en;
mod es;
mod pt_br;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lang {
    #[default]
    PtBr,
    En,
    Es,
}

impl Lang {
    pub const ALL: [Lang; 3] = [Lang::PtBr, Lang::En, Lang::Es];

    pub fn code(self) -> &'static str {
        match self {
            Lang::PtBr => "pt-BR",
            Lang::En => "en",
            Lang::Es => "es",
        }
    }

    pub fn short(self) -> &'static str {
        match self {
            Lang::PtBr => "PT",
            Lang::En => "EN",
            Lang::Es => "ES",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Lang::PtBr => "Português",
            Lang::En => "English",
            Lang::Es => "Español",
        }
    }

    pub fn date_locale(self) -> &'static str {
        match self {
            Lang::PtBr => "pt-BR",
            Lang::En => "en-US",
            Lang::Es => "es-ES",
        }
    }
}

fn dict_for(lang: Lang) -> &'static [(&'static str, &'static str)] {
    match lang {
        Lang::PtBr => pt_br::dict(),
        Lang::En => en::dict(),
        Lang::Es => es::dict(),
    }
}

pub fn tr(key: &str, lang: Lang) -> &str {
    for &(k, v) in dict_for(lang) {
        if k == key {
            return v;
        }
    }
    if lang != Lang::PtBr {
        for &(k, v) in dict_for(Lang::PtBr) {
            if k == key {
                return v;
            }
        }
    }
    key
}

pub fn tr_args(key: &str, lang: Lang, vars: &[(&str, &str)]) -> String {
    let template = tr(key, lang);
    let mut result = template.to_string();
    for &(name, value) in vars {
        let placeholder = format!("{{{}}}", name);
        result = result.replace(&placeholder, value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lang_default_is_ptbr() {
        assert_eq!(Lang::default(), Lang::PtBr);
    }

    #[test]
    fn lang_codes() {
        assert_eq!(Lang::PtBr.code(), "pt-BR");
        assert_eq!(Lang::En.code(), "en");
        assert_eq!(Lang::Es.code(), "es");
    }

    #[test]
    fn lang_short() {
        assert_eq!(Lang::PtBr.short(), "PT");
        assert_eq!(Lang::En.short(), "EN");
        assert_eq!(Lang::Es.short(), "ES");
    }

    #[test]
    fn lang_label() {
        assert_eq!(Lang::PtBr.label(), "Português");
        assert_eq!(Lang::En.label(), "English");
        assert_eq!(Lang::Es.label(), "Español");
    }

    #[test]
    fn lang_date_locale() {
        assert_eq!(Lang::PtBr.date_locale(), "pt-BR");
        assert_eq!(Lang::En.date_locale(), "en-US");
        assert_eq!(Lang::Es.date_locale(), "es-ES");
    }

    #[test]
    fn lang_all_contains_three() {
        assert_eq!(Lang::ALL.len(), 3);
    }

    #[test]
    fn tr_resolves_in_each_language() {
        assert_eq!(tr("topbar.states.connected", Lang::En), "Connected");
        assert_eq!(tr("topbar.states.connected", Lang::Es), "Conectado");
        assert_eq!(tr("sidebar.connect", Lang::PtBr), "Conectar");
        assert_eq!(tr("tabs.tools", Lang::En), "Tools");
        assert_eq!(tr("tabs.tools", Lang::PtBr), "Tools");
        assert_eq!(tr("tabs.tools", Lang::Es), "Tools");
    }

    #[test]
    fn tr_fallback_to_ptbr() {
        assert_eq!(tr("common.copy", Lang::PtBr), "Copiar");
        assert_eq!(tr("common.copy", Lang::En), "Copy");
        assert_eq!(tr("common.copy", Lang::Es), "Copiar");
    }

    #[test]
    fn tr_unknown_key_returns_key_itself() {
        let key = "nonexistent.module.key";
        assert_eq!(tr(key, Lang::PtBr), key);
        assert_eq!(tr(key, Lang::En), key);
        assert_eq!(tr(key, Lang::Es), key);
    }

    #[test]
    fn tr_args_interpolates() {
        let result = tr_args("conn.connected", Lang::En, &[("name", "srv"), ("ms", "12")]);
        assert_eq!(result, "connected to srv (12ms)");
    }

    #[test]
    fn tr_args_missing_var_keeps_placeholder() {
        let result = tr_args("conn.connected", Lang::En, &[("name", "srv")]);
        assert_eq!(result, "connected to srv ({ms}ms)");
    }

    #[test]
    fn tr_args_interpolates_ptbr() {
        let result = tr_args(
            "conn.connected",
            Lang::PtBr,
            &[("name", "srv"), ("ms", "12")],
        );
        assert_eq!(result, "conectado a srv (12ms)");
    }

    #[test]
    fn tr_args_interpolates_es() {
        let result = tr_args("conn.connected", Lang::Es, &[("name", "srv"), ("ms", "12")]);
        assert_eq!(result, "conectado a srv (12ms)");
    }

    #[test]
    fn all_languages_have_same_key_set() {
        let keys_pt = dict_for(Lang::PtBr)
            .iter()
            .map(|&(k, _)| k)
            .collect::<Vec<_>>();
        let keys_en = dict_for(Lang::En)
            .iter()
            .map(|&(k, _)| k)
            .collect::<Vec<_>>();
        let keys_es = dict_for(Lang::Es)
            .iter()
            .map(|&(k, _)| k)
            .collect::<Vec<_>>();

        let set_pt: std::collections::HashSet<_> = keys_pt.iter().copied().collect();
        let set_en: std::collections::HashSet<_> = keys_en.iter().copied().collect();
        let set_es: std::collections::HashSet<_> = keys_es.iter().copied().collect();

        let in_pt_not_en: Vec<_> = set_pt.difference(&set_en).copied().collect();
        let in_pt_not_es: Vec<_> = set_pt.difference(&set_es).copied().collect();
        let in_en_not_pt: Vec<_> = set_en.difference(&set_pt).copied().collect();
        let in_es_not_pt: Vec<_> = set_es.difference(&set_pt).copied().collect();

        assert!(
            in_pt_not_en.is_empty(),
            "keys in pt-BR but not en: {in_pt_not_en:?}"
        );
        assert!(
            in_pt_not_es.is_empty(),
            "keys in pt-BR but not es: {in_pt_not_es:?}"
        );
        assert!(
            in_en_not_pt.is_empty(),
            "keys in en but not pt-BR: {in_en_not_pt:?}"
        );
        assert!(
            in_es_not_pt.is_empty(),
            "keys in es but not pt-BR: {in_es_not_pt:?}"
        );
    }
}
