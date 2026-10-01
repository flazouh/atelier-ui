use gpui_kit::{Global, Keystroke};

use super::types::{Profile, Waiting};

/// The profile in force for the whole app, and a sequence half pressed. One keyboard, so one of each.
#[derive(Clone, Debug, Default)]
pub struct Keys {
    pub profile: Profile,
    pub(super) waiting: Waiting,
}

impl Global for Keys {}

/// One key press as the table reads it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Press {
    /// The key as the reader means it: `s`, `T`, `/`, `Escape`. With Shift and no other modifier this is
    /// the character Shift made.
    pub key: String,
    /// Command on macOS, Control elsewhere.
    pub secondary: bool,
    pub alt: bool,
    pub shift: bool,
}

impl Press {
    pub fn from_keystroke(k: &Keystroke) -> Self {
        let m = &k.modifiers;
        let secondary = m.secondary();
        let key = match (k.key.as_str(), &k.key_char) {
            ("escape", _) => "Escape".to_string(),
            ("enter", _) => "Enter".to_string(),
            (_, Some(ch)) if !secondary && !m.alt && ch.chars().count() == 1 => ch.clone(),
            (key, _) => key.to_string(),
        };
        Self { key, secondary, alt: m.alt, shift: m.shift }
    }
}
