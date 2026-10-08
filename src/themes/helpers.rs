use std::sync::OnceLock;

use super::types::{ATELIER, VSCODE};
use crate::theme::{Appearance, Theme};

/// Every theme, atelier's first, then each family in the order above.
pub fn all() -> &'static [Theme] {
    static THEMES: OnceLock<Vec<Theme>> = OnceLock::new();
    THEMES.get_or_init(|| {
        // The files ship with the crate, so one that does not load is a build mistake a test catches.
        let atelier = ATELIER.iter().map(|json| {
            crate::theme_file::parse(json)
                .expect("a bundled atelier theme loads")
                .0
        });
        let vscode = VSCODE.iter().map(|b| {
            crate::theme_import::import(b.json, b.name, b.family)
                .expect("a bundled theme imports")
                .theme
        });
        atelier.chain(vscode).collect()
    })
}

/// The theme called `name`, case aside.
/// A name saved when atelier was named lathe ("lathe Dark") finds atelier's theme of that appearance.
pub fn named(name: &str) -> Option<&'static Theme> {
    let renamed = name
        .get(..6)
        .filter(|head| head.eq_ignore_ascii_case("lathe "))
        .map(|_| format!("atelier {}", &name[6..]));
    let name = renamed.as_deref().unwrap_or(name);
    all().iter().find(|t| t.name.eq_ignore_ascii_case(name))
}

/// atelier in `appearance`, the default.
pub fn atelier(appearance: Appearance) -> &'static Theme {
    all()
        .iter()
        .find(|t| t.family.as_ref() == "atelier" && t.appearance == appearance)
        .expect("atelier has both appearances")
}

/// The families in order, each with its themes, for a picker.
pub fn families() -> Vec<(&'static str, Vec<&'static Theme>)> {
    let mut out: Vec<(&'static str, Vec<&'static Theme>)> = Vec::new();
    for theme in all() {
        match out
            .iter_mut()
            .find(|(family, _)| *family == theme.family.as_ref())
        {
            Some((_, list)) => list.push(theme),
            None => out.push((theme.family.as_ref(), vec![theme])),
        }
    }
    out
}
