use crate::{
    icon::{IconName},
    };
use super::types::KeyPart;

/// The icon that stands for a key symbol, if the character is one.
pub(super) fn symbol(c: char) -> Option<IconName> {
    Some(match c {
        '⌘' => IconName::Command,
        '⇧' => IconName::Shift,
        '⌥' => IconName::Option,
        '⌃' => IconName::Control,
        '↵' | '⏎' => IconName::Return,
        '⌫' => IconName::Backspace,
        '⇥' => IconName::Tab,
        '←' => IconName::ArrowBack,
        '→' => IconName::ArrowForward,
        '↑' => IconName::ArrowUp,
        '↓' => IconName::ArrowDown,
        _ => return None,
    })
}

/// `keys` split into symbols and runs of text, in order. Spaces between parts are dropped: the parts
/// are spaced by the layout.
pub fn parts(keys: &str) -> Vec<KeyPart> {
    let mut parts = Vec::new();
    let mut text = String::new();
    let flush = |text: &mut String, parts: &mut Vec<KeyPart>| {
        let run = text.trim();
        if !run.is_empty() {
            parts.push(KeyPart::Text(run.to_string()));
        }
        text.clear();
    };
    for c in keys.chars() {
        match symbol(c) {
            Some(icon) => {
                flush(&mut text, &mut parts);
                parts.push(KeyPart::Symbol(icon));
            }
            None if c == ' ' => flush(&mut text, &mut parts),
            None => text.push(c),
        }
    }
    flush(&mut text, &mut parts);
    parts
}
