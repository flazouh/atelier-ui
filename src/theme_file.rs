//! atelier's own theme files (`assets/themes/atelier-*.json`), and the tokens every theme resolves to.
//!
//! A theme names its page, ink, surfaces, muted text, accent and status colours, and its syntax
//! palette. What it leaves out is derived here, by one set of rules shared with the VS Code importer
//! ([`crate::theme_import`]), and the names of the derived tokens are kept so a test and
//! `docs/themes.md` can list them.

mod helpers;
mod structs;

pub use helpers::{appearance, hex, parse};
pub use structs::Tokens;

#[cfg(test)]
use gpui_kit::Hsla;

#[cfg(test)]
mod tests;
