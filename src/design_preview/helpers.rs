use gpui_kit::App;

use super::structs::Choice;
use crate::tabs::TabsVariant;

thread_local! {
    /// The elevation in force. A thread local, not a global: the panels that read it have no `App` to hand.
    static ELEVATION: std::cell::Cell<usize> = const { std::cell::Cell::new(2) };
    /// How strong the elevation is, 0 to 100: the tone lift and the shadow alpha scale with it.
    static STRENGTH: std::cell::Cell<usize> = const { std::cell::Cell::new(50) };
}

/// Puts the saved strength in force. `None` keeps the default, 50.
pub fn init_strength(value: Option<u8>) {
    set_strength(value.map_or(50, usize::from));
}

/// The elevation strength in force, 0 to 100.
pub fn strength() -> usize {
    STRENGTH.with(|s| s.get())
}

pub fn set_strength(value: usize) {
    STRENGTH.with(|s| s.set(value.min(100)));
}

fn strength_factor() -> f32 {
    strength() as f32 / 100.
}

/// Puts the saved elevation in force. `None` keeps the default, C.
pub fn init_elevation(design: Option<u8>) {
    set_elevation(design.map_or(2, usize::from));
}

/// The elevation of floating panels in force, 0 to 3.
pub fn elevation() -> usize {
    ELEVATION.with(|e| e.get())
}

pub fn set_elevation(design: usize) {
    ELEVATION.with(|e| e.set(clamp(design)));
}

/// A panel's fill in elevation `design`: A keeps `base`; the others are one tone step above it, lighter in dark
/// and whiter in light.
pub fn panel_fill(theme: &crate::Theme, design: usize, base: gpui_kit::Hsla) -> gpui_kit::Hsla {
    if clamp(design) == 0 {
        return base;
    }
    let white = gpui_kit::hsla(0., 0., 1., 1.);
    // At 100: half way to white in light, 6% in dark. The default, 50, is 25% and 3%.
    let amount = if theme.appearance == crate::theme::Appearance::Dark {
        0.06
    } else {
        0.5
    } * strength_factor();
    crate::theme::mix(base, white, amount)
}

/// The fill of a hovered or chosen row on a panel filled `panel`: the panel's own fill, stepped toward the ink just
/// far enough to keep the contrast the page-level pill (`card_strong` on `card`) had. So rows look the same
/// whatever the lift.
pub fn row_tone(theme: &crate::Theme, panel: gpui_kit::Hsla) -> gpui_kit::Hsla {
    let today = crate::theme::contrast(theme.card_strong, theme.card);
    let mut step = 0.02;
    loop {
        let pill = crate::theme::mix(panel, theme.foreground, step);
        if crate::theme::contrast(pill, panel) >= today || step >= 0.5 {
            return pill;
        }
        step += 0.005;
    }
}

/// A panel's 1px edge in elevation `design`: A's subtle edge, or none (the pixel stays, clear, so no size moves).
pub fn panel_edge(theme: &crate::Theme, design: usize) -> gpui_kit::Hsla {
    if clamp(design) == 0 {
        crate::theme::dropdown_edge(theme)
    } else {
        gpui_kit::transparent_black()
    }
}

/// A panel's shadows in elevation `design`. A keeps `base`; B has none; C is a tight contact shadow and a wide
/// soft one; D adds a faint inner highlight on the top edge. A panel given no shadow stays without.
pub fn panel_shadows(
    theme: &crate::Theme,
    design: usize,
    base: Vec<gpui_kit::BoxShadow>,
) -> Vec<gpui_kit::BoxShadow> {
    use crate::scale::px;
    use gpui_kit::{BoxShadow, hsla, point};
    let design = clamp(design);
    if design == 0 || base.is_empty() {
        return base;
    }
    if design == 1 {
        return Vec::new();
    }
    let dark = theme.appearance == crate::theme::Appearance::Dark;
    let scale = strength_factor();
    let layer = |y: f32, blur: f32, alpha: f32| BoxShadow {
        color: hsla(0., 0., 0., alpha * scale),
        offset: point(px(0.), px(y)),
        blur_radius: px(blur),
        spread_radius: px(0.),
        inset: false,
    };
    let mut shadows = vec![
        layer(1., 2., if dark { 0.40 } else { 0.12 }),
        layer(8., 24., if dark { 0.35 } else { 0.10 }),
    ];
    if design == 3 {
        shadows.push(BoxShadow {
            color: theme.foreground.opacity(0.06 * scale),
            offset: point(px(0.), px(1.)),
            blur_radius: px(0.),
            spread_radius: px(0.),
            inset: true,
        });
    }
    shadows
}

/// Puts the saved choice in force. `None` keeps the default: A for the tabs.
pub fn init(tabs: Option<u8>, cx: &mut App) {
    cx.set_global(Choice {
        tabs: clamp(tabs.map(usize::from).unwrap_or(0)),
    });
}

fn clamp(n: usize) -> usize {
    n.min(3)
}

pub(super) fn now(cx: &App) -> Choice {
    cx.try_global::<Choice>()
        .copied()
        .unwrap_or(Choice { tabs: 0 })
}

/// The design of the editor tabs in force, 0 to 3.
pub fn tabs(cx: &App) -> usize {
    now(cx).tabs
}

pub fn set_tabs(design: usize, cx: &mut App) {
    cx.set_global(Choice {
        tabs: clamp(design),
    });
}

/// The tabs variant of editor design `design`.
pub fn tab_variant(design: usize) -> TabsVariant {
    [
        TabsVariant::Chip,
        TabsVariant::ChipLine,
        TabsVariant::Dot,
        TabsVariant::Tick,
    ][clamp(design)]
}
