use std::sync::Arc;

use gpui_kit::{Hsla, SharedString, component::highlighter::HighlightTheme};
use serde::Deserialize;

use crate::theme::{Appearance, Theme, mix};

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
pub(super) struct Ui {
    pub(super) page: String,
    pub(super) ink: String,
    pub(super) card: String,
    pub(super) card_strong: String,
    pub(super) muted: String,
    pub(super) divider: String,
    pub(super) accent: String,
    pub(super) info: String,
    pub(super) danger: String,
    pub(super) success: String,
    pub(super) warning: String,
    #[serde(default)]
    pub(super) warning_fill: Option<String>,
    pub(super) chip_rest: Option<String>,
    pub(super) chip_hover: Option<String>,
    pub(super) chip_arrow: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct File {
    pub(super) name: String,
    pub(super) family: String,
    pub(super) appearance: String,
    pub(super) ui: Ui,
    pub(super) status: Option<[String; 5]>,
    /// Scales each status tone's lightness: atelier Light's cream page needs its ramp a step darker.
    pub(super) status_lightness: Option<f32>,
    pub(super) syntax: serde_json::Value,
}
