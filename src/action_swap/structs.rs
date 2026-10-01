use std::rc::Rc;

use gpui_kit::{
    App, ClickEvent, ElementId, FocusHandle, FontWeight, InteractiveElement, IntoElement,
    MouseButton, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled,
    Window, div, prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    focus::ring_shadow,
    kbd::Kbd,
    keys::{self, Command},
    motion::{Channel, Curve, FrameClock, duration, ease},
    roll::{Kind, Roll},
    theme::{ActiveTheme},
    theme::radius,
    typography::FONT_FAMILY,
};
use super::types::{Click, PRESS_INSET, SwapSize, SwapVariant};
use super::helpers::colors;

pub(super) struct Motion {
    pub(super) hover: Channel,
    pub(super) press: Channel,
    pub(super) hovered: bool,
    pub(super) pressed: bool,
    pub(super) clock: FrameClock,
    pub(super) focus: Option<FocusHandle>,
}

#[derive(IntoElement)]
pub struct ActionSwapButton {
    pub(super) id: ElementId,
    pub(super) label: SharedString,
    pub(super) variant: SwapVariant,
    pub(super) size: SwapSize,
    pub(super) command: Option<Command>,
    pub(super) cap: Option<SharedString>,
    pub(super) disabled: bool,
    pub(super) on_click: Option<Click>,
    pub(super) selector: Option<&'static str>,
}

impl ActionSwapButton {
    /// `label` is the words now; when it differs from last frame's, the old words roll out and these roll in.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self { id: id.into(), label: label.into(), variant: SwapVariant::default(), size: SwapSize::default(), command: None, cap: None, disabled: false, on_click: None, selector: None }
    }

    pub fn variant(mut self, variant: SwapVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: SwapSize) -> Self {
        self.size = size;
        self
    }

    /// Wears the cap of `command`'s key in the profile in force.
    pub fn command(mut self, command: Command) -> Self {
        self.command = Some(command);
        self
    }

    pub fn cap(mut self, keys: impl Into<SharedString>) -> Self {
        self.cap = Some(keys.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }

    pub fn debug_name(mut self, name: &'static str) -> Self {
        self.selector = Some(name);
        self
    }
}

impl RenderOnce for ActionSwapButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let reduce = cx.reduce_motion();
        let motion = window.use_keyed_state(self.id.clone(), cx, |_, _| Motion {
            hover: Channel::new(0.),
            press: Channel::new(0.),
            hovered: false,
            pressed: false,
            clock: FrameClock::default(),
            focus: None,
        });
        let disabled = self.disabled;
        let (hover, press, moving, focus) = motion.update(cx, |m, cx| {
            let focus = m.focus.get_or_insert_with(|| cx.focus_handle()).clone();
            let go = |channel: &mut Channel, to: f32, time: std::time::Duration| {
                if channel.target() != to {
                    channel.animate(to, Curve::Ease(time.as_secs_f32(), ease::FLUID), 0., reduce);
                }
            };
            go(&mut m.hover, f32::from(u8::from(m.hovered && !disabled)), duration::REVEAL);
            go(&mut m.press, f32::from(u8::from(m.pressed && !disabled)), duration::PRESS_DOWN);
            m.clock.tick();
            let moving = m.hover.is_running() | m.press.is_running();
            if !moving {
                m.clock.rest();
            }
            (m.hover.value(), m.press.value(), moving, focus)
        });
        if moving {
            window.request_animation_frame();
        }
        let keyed = focus.is_focused(window) && window.last_input_was_keyboard() && !disabled;
        let (height, pad_x, gap, text, line) = self.size.metrics();
        let (fill, ink) = colors(self.variant, &theme, hover);
        let cap = self.cap.clone().or_else(|| self.command.and_then(|c| keys::chord_for(keys::profile(cx), c)).map(keys::cap));
        let words = Roll::new((self.id.clone(), "words"), self.label.clone(), Kind::Swap, px(line), move |label: &SharedString| {
            div().h(px(line)).line_height(px(line)).whitespace_nowrap().child(label.clone()).into_any_element()
        });
        let inset = PRESS_INSET * press;
        let pill = div().absolute().inset_0().rounded(radius::lg()).bg(fill);
        let (over, out, down, up) = (motion.clone(), motion.clone(), motion.clone(), motion);
        let _ = out;
        div()
            .id(self.id.clone())
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .gap(px(gap))
            .h(px(height))
            .px(px(pad_x))
            .rounded(radius::lg())
            .font_family(FONT_FAMILY)
            .font_weight(FontWeight::MEDIUM)
            .text_size(px(text))
            .line_height(px(line))
            .text_color(ink)
            .when(keyed, |d| d.shadow(ring_shadow(&theme, theme.background)))
            .when(disabled, |d| d.opacity(0.5))
            .when(!disabled, |d| d.cursor_pointer().track_focus(&focus.tab_stop(true)))
            .when_some(self.selector, |d, name| d.debug_selector(move || name.into()))
            .child(pill.top(px(inset * height)).bottom(px(inset * height)).left(gpui_kit::relative(inset)).right(gpui_kit::relative(inset)))
            .child(div().relative().child(words))
            .children(cap.map(|cap| {
                let kbd = Kbd::new(cap);
                div().relative().child(if self.variant == SwapVariant::Primary { kbd.on(fill, ink) } else { kbd.ink(ink) })
            }))
            .when(!disabled, |d| {
                d.on_hover(move |on, _, cx| {
                    over.update(cx, |m, _| m.hovered = *on);
                })
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    down.update(cx, |m, _| m.pressed = true);
                })
                .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                    up.update(cx, |m, _| m.pressed = false);
                })
                .when_some(self.on_click.clone(), |d, click| {
                    let key = click.clone();
                    d.on_click(move |event, window, cx| click(event, window, cx)).on_key_down(move |event, window, cx| {
                        if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                            cx.stop_propagation();
                            key(&ClickEvent::default(), window, cx);
                        }
                    })
                })
            })
    }
}
