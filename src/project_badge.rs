//! A project's badge in the sidebar: its first letter (two when names collide) on a colour from a fixed palette, or
//! the project's own icon in its place. The palette is acepe's twelve (`project-color-options.ts`). A project has a
//! colour of its own by where it lives, so it keeps it from run to run; one the reader picked beats that.
use std::{collections::BTreeMap, path::PathBuf};
use gpui_kit::{
    Hsla, IntoElement, ObjectFit, ParentElement, RenderOnce, SharedString, Styled, StyledImage, Window, App, div,
    prelude::FluentBuilder, Rgba,
};
use crate::scale::px;
use crate::theme::contrast;

/// How many colours a project badge can take.
pub const COUNT: usize = 12;
/// One colour of the palette: its name and its red, green and blue bytes.
pub struct Swatch {
    pub name: String,
    pub rgb: u32,
}
/// The twelve colours, in acepe's order. They are data (`assets/project_palette.json`), not colours a component names.
pub fn palette() -> &'static [Swatch] {
    static PALETTE: std::sync::OnceLock<Vec<Swatch>> = std::sync::OnceLock::new();
    PALETTE.get_or_init(|| {
        #[derive(serde::Deserialize)]
        struct Row {
            name: String,
            hex: String,
        }
        let rows: Vec<Row> = serde_json::from_str(include_str!("../assets/project_palette.json")).expect("the palette is JSON");
        rows.into_iter().map(|r| Swatch { name: r.name, rgb: u32::from_str_radix(&r.hex, 16).expect("a hex colour") }).collect()
    })
}
/// The longest label: the badge is a small square next to the name, so a third letter would crowd it.
const MAX_LABEL: usize = 2;
/// The badge's side.
pub const SIZE: f32 = 16.;

/// The colour a place gets when no one picked one: FNV-1a of the place, modulo the palette. Stable across runs.
pub fn fallback_color(place: &str) -> usize {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in place.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    (hash % COUNT as u64) as usize
}
/// The colour index in force: the one picked, when it is in the palette, else the place's own.
pub fn color_of(picked: Option<usize>, place: &str) -> usize {
    picked.filter(|i| *i < COUNT).unwrap_or_else(|| fallback_color(place))
}
/// The fill of colour `index`.
pub fn fill(index: usize) -> Hsla {
    let rgb = palette()[index.min(COUNT - 1)].rgb;
    Rgba { r: ((rgb >> 16) & 0xFF) as f32 / 255., g: ((rgb >> 8) & 0xFF) as f32 / 255., b: (rgb & 0xFF) as f32 / 255., a: 1. }.into()
}
/// Dark or white ink, whichever reads better on `fill`.
pub fn ink_on(fill: Hsla) -> Hsla {
    let (dark, light) = (gpui_kit::hsla(0., 0., 0.08, 1.), gpui_kit::hsla(0., 0., 1., 1.));
    if contrast(dark, fill) >= contrast(light, fill) { dark } else { light }
}
/// A label for each project, unique across the set where two letters can make it so. Starts from the first letter and
/// grows one at a time while another project shares the prefix, to two at most. `projects` is `(key, name)`.
pub fn labels(projects: &[(&str, &str)]) -> BTreeMap<String, String> {
    let upper: Vec<(&str, &str, String)> = projects.iter().map(|(key, name)| (*key, *name, name.to_uppercase())).collect();
    let mut out = BTreeMap::new();
    for (key, name, normal) in &upper {
        let total = normal.chars().count();
        let mut length = 1;
        while length < total && length < MAX_LABEL {
            let prefix: String = normal.chars().take(length).collect();
            if !upper.iter().any(|(other, _, n)| other != key && n.starts_with(&prefix)) {
                break;
            }
            length += 1;
        }
        let mut chars = name.chars();
        let label: String = match chars.next() {
            Some(first) => first.to_uppercase().chain(chars.take(length - 1)).collect(),
            None => String::new(),
        };
        out.insert((*key).to_string(), label);
    }
    out
}

/// The badge: the project's icon when it has one, else its label on its colour.
#[derive(IntoElement)]
pub struct ProjectBadge {
    label: SharedString,
    color: usize,
    icon: Option<PathBuf>,
}
impl ProjectBadge {
    pub fn new(label: impl Into<SharedString>, color: usize) -> Self {
        Self { label: label.into(), color, icon: None }
    }
    /// An image file that stands in for the letter.
    pub fn icon(mut self, icon: Option<PathBuf>) -> Self {
        self.icon = icon;
        self
    }
}
impl RenderOnce for ProjectBadge {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let fill = fill(self.color);
        let two = self.label.chars().count() > 1;
        div()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .size(px(SIZE))
            .rounded(px(4.))
            .overflow_hidden()
            .when_some(self.icon.clone(), |d, icon| d.child(gpui_kit::img(icon).size(px(SIZE)).object_fit(ObjectFit::Contain)))
            .when(self.icon.is_none(), |d| {
                d.bg(fill)
                    .text_color(ink_on(fill))
                    .text_size(px(if two { 8. } else { 10. }))
                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                    .line_height(px(SIZE))
                    .child(self.label.clone())
            })
    }
}
#[cfg(test)]
mod tests;
