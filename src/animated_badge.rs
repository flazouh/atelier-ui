//! AnimatedBadge: beui.dev's Animated Badge (`components/motion/animated-badge.tsx`). A pill with a status
//! icon and words. When the status or the words change, each rolls up out and the new one rolls up in
//! ([`crate::roll`]).
//!
//! - Sizes, in atelier's look (the old `Badge`: a pill with no border): Small is 20px tall with 8px across, 4px
//!   between, 11px words and a 12px icon; Medium is 24px, 10px, 6px, 12px words and a 14px icon.
//! - Statuses: Neutral (a ring), Info, Success (a check), Warning, Danger (a cross) and Loading (a
//!   turning ring). A tone is its colour for the words and the icon and a 14% wash for the fill, as the old
//!   `Badge` had it. Neutral is the muted words on the `card_strong` step.
//! - Loading pulses: a wash of the words' colour swells between 8% and 16% over 1.6s. With Reduce Motion it
//!   holds still and the ring does not turn.
//!
//! What gpui cannot draw is left out: the pulse's growth (it grows only in opacity), and the badge's spring
//! when its width changes (the width changes at once). Info and Loading use atelier's `info` tone, not the
//! primary, so they read on every primary the reader can choose.
use gpui_kit::{App, ElementId, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div, };
use crate::scale::px;

use crate::{
    icon::{Icon, IconName},
    motion::{keyframes, now_millis},
    roll::{Kind, Roll},
    theme::{ActiveTheme, Theme},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeStatus {
    #[default]
    Neutral,
    Info,
    Success,
    Warning,
    Danger,
    Loading,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeSize {
    Small,
    #[default]
    Medium,
}

impl BadgeSize {
    /// Height, padding across, gap, words and icon.
    pub fn metrics(self) -> (f32, f32, f32, f32, f32) {
        match self {
            BadgeSize::Small => (20., 8., 4., 11., 12.),
            BadgeSize::Medium => (24., 10., 6., 12., 14.),
        }
    }
}

/// How long the loading pulse takes, and how strong the wash is at its ends and its middle.
pub const PULSE_MILLIS: u128 = 1600;
pub const PULSE_WASH: (f32, f32) = (0.08, 0.16);

/// The wash's strength `millis` into the pulse.
pub fn pulse_at(millis: u128) -> f32 {
    let t = (millis % PULSE_MILLIS) as f32 / PULSE_MILLIS as f32;
    keyframes(&[PULSE_WASH.0, PULSE_WASH.1, PULSE_WASH.0], &[0., 0.5, 1.], 1., [0.42, 0., 0.58, 1.], t)
}

/// The words' colour and the fill of a status: the old `Badge`'s.
pub fn colors(status: BadgeStatus, theme: &Theme) -> (Hsla, Hsla) {
    let tone = |c: Hsla| (c, c.opacity(0.14));
    match status {
        BadgeStatus::Neutral => (theme.muted_foreground, theme.card_strong),
        BadgeStatus::Info | BadgeStatus::Loading => tone(theme.info),
        BadgeStatus::Success => tone(theme.success),
        BadgeStatus::Warning => tone(theme.warning),
        BadgeStatus::Danger => tone(theme.danger),
    }
}

fn icon_of(status: BadgeStatus) -> IconName {
    match status {
        BadgeStatus::Neutral => IconName::Circle,
        BadgeStatus::Info => IconName::Info,
        BadgeStatus::Success => IconName::Check,
        BadgeStatus::Warning => IconName::Warning,
        BadgeStatus::Danger => IconName::Close,
        BadgeStatus::Loading => IconName::Progress,
    }
}

#[derive(IntoElement)]
pub struct AnimatedBadge {
    id: ElementId,
    status: BadgeStatus,
    size: BadgeSize,
    label: Option<SharedString>,
    show_icon: bool,
    selector: Option<&'static str>,
}

impl AnimatedBadge {
    pub fn new(id: impl Into<ElementId>, status: BadgeStatus) -> Self {
        Self { id: id.into(), status, size: BadgeSize::default(), label: None, show_icon: true, selector: None }
    }

    pub fn size(mut self, size: BadgeSize) -> Self {
        self.size = size;
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn show_icon(mut self, show: bool) -> Self {
        self.show_icon = show;
        self
    }

    pub fn debug_name(mut self, name: &'static str) -> Self {
        self.selector = Some(name);
        self
    }
}

impl RenderOnce for AnimatedBadge {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let reduce = cx.reduce_motion();
        let (height, pad, gap, text, icon) = self.size.metrics();
        let (ink, fill) = colors(self.status, &theme);
        let loading = self.status == BadgeStatus::Loading;
        let millis = now_millis();
        if loading && !reduce {
            window.request_animation_frame();
        }
        let wash = (loading && !reduce).then(|| div().absolute().inset_0().rounded_full().bg(ink.opacity(pulse_at(millis))));
        let turn = (loading && !reduce).then(|| (millis % 1000) as f32 / 1000.);
        let mark = self.show_icon.then(|| {
            Roll::new((self.id.clone(), "icon"), self.status, Kind::Icon, px(icon), move |status: &BadgeStatus| {
                let mut icon = Icon::new(icon_of(*status)).size(px(icon)).color(ink);
                if let (BadgeStatus::Loading, Some(t)) = (status, turn) {
                    icon = icon.turn(t);
                }
                icon.into_any_element()
            })
        });
        let words = self.label.map(|label| {
            Roll::new((self.id.clone(), "words"), label, Kind::Words, px(16.), move |label: &SharedString| {
                div().h(px(16.)).line_height(px(16.)).whitespace_nowrap().child(label.clone()).into_any_element()
            })
        });
        div()
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(gap))
            .h(px(height))
            .px(px(pad))
            .overflow_hidden()
            .rounded_full()
            .bg(fill)
            .text_color(ink)
            .text_size(px(text))
            .font_weight(FontWeight::MEDIUM)
            .whitespace_nowrap()
            .children(wash)
            .children(mark)
            .children(words)
    }
}

#[cfg(test)]
mod tests;
