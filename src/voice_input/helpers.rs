use gpui_kit::{
    AnyElement, App, Div, Hsla, IntoElement, ParentElement, SharedString, Styled, Window, div,
    prelude::FluentBuilder,
};

use super::structs::Mic;
use super::types::{BUTTON, RING_REACH, RING_SECONDS, VoiceMode};
use crate::{
    button::{Button, ButtonSize, ButtonVariant},
    icon::{Icon, IconName},
    scale::px,
    theme::Theme,
    typography::TextSize,
    voice_waves::{VoiceWaves, amber_for, on_amber},
};

/// `m:ss` for a recording that has run `seconds`.
pub fn clock(seconds: u64) -> String {
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

/// How far into a breath of the ring it is, 0 to 1, `seconds` after listening began.
pub fn ring_phase(seconds: f32) -> f32 {
    (seconds / RING_SECONDS).fract()
}

/// The ring's opacity and its reach in pixels at `phase`, scaled by how far the swap has gone (`swap`, 0 to 1): it starts
/// strong at the disc and thins out as it grows, and it does not show before the stop square does.
pub fn ring_at(phase: f32, swap: f32) -> (f32, f32) {
    let eased = 1. - (1. - phase).powi(2);
    (0.5 * (1. - phase) * swap.clamp(0., 1.), RING_REACH * eased)
}

pub(crate) fn key(mode: VoiceMode) -> &'static str {
    match mode {
        VoiceMode::Idle => "idle",
        VoiceMode::Setup => "setup",
        VoiceMode::Listening => "listening",
        VoiceMode::Failed => "failed",
    }
}

/// The words for a press that ended without any: a warning mark and the reason, in the warning color.
pub(crate) fn failed_row(message: SharedString, theme: &Theme) -> AnyElement {
    div()
        .w_full()
        .flex()
        .items_center()
        .gap(px(6.))
        .text_size(TextSize::Xs.font_size())
        .text_color(theme.warning)
        .child(
            div().flex_none().child(
                Icon::new(IconName::Warning)
                    .size(px(14.))
                    .color(theme.warning),
            ),
        )
        .child(
            div()
                .min_w_0()
                .overflow_hidden()
                .text_ellipsis()
                .whitespace_nowrap()
                .child(message),
        )
        .into_any_element()
}

/// The bars and the time beside them: what the bar says while it listens.
pub(crate) fn listening_row(level: f32, seconds: f32, muted: Hsla) -> AnyElement {
    div()
        .w_full()
        .flex()
        .items_center()
        .gap(px(12.))
        .child(
            div().flex_1().min_w_0().child(
                VoiceWaves::new("voice-input-waves")
                    .level(level)
                    .height(px(26.))
                    .bars(44),
            ),
        )
        .child(
            div()
                .flex_none()
                .w(px(34.))
                .text_size(TextSize::Xs.font_size())
                .text_color(muted)
                .child(SharedString::from(clock(seconds as u64))),
        )
        .into_any_element()
}

pub(crate) fn mic_slot(
    mic: Mic,
    on_click: impl Fn(&gpui_kit::ClickEvent, &mut Window, &mut App) + 'static,
) -> Div {
    let Mic {
        id,
        mode,
        swap,
        seconds,
        blocked,
        theme,
        reduce,
    } = mic;
    let (tone, foreground) = (amber_for(&theme), theme.foreground);
    // The microphone and the stop square share one slot, blended by `swap` so neither ever pops.
    let ink = on_amber(&theme);
    let glyphs = div()
        .relative()
        .size(px(16.))
        .child(
            div()
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .justify_center()
                .top(px(-3. * swap))
                .opacity(1. - swap)
                .child(Icon::new(IconName::Mic).size(px(16.)).color(foreground)),
        )
        .child(
            div()
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .justify_center()
                .top(px(3. * (1. - swap)))
                .opacity(swap)
                .child(Icon::new(IconName::Stop).size(px(24.)).color(ink)),
        );
    let (ring_opacity, ring_reach) = if reduce {
        (0., 0.)
    } else {
        ring_at(ring_phase(seconds), swap)
    };
    let button = Button::new(id)
        .content(glyphs)
        .pill(true)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Icon)
        .disabled(blocked || mode == VoiceMode::Setup)
        .tooltip(if mode == VoiceMode::Listening {
            "Stop"
        } else {
            "Dictate"
        })
        .on_click(on_click);
    div()
        .relative()
        .flex_none()
        .size(px(BUTTON))
        // The disc fills with amber as the square comes in.
        .child(
            div()
                .absolute()
                .inset_0()
                .rounded_full()
                .bg(tone.opacity(swap)),
        )
        .when(ring_opacity > 0.01, |d| {
            d.child(
                div()
                    .absolute()
                    .top(px(-ring_reach))
                    .left(px(-ring_reach))
                    .size(px(BUTTON + 2. * ring_reach))
                    .rounded_full()
                    .border_1()
                    .border_color(tone.opacity(ring_opacity)),
            )
        })
        .child(button)
}
