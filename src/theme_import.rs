//! A VS Code colour theme (`colors` and `tokenColors`) as a atelier [`Theme`](crate::theme::Theme). The mapping is in
//! `docs/themes.md`; each UI token reads the first of its keys the theme has, and one it lacks is
//! derived ([`Tokens::resolve`](crate::theme_file::Tokens::resolve)). Text colours that fail WCAG AA on the page or a card are moved toward
//! the ink until they pass, and the import says which.

mod helpers;
mod structs;
mod types;

pub use helpers::import;
pub use structs::Imported;
pub use types::{SYNTAX_SCOPES, UI_KEYS};

#[cfg(test)]
use helpers::rule_for;

#[cfg(test)]
use crate::{
    theme::{Appearance, TEXT_CONTRAST, contrast},
    theme_file::hex,
};
#[cfg(test)]
use serde_json::Value;

#[cfg(test)]
mod tests;
