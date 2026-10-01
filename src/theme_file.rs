//! atelier's own theme files (`assets/themes/atelier-*.json`), and the tokens every theme resolves to.
//!
//! A theme names its page, ink, surfaces, muted text, accent and status colours, and its syntax
//! palette. What it leaves out is derived here, by one set of rules shared with the VS Code importer
//! ([`crate::theme_import`]), and the names of the derived tokens are kept so a test and
//! `docs/themes.md` can list them.

use std::sync::Arc;

use gpui_kit::{Hsla, Rgba, SharedString, component::highlighter::HighlightTheme};
use serde::Deserialize;

use crate::theme::{Appearance, Theme, mix};

/// A colour from `#RGB`, `#RGBA`, `#RRGGBB` or `#RRGGBBAA`.
pub fn hex(text: &str) -> Option<Hsla> {
    let digits = text.trim().strip_prefix('#')?;
    let value = |range: std::ops::Range<usize>| u8::from_str_radix(digits.get(range)?, 16).ok();
    let (r, g, b, a) = match digits.len() {
        3 | 4 => {
            let short = |i: usize| value(i..i + 1).map(|v| v * 17);
            (short(0)?, short(1)?, short(2)?, if digits.len() == 4 { short(3)? } else { 255 })
        }
        6 | 8 => (value(0..2)?, value(2..4)?, value(4..6)?, if digits.len() == 8 { value(6..8)? } else { 255 }),
        _ => return None,
    };
    let unit = |v: u8| v as f32 / 255.;
    Some(Rgba { r: unit(r), g: unit(g), b: unit(b), a: unit(a) }.into())
}

/// A theme's tokens as its file gives them; `None` for one it leaves to [`Tokens::resolve`].
#[derive(Clone, Debug, Default)]
pub struct Tokens {
    pub page: Option<Hsla>,
    pub ink: Option<Hsla>,
    pub card: Option<Hsla>,
    pub card_strong: Option<Hsla>,
    pub muted: Option<Hsla>,
    pub divider: Option<Hsla>,
    pub accent: Option<Hsla>,
    pub info: Option<Hsla>,
    pub danger: Option<Hsla>,
    pub success: Option<Hsla>,
    pub warning: Option<Hsla>,
    pub warning_fill: Option<Hsla>,
    pub selection: Option<Hsla>,
    pub popover: Option<Hsla>,
    pub shadow: Option<Hsla>,
    pub diff_added: Option<Hsla>,
    pub diff_removed: Option<Hsla>,
    /// A button's arrow chip at rest and hovered, and its arrow at rest.
    pub chip_rest: Option<Hsla>,
    pub chip_hover: Option<Hsla>,
    pub chip_arrow: Option<Hsla>,
    /// Running, done, failed, pending, cancelled.
    pub status: Option<[Hsla; 5]>,
}

impl Tokens {
    /// The theme, every token set: each missing one derived from its nearest given one. Also the
    /// names of the derived tokens, in the order of the fields.
    pub fn resolve(
        self,
        name: SharedString,
        family: SharedString,
        appearance: Appearance,
        syntax: Arc<HighlightTheme>,
    ) -> (Theme, Vec<&'static str>) {
        let mut derived = Vec::new();
        let mut take = |value: Option<Hsla>, name: &'static str, or: &dyn Fn() -> Hsla| {
            value.unwrap_or_else(|| {
                derived.push(name);
                or()
            })
        };
        let (light_page, dark_page) = (Hsla { h: 0., s: 0., l: 0.97, a: 1. }, Hsla { h: 0., s: 0., l: 0.08, a: 1. });
        let page = take(self.page, "page", &|| if appearance == Appearance::Light { light_page } else { dark_page });
        let ink = take(self.ink, "ink", &|| if appearance == Appearance::Light { dark_page } else { light_page });
        let card = take(self.card, "card", &|| mix(page, ink, 0.04));
        let card_strong = take(self.card_strong, "card_strong", &|| mix(card, ink, 0.06));
        let muted = take(self.muted, "muted", &|| mix(ink, page, 0.45));
        let divider = take(self.divider, "divider", &|| ink.opacity(0.08));
        let info = take(self.info, "info", &|| mix(ink, page, 0.3));
        let accent = take(self.accent, "accent", &|| info);
        let danger = take(self.danger, "danger", &|| mix(ink, page, 0.3));
        let success = take(self.success, "success", &|| mix(ink, page, 0.3));
        let warning = take(self.warning, "warning", &|| mix(ink, page, 0.3));
        let warning_fill = take(self.warning_fill, "warning_fill", &|| warning);
        let selection = take(self.selection, "selection", &|| ink.opacity(0.45));
        let popover = take(self.popover, "popover", &|| card);
        let shadow = take(self.shadow, "shadow", &|| gpui_kit::black().opacity(0.1));
        // beui washes a saturated green and red at 7%; muted ones need 18% for the same weight.
        let diff_added = take(self.diff_added, "diff_added", &|| success.opacity(0.18));
        let diff_removed = take(self.diff_removed, "diff_removed", &|| danger.opacity(0.18));
        // mem0's chip: the darker of page and ink at rest, the lighter when hovered.
        let (darker, lighter) = if page.l < ink.l { (page, ink) } else { (ink, page) };
        let chip_rest = take(self.chip_rest, "chip_rest", &|| darker);
        let chip_hover = take(self.chip_hover, "chip_hover", &|| lighter);
        let chip_arrow = take(self.chip_arrow, "chip_arrow", &|| lighter);
        let status = self.status.unwrap_or_else(|| {
            derived.push("status");
            [info, success, danger, warning, muted]
        });
        let theme = Theme {
            name,
            family,
            appearance,
            background: page,
            foreground: ink,
            card,
            card_strong,
            muted_foreground: muted,
            divider,
            // The primary button is the page inverted in every theme: ink fill, page text.
            primary: ink,
            primary_foreground: page,
            accent,
            info,
            danger,
            success,
            warning,
            warning_fill,
            selection,
            popover,
            shadow,
            diff_added,
            diff_removed,
            chip_rest,
            chip_hover,
            chip_arrow,
            status,
            syntax,
        };
        (theme, derived)
    }
}

