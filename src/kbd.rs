//! A keyboard shortcut hint, such as `⌘↵`.
//!
//! Geist has no key symbols, so a font fallback drew `⌘` and `↵` at another size and on another
//! baseline than the letters beside them. Each symbol is a Hugeicons icon instead, sized and centred like
//! the letters, as Zed draws its key hints. Letters and names such as `Esc` stay text.

mod helpers;
mod structs;
mod types;

pub use helpers::parts;
pub use structs::Kbd;
pub use types::KeyPart;

#[cfg(test)]
use crate::icon::IconName;

#[cfg(test)]
mod tests;
