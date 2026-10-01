//! TextInput: beui.dev's Input (`components/motion/input.tsx`), in atelier's look. A field 28px tall with an
//! optional label over it, an icon at either end, and a line for an error under it.
//!
//! - Field: the app's own field from before this part: `rounded LG` (8), the `card_strong` fill with no border, the
//!   text at 14px, 10px in from the edge (30px with an icon), and a small muted label over it. The web's pill is
//!   44px with a border; that is its look, not ours.
//! - Focus: the quiet ring (atelier's ring: 2px, 3:1 with the surface).
//! - Error: a 2px ring in the danger tone, the field shakes once
//!   (`x: 0, -6, 6, -4, 4, -2, 0` over 450ms), and the message comes up under it over 200ms. With
//!   [`TextInput::reserve_error_line`] the line keeps its 16px, so nothing below it moves.
//! - Success: a check at the right end. Under Reduce Motion nothing shakes or slides.
//!
//! What gpui cannot draw is left out: the blur on the message as it comes and goes, the message's exit
//! (it goes at once), and the check's draw-on (it fades in).
use gpui_kit::{
    AnyElement, App, ElementId, Entity, Focusable, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    component::input::{Input, InputState},
    div, prelude::FluentBuilder, 
};
use crate::scale::px;

use crate::{
    focus::ring_shadow,
    icon::{Icon, IconName},
    motion::{Channel, Curve, FrameClock, keyframes},
    theme::{ActiveTheme, Theme, mix},
    typography::{FONT_FAMILY, TextSize},
};

/// The field's height: atelier's control size.
pub const HEIGHT: f32 = 28.;
/// The field's corner: atelier's field radius.
pub const CORNER: f32 = 8.;
/// The gap between the label, the field and the message.
pub const GAP: f32 = 4.;
/// The room the message line keeps when reserved (`min-h-4`).
pub const MESSAGE_LINE: f32 = 16.;
/// The text's left edge with no icon and with one.
pub const TEXT_INSET: f32 = 10.;
pub const TEXT_INSET_ICON: f32 = 30.;
/// How long the shake takes, and where it goes (`x` in px, at even steps).
pub const SHAKE_SECONDS: f32 = 0.45;
pub const SHAKE: [f32; 7] = [0., -6., 6., -4., 4., -2., 0.];
/// How long the message takes to come up.
const MESSAGE_SECONDS: f32 = 0.2;
/// The linear curve, for progress measured in seconds.
const LINEAR: [f32; 4] = [0., 0., 1., 1.];
/// Motion's ease-in-out, between the shake's keyframes.
const KEYFRAME_EASE: [f32; 4] = [0.42, 0., 0.58, 1.];

/// The field's sideways offset `t` seconds into a shake.
pub fn shake_offset(t: f32) -> f32 {
    let times: Vec<f32> = (0..SHAKE.len()).map(|i| i as f32 / (SHAKE.len() - 1) as f32).collect();
    keyframes(&SHAKE, &times, SHAKE_SECONDS, KEYFRAME_EASE, t)
}

/// The idle border on `surface` (`border-border`).
pub fn edge(theme: &Theme, surface: Hsla) -> Hsla {
    mix(surface, theme.foreground, 0.12)
}

/// The field's fill on `surface`: the theme's `card_strong` step, so the field parts from its surface with no line.
pub fn fill(theme: &Theme, _surface: Hsla) -> Hsla {
    theme.card_strong
}

struct Motion {
    had_error: bool,
    shake: Channel,
    message: Channel,
    clock: FrameClock,
}

#[derive(IntoElement)]
pub struct TextInput {
    id: ElementId,
    state: Entity<InputState>,
    label: Option<SharedString>,
    error: Option<SharedString>,
    invalid: bool,
    reserve: bool,
    success: bool,
    left: Option<IconName>,
    right: Option<AnyElement>,
    disabled: bool,
    surface: Option<Hsla>,
    selector: Option<&'static str>,
}

impl TextInput {
    pub fn new(id: impl Into<ElementId>, state: &Entity<InputState>) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            label: None,
            error: None,
            invalid: false,
            reserve: false,
            success: false,
            left: None,
            right: None,
            disabled: false,
            surface: None,
            selector: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Shows the field as wrong and says why under it.
    pub fn error(mut self, message: impl Into<SharedString>) -> Self {
        self.error = Some(message.into());
        self.invalid = true;
        self
    }

    /// Shows the field as wrong with no message.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Keeps the message line's room whether or not there is a message.
    pub fn reserve_error_line(mut self, reserve: bool) -> Self {
        self.reserve = reserve;
        self
    }

    pub fn success(mut self, success: bool) -> Self {
        self.success = success;
        self
    }

    pub fn left_icon(mut self, icon: IconName) -> Self {
        self.left = Some(icon);
        self
    }

