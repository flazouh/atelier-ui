use std::collections::BTreeMap;

use gpui_kit::{Hsla, Rgba};

use crate::theme::contrast;
use super::structs::Swatch;
use super::types::{COUNT, MAX_LABEL};

/// The twelve colours, in acepe's order. They are data (`assets/project_palette.json`), not colours a component names.
pub fn palette() -> &'static [Swatch] {
    static PALETTE: std::sync::OnceLock<Vec<Swatch>> = std::sync::OnceLock::new();
    PALETTE.get_or_init(|| {
        #[derive(serde::Deserialize)]
        struct Row {
            name: String,
            hex: String,
        }
        let rows: Vec<Row> = serde_json::from_str(include_str!("../../assets/project_palette.json")).expect("the palette is JSON");
        rows.into_iter().map(|r| Swatch { name: r.name, rgb: u32::from_str_radix(&r.hex, 16).expect("a hex colour") }).collect()
    })
}

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