#[derive(Deserialize)]
struct Ui {
    page: String,
    ink: String,
    card: String,
    card_strong: String,
    muted: String,
    divider: String,
    accent: String,
    info: String,
    danger: String,
    success: String,
    warning: String,
    #[serde(default)]
    warning_fill: Option<String>,
    chip_rest: Option<String>,
    chip_hover: Option<String>,
    chip_arrow: Option<String>,
}

#[derive(Deserialize)]
struct File {
    name: String,
    family: String,
    appearance: String,
    ui: Ui,
    status: Option<[String; 5]>,
    /// Scales each status tone's lightness: atelier Light's cream page needs its ramp a step darker.
    status_lightness: Option<f32>,
    syntax: serde_json::Value,
}

pub fn appearance(text: &str) -> Result<Appearance, String> {
    match text {
        "light" => Ok(Appearance::Light),
        "dark" => Ok(Appearance::Dark),
        other => Err(format!("appearance {other:?} is neither light nor dark")),
    }
}

/// A atelier theme file, and the tokens it left to derive.
pub fn parse(json: &str) -> Result<(Theme, Vec<&'static str>), String> {
    let file: File = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let colour = |text: &str| hex(text).ok_or_else(|| format!("{text:?} is not a colour"));
    let ui = &file.ui;
    let status = match &file.status {
        Some(list) => {
            let mut tones = [Hsla::default(); 5];
            for (tone, text) in tones.iter_mut().zip(list) {
                let c = colour(text)?;
                *tone = Hsla { l: c.l * file.status_lightness.unwrap_or(1.), ..c };
            }
            Some(tones)
        }
        None => None,
    };
    let tokens = Tokens {
        page: Some(colour(&ui.page)?),
        ink: Some(colour(&ui.ink)?),
        card: Some(colour(&ui.card)?),
        card_strong: Some(colour(&ui.card_strong)?),
        muted: Some(colour(&ui.muted)?),
        divider: Some(colour(&ui.divider)?),
        accent: Some(colour(&ui.accent)?),
        info: Some(colour(&ui.info)?),
        danger: Some(colour(&ui.danger)?),
        success: Some(colour(&ui.success)?),
        warning: Some(colour(&ui.warning)?),
        warning_fill: ui.warning_fill.as_deref().map(colour).transpose()?,
        chip_rest: ui.chip_rest.as_deref().map(colour).transpose()?,
        chip_hover: ui.chip_hover.as_deref().map(colour).transpose()?,
        chip_arrow: ui.chip_arrow.as_deref().map(colour).transpose()?,
        status,
        ..Tokens::default()
    };
    let syntax: HighlightTheme = serde_json::from_value(file.syntax).map_err(|e| format!("syntax: {e}"))?;
    let (mut theme, derived) = tokens.resolve(file.name.into(), file.family.into(), appearance(&file.appearance)?, Arc::new(syntax));
    crate::theme::raise_marks(&mut theme);
    Ok((theme, derived))
}

#[cfg(test)]
mod tests;
