//! Every theme atelier offers, loaded once from the data files in `assets/themes`: atelier's own
//! ([`crate::theme_file`]) and VS Code themes ([`crate::theme_import`]), their sources in
//! `assets/themes/vscode/SOURCES.md`.

use std::sync::OnceLock;

use crate::theme::{Appearance, Theme};

/// A VS Code theme file bundled with atelier-ui: its name, family and text.
struct Bundled {
    name: &'static str,
    family: &'static str,
    json: &'static str,
}

const VSCODE: &[Bundled] = &[
    Bundled { name: "Cursor Dark", family: "Cursor", json: include_str!("../assets/themes/vscode/cursor-dark.json") },
    Bundled { name: "Cursor Light", family: "Cursor", json: include_str!("../assets/themes/vscode/cursor-light.json") },
    Bundled { name: "GitHub Light", family: "GitHub", json: include_str!("../assets/themes/vscode/github-light.json") },
    Bundled { name: "GitHub Dark", family: "GitHub", json: include_str!("../assets/themes/vscode/github-dark.json") },
    Bundled { name: "Catppuccin Latte", family: "Catppuccin", json: include_str!("../assets/themes/vscode/catppuccin-latte.json") },
    Bundled { name: "Catppuccin Frappé", family: "Catppuccin", json: include_str!("../assets/themes/vscode/catppuccin-frappe.json") },
    Bundled { name: "Catppuccin Macchiato", family: "Catppuccin", json: include_str!("../assets/themes/vscode/catppuccin-macchiato.json") },
    Bundled { name: "Catppuccin Mocha", family: "Catppuccin", json: include_str!("../assets/themes/vscode/catppuccin-mocha.json") },
];

const ATELIER: [&str; 2] = [include_str!("../assets/themes/atelier-light.json"), include_str!("../assets/themes/atelier-dark.json")];

/// Every theme, atelier's first, then each family in the order above.
pub fn all() -> &'static [Theme] {
    static THEMES: OnceLock<Vec<Theme>> = OnceLock::new();
    THEMES.get_or_init(|| {
        // The files ship with the crate, so one that does not load is a build mistake a test catches.
        let atelier = ATELIER.iter().map(|json| crate::theme_file::parse(json).expect("a bundled atelier theme loads").0);
        let vscode = VSCODE.iter().map(|b| crate::theme_import::import(b.json, b.name, b.family).expect("a bundled theme imports").theme);
        atelier.chain(vscode).collect()
    })
}

/// The theme called `name`, case aside.
/// A name saved when atelier was named lathe ("lathe Dark") finds atelier's theme of that appearance.
pub fn named(name: &str) -> Option<&'static Theme> {
    let renamed = name.get(..6).filter(|head| head.eq_ignore_ascii_case("lathe ")).map(|_| format!("atelier {}", &name[6..]));
    let name = renamed.as_deref().unwrap_or(name);
    all().iter().find(|t| t.name.eq_ignore_ascii_case(name))
}

/// atelier in `appearance`, the default.
pub fn atelier(appearance: Appearance) -> &'static Theme {
    all().iter().find(|t| t.family.as_ref() == "atelier" && t.appearance == appearance).expect("atelier has both appearances")
}

/// The families in order, each with its themes, for a picker.
pub fn families() -> Vec<(&'static str, Vec<&'static Theme>)> {
    let mut out: Vec<(&'static str, Vec<&'static Theme>)> = Vec::new();
    for theme in all() {
        match out.iter_mut().find(|(family, _)| *family == theme.family.as_ref()) {
            Some((_, list)) => list.push(theme),
            None => out.push((theme.family.as_ref(), vec![theme])),
        }
    }
    out
}

#[cfg(test)]
mod tests;
