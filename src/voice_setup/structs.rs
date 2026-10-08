use std::time::Instant;

use gpui_kit::{
    App,
    ElementId,
    IntoElement,
    ParentElement,
    RenderOnce,
    SharedString,
    Styled,
    Window,
    div,
    prelude::FluentBuilder,
};

use crate::{
    cell_bar::{CellBar, pour},
    icon::{Icon, IconName},
    scale::px,
    theme::ActiveTheme,
    typography::{MONO_FONT_FAMILY, TextSize},
    voice_waves::amber_for,
};
use super::types::SetupPhase;
use super::helpers::{copy, fraction};

pub(super) struct SetupState {
    pub(super) shown: f32,
    pub(super) at: Instant,
    pub(super) born: Instant,
}

#[derive(IntoElement)]
pub struct VoiceSetup {
    pub(super) id: ElementId,
    pub(super) phase: SetupPhase,
    pub(super) total_mb: f32,
}

impl VoiceSetup {
    pub fn new(id: impl Into<ElementId>, phase: SetupPhase) -> Self {
        Self { id: id.into(), phase, total_mb: 164. }
    }

    /// The size of the download, for the words.
    pub fn total_mb(mut self, total_mb: f32) -> Self {
        self.total_mb = total_mb;
        self
    }
}

impl RenderOnce for VoiceSetup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let reduce = cx.reduce_motion();
        let target = fraction(self.phase);
        let state = window.use_keyed_state(self.id, cx, |_, _| {
            let now = Instant::now();
            SetupState { shown: 0., at: now, born: now }
        });
        let (shown, seconds) = state.update(cx, |s, _| {
            let now = Instant::now();
            let dt = now.duration_since(s.at).as_secs_f32().min(0.1);
            s.at = now;
            let want = target.unwrap_or(0.);
            s.shown = if reduce { want } else { pour(s.shown, want, dt) };
            (s.shown, now.duration_since(s.born).as_secs_f32())
        });
        // Frames only while something moves: the loading cell, or a fill still on its way to its number.
        let pouring = target.is_some_and(|t| (t - shown).abs() > 1e-3);
        if !reduce && (target.is_none() || pouring) {
            window.request_animation_frame();
        }

        let (words, size) = copy(self.phase, self.total_mb);
        let ready = matches!(self.phase, SetupPhase::Ready);
        let bar = CellBar::new(target.map(|_| shown)).color(amber_for(&theme)).seconds(seconds).moving(!reduce);

        div()
            .flex()
            .items_center()
            .gap(px(10.))
            .text_size(TextSize::Xs.font_size())
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .min_w_0()
                    .text_color(if ready { theme.foreground } else { theme.muted_foreground })
                    .when(ready, |d| d.child(Icon::new(IconName::Check).size(px(14.)).color(theme.success)))
                    .child(SharedString::from(words)),
            )
            .child(bar)
            .when_some(size, |d, size| d.child(div().flex_none().font_family(MONO_FONT_FAMILY).text_color(theme.muted_foreground).child(SharedString::from(size))))
    }
}