    /// Something at the right end, such as a button.
    pub fn right(mut self, right: impl IntoElement) -> Self {
        self.right = Some(right.into_any_element());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// The colour the field sits on, which its border and ring are measured against. The card by default.
    pub fn surface(mut self, surface: Hsla) -> Self {
        self.surface = Some(surface);
        self
    }

    /// The name a test finds the field's box by with `debug_bounds`.
    pub fn debug_name(mut self, name: &'static str) -> Self {
        self.selector = Some(name);
        self
    }
}

impl RenderOnce for TextInput {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let reduce = cx.reduce_motion();
        let surface = self.surface.unwrap_or(theme.card);
        let focused = self.state.read(cx).focus_handle(cx).contains_focused(window, cx);
        let motion = window.use_keyed_state(self.id.clone(), cx, |_, _| Motion {
            had_error: false,
            shake: Channel::new(1.),
            message: Channel::new(1.),
            clock: FrameClock::default(),
        });
        let (offset, rise, moving) = motion.update(cx, |m, _| {
            if self.invalid && !m.had_error {
                m.shake = Channel::new(0.);
                m.shake.animate(1., Curve::Ease(SHAKE_SECONDS, LINEAR), 0., reduce);
                m.message = Channel::new(0.);
                m.message.animate(1., Curve::Ease(MESSAGE_SECONDS, LINEAR), 0., reduce);
            }
            m.had_error = self.invalid;
            m.clock.tick();
            let moving = m.shake.is_running() || m.message.is_running();
            if !moving {
                m.clock.rest();
            }
            (shake_offset(m.shake.value() * SHAKE_SECONDS), m.message.value(), moving)
        });
        if moving {
            window.request_animation_frame();
        }

        let ring = if self.invalid {
            Some(vec![gpui_kit::BoxShadow {
                color: theme.danger,
                offset: gpui_kit::point(px(0.), px(0.)),
                blur_radius: px(0.),
                spread_radius: px(crate::focus::RING_WIDTH),
                inset: false,
            }])
        } else if focused {
            Some(ring_shadow(&theme, surface))
        } else {
            None
        };
        let pad_left = if self.left.is_some() { TEXT_INSET_ICON } else { TEXT_INSET };
        let pad_right = if self.success || self.right.is_some() { TEXT_INSET_ICON } else { TEXT_INSET };
        let field = div()
            .relative()
            .h(px(HEIGHT))
            .left(px(if reduce { 0. } else { offset }))
            .overflow_hidden()
            .rounded(px(CORNER))
            // gpui paints a shadow under the whole box, not round it: the fill keeps the ring out of the field.
            .bg(fill(&theme, surface))
            .when(self.disabled, |d| d.opacity(0.6))
            .when_some(ring, |d, ring| d.shadow(ring))
            .when_some(self.selector, |d, name| d.debug_selector(move || name.into()))
            .children(self.left.map(|icon| {
                div()
                    .absolute()
                    .left(px(8.))
                    .top_0()
                    .h_full()
                    .flex()
                    .items_center()
                    .child(Icon::new(icon).size(px(16.)).color(theme.muted_foreground))
            }))
            .child(
                div()
                    .size_full()
                    .flex()
                    .items_center()
                    .pl(px(pad_left))
                    .pr(px(pad_right))
                    .text_color(theme.foreground)
                    .child(Input::new(&self.state).appearance(false).px(px(0.)).text_size(TextSize::Sm.font_size()).disabled(self.disabled)),
            )
            .child(if self.success {
                div()
                    .absolute()
                    .right(px(10.))
                    .top_0()
                    .h_full()
                    .flex()
                    .items_center()
                    .child(Icon::new(IconName::Check).size(px(16.)).color(theme.success))
                    .into_any_element()
            } else if let Some(right) = self.right {
                div().absolute().right_0().top_0().h_full().flex().items_center().text_color(theme.muted_foreground).child(right).into_any_element()
            } else {
                div().into_any_element()
            });

        let message = self.error.map(|words| {
            div()
                .px(px(4.))
                .mt(px((1. - rise) * -4.))
                .opacity(rise)
                .text_size(TextSize::Xs.font_size())
                .line_height(px(16.))
                .text_color(theme.danger)
                .child(words)
        });
        div()
            .flex()
            .flex_col()
            .gap(px(GAP))
            .font_family(FONT_FAMILY)
            .children(self.label.map(|label| {
                div()
                    .px(px(4.))
                    .text_size(TextSize::Xs.font_size())
                    .line_height(px(16.))
                    .text_color(theme.muted_foreground)
                    .child(label)
            }))
            .child(field)
            .children(if self.reserve { Some(div().min_h(px(MESSAGE_LINE)).children(message)) } else { message })
    }
}

#[cfg(test)]
mod tests;
