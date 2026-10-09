use std::sync::Arc;

use gpui_kit::{Hsla, Rgba, component::highlighter::HighlightTheme};

use crate::theme::{Appearance, Theme};
use super::structs::{File, Tokens};

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

/// The three shades of a hue from its base: the base, a darker one and a lighter one.
pub fn shades(base: Hsla) -> [Hsla; 3] {
    [base, Hsla { l: base.l * 0.82, ..base }, Hsla { l: base.l + (1. - base.l) * 0.35, ..base }]
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
    let chart = match &file.chart {
        Some(hues) => {
            let mut palette = [[Hsla::default(); 3]; 4];
            for (row, texts) in palette.iter_mut().zip(hues) {
                for (shade, text) in row.iter_mut().zip(texts) {
                    *shade = colour(text)?;
                }
            }
            Some(palette)
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
        chart,
        stop: file.stop.as_deref().map(colour).transpose()?,
        ..Tokens::default()
    };
    let syntax: HighlightTheme = serde_json::from_value(file.syntax).map_err(|e| format!("syntax: {e}"))?;
    let (mut theme, derived) = tokens.resolve(file.name.into(), file.family.into(), appearance(&file.appearance)?, Arc::new(syntax));
    crate::theme::raise_marks(&mut theme);
    Ok((theme, derived))
}

/// GitHub's (Primer's) open, draft, done and closed colours, so a pull request reads as it does there.
pub(crate) fn github_pull(appearance: Appearance) -> [Hsla; 4] {
    let hex = match appearance {
        Appearance::Light => [0x1A7F37, 0x59636E, 0x8250DF, 0xCF222E],
        Appearance::Dark => [0x3FB950, 0x9198A1, 0xA371F7, 0xF85149],
    };
    hex.map(|c| gpui_kit::rgb(c).into())
}
