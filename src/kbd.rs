//! A keyboard shortcut hint, such as `⌘↵`.
//!
//! Geist has no key symbols, so a font fallback drew `⌘` and `↵` at another size and on another
//! baseline than the letters beside them. Each symbol is a Material Symbols icon instead, sized and centred like
//! the letters, as Zed draws its key hints. Letters and names such as `Esc` stay text.

use gpui_kit::{App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div, };
use crate::scale::px;

use crate::{
    icon::{Icon, IconName},
    theme::{ActiveTheme, radius},
    typography::MONO_FONT_FAMILY,
};

/// One piece of a hint: a key symbol drawn as an icon, or plain text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyPart {
    Symbol(IconName),
    Text(String),
}

/// The icon that stands for a key symbol, if the character is one.
fn symbol(c: char) -> Option<IconName> {
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

#[derive(IntoElement)]
pub struct Kbd {
    keys: SharedString,
    /// Drawn on a filled control in its ink, instead of on the page.
    ink: Option<gpui_kit::Hsla>,
    /// Drawn on a coloured fill: the fill and the text on it.
    on: Option<(gpui_kit::Hsla, gpui_kit::Hsla)>,
}

impl Kbd {
    /// `keys` as the user reads them, such as "⌘↵", "⇧Tab" or "Ctrl ⌫". A chord as it is written ("⌘⇧g")
    /// goes through [`crate::keys::cap`], so off the Mac it draws Control whoever passes it.
    pub fn new(keys: impl Into<SharedString>) -> Self {
        Self { keys: crate::keys::cap(&keys.into()), ink: None, on: None }
    }

    /// For a cap inside a button: a wash of the button's `ink` instead of the page's card fill, so it
    /// reads on a primary fill as on a ghost one.
    pub fn ink(mut self, ink: gpui_kit::Hsla) -> Self {
        self.ink = Some(ink);
        self
    }
}

impl Kbd {
    /// For a cap on a coloured `fill` (a primary button): the button's `text`, on a patch that is a tone of the fill
    /// and keeps 3:1 with the text (see [`crate::theme::cap_patch`]).
    pub fn on(mut self, fill: gpui_kit::Hsla, text: gpui_kit::Hsla) -> Self {
        self.on = Some((fill, text));
        self
    }
}

impl RenderOnce for Kbd {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let (fill, color) = match (self.on, self.ink) {
            (Some((fill, text)), _) => (crate::theme::cap_patch(fill, text), text),
            (None, Some(ink)) => (ink.opacity(0.14), ink.opacity(0.75)),
            (None, None) => (theme.card_strong, theme.muted_foreground),
        };
        div()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(2.))
            .h(px(18.))
            .px(px(5.))
            .rounded(radius::md() - px(2.))
            .bg(fill)
            .text_color(color)
            .font_family(MONO_FONT_FAMILY)
            .text_size(px(11.))
            .line_height(px(16.))
            .children(parts(&self.keys).into_iter().map(|part| match part {
                // 11px sits with the 11px letters: the icon's box matches their cap height closely.
                KeyPart::Symbol(icon) => Icon::new(icon).size(px(11.)).color(color).into_any_element(),
                KeyPart::Text(text) => div().child(text).into_any_element(),
            }))
    }
}

#[cfg(test)]
mod tests;
