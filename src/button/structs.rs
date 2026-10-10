use std::rc::Rc;

use gpui_kit::{
    AnyElement, App, ClickEvent, Corners, ElementId, FocusHandle, FontWeight, Hsla,
    InteractiveElement, IntoElement, KeyDownEvent, MouseButton, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    ClickHandler,
    icon::{Icon, IconName},
    kbd::Kbd,
    motion::{Animated, Channel, Curve, FrameClock, Spring, duration, ease},
    theme::{ActiveTheme, radius},
    tooltip::Tooltip,
    typography::FONT_FAMILY,
};
use super::types::{ButtonSize, ButtonVariant, Hold, KeyHandler, ROUND};
use super::helpers::{chip, colors, hover_target, side_pad, update_motion};

pub(super) struct Metrics {
    pub(super) height: f32,
    pub(super) pad_x: f32,
    pub(super) gap: f32,
    pub(super) text: f32,
    pub(super) icon: f32,
    /// Space around the arrow chip, which fills the rest of the height.
    pub(super) chip_inset: f32,
}

#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    pub(super) label: Option<SharedString>,
    pub(super) icon: Option<IconName>,
    trailing_icon: Option<IconName>,
    /// The key cap after the label: a command's, from the key table, or atelier's own.
    command: Option<crate::keys::Command>,
    cap: Option<SharedString>,
    /// The words' colour in place of the variant's, for a verb that carries a tone.
    pub(super) ink: Option<Hsla>,
    /// The fill in place of the variant's, for a button on a picture that is dark whatever the theme.
    fill: Option<Hsla>,
    /// The icon's colour in place of the words'.
    icon_ink: Option<Hsla>,
    content: Option<AnyElement>,
    pub(super) chip: Option<IconName>,
    pub(super) variant: ButtonVariant,
    pub(super) size: ButtonSize,
    pill: bool,
    /// Which corners round; a segment of a [`crate::button_group::ButtonGroup`] squares its inner ones.
    pub(super) corners: Corners<bool>,
    /// A stop in the Tab order that Enter and Space press.
    focusable: bool,
    /// The owner's handle, when it moves focus here itself.
    focus_with: Option<FocusHandle>,
    pub(super) tooltip: Option<SharedString>,
    /// Its menu or popover is open: the tooltip would cover it, so none shows.
    pub(super) open: bool,
    on_key: Option<KeyHandler>,
    /// A name tests find it by.
    selector: Option<&'static str>,
    pub(super) disabled: bool,
    on_click: Option<ClickHandler>,
    /// Hears the press and the release, for a button that acts while it is held.
    hold: Option<(Hold, Hold)>,
    /// Gives way when its row is too narrow, its content cut, instead of pushing the row wider.
    shrink: bool,
}

impl Button {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            icon: None,
            trailing_icon: None,
            command: None,
            cap: None,
            ink: None,
            fill: None,
            icon_ink: None,
            content: None,
            chip: None,
            variant: ButtonVariant::default(),
            size: ButtonSize::default(),
            pill: false,
            corners: ROUND,
            focusable: true,
            focus_with: None,
            tooltip: None,
            open: false,
            on_key: None,
            selector: None,
            disabled: false,
            on_click: None,
            hold: None,
            shrink: false,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// An icon before the label, or the only content of an icon button.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Wears the cap of `command`'s key in the profile in force, read from the key table, so the cap and
    /// the key that works are one. A profile with no key for it shows none.
    pub fn command(mut self, command: crate::keys::Command) -> Self {
        self.command = Some(command);
        self
    }

    /// Colours the words, keeping the fill: how a verb carries a tone (Request changes in `danger`)
    /// while no button carries a colour of its own.
    /// Fills the button with `fill` in place of the variant's colour. Pair it with [`Self::ink`].
    pub fn fill(mut self, fill: impl Into<Hsla>) -> Self {
        self.fill = Some(fill.into());
        self
    }
    pub fn ink(mut self, ink: impl Into<Hsla>) -> Self {
        self.ink = Some(ink.into());
        self
    }

    /// Colours the icon only, such as a check in `success` on the primary Approve.
    pub fn icon_ink(mut self, ink: impl Into<Hsla>) -> Self {
        self.icon_ink = Some(ink.into());
        self
    }

    /// Wears `keys` as a cap, for a key the table has no command for, such as "⌘⇧↵".
    pub fn cap(mut self, keys: impl Into<SharedString>) -> Self {
        self.cap = Some(keys.into());
        self
    }

    /// The cap it shows now, if any.
    pub fn cap_text(&self, cx: &App) -> Option<SharedString> {
        self.cap.clone().or_else(|| {
            let chord = crate::keys::chord_for(crate::keys::profile(cx), self.command?)?;
            Some(crate::keys::cap(chord))
        })
    }

    /// A small icon after the label, such as the chevron of a menu button.
    pub fn trailing_icon(mut self, icon: IconName) -> Self {
        self.trailing_icon = Some(icon);
        self
    }

    /// Replaces the icon and label with `content`, for a slot the caller animates itself.
    /// The button gives way when the row is too narrow: its content (which must cut itself, as a `truncate` text
    /// does) is clipped, and the rest of the row keeps its place.
    pub fn shrink(mut self, shrink: bool) -> Self {
        self.shrink = shrink;
        self
    }

    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }

    /// An icon in a chip on the right, which slides on hover.
    pub fn chip(mut self, icon: IconName) -> Self {
        self.chip = Some(icon);
        self
    }

    /// Rounds every corner fully instead of the 8px corner, for a round icon button.
    pub fn pill(mut self, pill: bool) -> Self {
        self.pill = pill;
        self
    }

    /// Rounds only these corners; the others stay square.
    pub fn corners(mut self, corners: Corners<bool>) -> Self {
        self.corners = corners;
        self
    }

    /// Makes it a stop in the Tab order: focused, it takes the hover look while the keyboard leads, and
    /// Enter or Space presses it.
    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    /// A focus stop on the owner's handle, so the owner can hand focus back to it.
    pub fn focus_handle(mut self, handle: FocusHandle) -> Self {
        self.focusable = true;
        self.focus_with = Some(handle);
        self
    }

    /// Words shown on hover, also while disabled, such as why it cannot be pressed.
    pub fn tooltip(mut self, words: impl Into<SharedString>) -> Self {
        self.tooltip = Some(words.into());
        self
    }

    /// Says the menu or popover this button opens is open. Its tooltip shows only while it is shut, so it
    /// never covers the menu it belongs to.
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Hears the keys pressed while it has focus, for a key of its own beyond Enter and Space.
    pub fn on_key(mut self, handler: impl Fn(&KeyDownEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_key = Some(Rc::new(handler));
        self
    }

    /// The name a test finds it by with `debug_bounds`.
    pub fn debug_name(mut self, name: &'static str) -> Self {
        self.selector = Some(name);
        self
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    /// Hears the press (`down`) and the end of it (`up`: the release, on the button or off it), for a button that acts
    /// while it is held. A click still comes after a release on the button.
    pub fn on_hold(mut self, down: impl Fn(&mut Window, &mut App) + 'static, up: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.hold = Some((Rc::new(down), Rc::new(up)));
        self
    }
}

/// Per-button motion, kept across frames by element id.
pub(super) struct ButtonMotion {
    pub(super) hovered: bool,
    /// The menu or picker it opens is open: the hover level stays.
    pub(super) held: bool,
    pressed: bool,
    /// 0 at rest, 1 hovered: drives fill and chip colors.
    pub(super) tint: Animated,
    /// 0 at rest, 1 hovered: how far the chip icons have slid.
    pub(super) slide: Animated,
    /// 1 when the ring is out, 0 when pressed in.
    pub(super) ring: Channel,
    clock: FrameClock,
    pub(super) focus: Option<FocusHandle>,
}

impl ButtonMotion {
    pub(super) fn new() -> Self {
        Self {
            hovered: false,
            held: false,
            pressed: false,
            tint: Animated::new(Spring::CHIP_TINT, 0.),
            slide: Animated::new(Spring::CHIP_SLIDE, 0.),
            ring: Channel::new(1.),
            clock: FrameClock::default(),
            focus: None,
        }
    }

    pub(super) fn retarget(&mut self, reduce: bool) {
        let hover = hover_target(self.hovered, self.held);
        self.tint.set_target(hover);
        self.slide.set_target(hover);
        let (ring, time) = if self.pressed { (0., duration::PRESS_DOWN) } else { (1., duration::PRESS_UP) };
        if self.ring.target() != ring {
            self.ring.animate(ring, Curve::Ease(time.as_secs_f32(), ease::FLUID), 0., reduce);
        }
    }

    /// Advances one frame. Returns true while the button still moves.
    fn advance(&mut self, reduce: bool) -> bool {
        let dt = self.clock.tick();
        let moving = self.tint.step(dt, reduce) | self.slide.step(dt, reduce) | self.ring.is_running();
        if !moving {
            self.clock.rest();
        }
        moving
    }
}

impl RenderOnce for Button {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let motion = window.use_keyed_state(self.id.clone(), cx, |_, _| ButtonMotion::new());
        let reduce = cx.reduce_motion();
        let disabled = self.disabled;
        let focusable = self.focusable && !disabled;
        let (tint, slide, ring, moving, focus) = motion.update(cx, |m, cx| {
            let focus = match (focusable, &self.focus_with) {
                (false, _) => None,
                (true, Some(handle)) => Some(handle.clone()),
                (true, None) => Some(m.focus.get_or_insert_with(|| cx.focus_handle()).clone()),
            };
            // A button disabled while hovered or pressed (Send after a click) must not keep that look.
            if disabled && (m.hovered || m.pressed) {
                m.hovered = false;
                m.pressed = false;
                m.retarget(reduce);
            }
            if m.held != self.open {
                m.held = self.open;
                m.retarget(reduce);
            }
            let moving = m.advance(reduce);
            (m.tint.value(), m.slide.value(), m.ring.value(), moving, focus)
        });
        // Focus from the keyboard shows as the hover look, so a Tab walk can see where it is.
        let keyed = focus.as_ref().is_some_and(|f| f.is_focused(window) && window.last_input_was_keyboard());
        let tint = if keyed { 1. } else { tint };
        if moving {
            window.request_animation_frame();
        }

        let cap = self.cap_text(cx);
        let m = self.size.metrics();
        let (fill, foreground) = colors(self.variant, cx.theme(), tint, self.chip.is_some());
        let foreground = self.ink.unwrap_or(foreground);
        let fill = self.fill.map_or(fill, |own| crate::theme::mix(own, foreground, 0.1 * tint));
        let icon_color = self.icon_ink.unwrap_or(foreground);
        let square = matches!(self.size, ButtonSize::Icon | ButtonSize::IconSm);
        let corner = if self.pill { px(m.height / 2.) } else { radius::lg() };
        let words = self.label.is_some() || self.content.is_some() || self.cap.is_some();
        let pad = side_pad(&m, words || self.icon.is_none());
        let pad_right = if self.chip.is_some() { m.chip_inset } else { pad };
        let inset = 1. - ring;
        let round = |on: bool| if on { corner - px(inset) } else { px(0.) };
        let c = self.corners;

        let theme = cx.theme().clone();
        div()
            .id(self.id.clone())
            .relative()
            .flex()
            .when(!self.shrink, |d| d.flex_none())
            .when(self.shrink, |d| d.min_w_0().overflow_hidden())
            .when(keyed, |d| d.rounded(corner).shadow(crate::focus::ring_shadow(&theme, theme.background)))
            .items_center()
            .h(px(m.height))
            .when(square, |d| d.w(px(m.height)).justify_center())
            .when(!square, |d| d.pl(px(pad)).pr(px(pad_right)).gap(px(m.gap)))
            .font_family(FONT_FAMILY)
            .font_weight(FontWeight::NORMAL)
            .text_size(px(m.text))
            .line_height(px(16.))
            .whitespace_nowrap()
            .text_color(foreground)
            .child(
                div()
                    .absolute()
                    .inset(px(inset))
                    .rounded_tl(round(c.top_left))
                    .rounded_tr(round(c.top_right))
                    .rounded_bl(round(c.bottom_left))
                    .rounded_br(round(c.bottom_right))
                    .bg(fill),
            )
            .when(keyed, |d| d.child(div().absolute().inset_0().debug_selector(|| "button-ring".into())))
            .when_some(self.content, |d, content| d.child(div().relative().when(self.shrink, |d| d.min_w_0().overflow_hidden()).child(content)))
            .when_some(self.icon, |d, icon| {
                d.child(div().relative().child(Icon::new(icon).size(px(m.icon)).color(icon_color)))
            })
            .when_some(self.label, |d, label| d.child(div().relative().child(label)))
            .when_some(cap, |d, cap| {
                let kbd = Kbd::new(cap);
                let kbd = if self.variant == ButtonVariant::Primary { kbd.on(fill, foreground) } else { kbd.ink(foreground) };
                d.child(div().relative().child(kbd))
            })
            .when_some(self.trailing_icon, |d, icon| {
                d.child(div().relative().child(Icon::new(icon).size(px(m.icon - 2.)).color(foreground)))
            })
            .when_some(self.chip, |d, icon| d.child(chip(icon, &m, tint, slide, cx.theme())))
            .when_some(focus, |d, focus| d.track_focus(&focus.tab_stop(true)))
            .when_some(self.selector, |d, name| d.debug_selector(move || name.into()))
            .when_some(self.tooltip.filter(|_| !self.open), |d, words| d.tooltip(Tooltip::text(words)))
            .when_some(self.on_key.filter(|_| !disabled), |d, f| d.on_key_down(move |event, window, cx| f(event, window, cx)))
            .when(disabled, |d| d.opacity(0.5))
            .when(!disabled, |d| {
                let (hover, press, release, release_out) = (motion.clone(), motion.clone(), motion.clone(), motion);
                let (hold_down, hold_up) = self.hold.map_or((None, None), |(down, up)| (Some(down), Some(up)));
                let hold_away = hold_up.clone();
                d.cursor_pointer()
                    .when_some(hold_down, |d, down| d.on_mouse_down(MouseButton::Left, move |_, window, cx| down(window, cx)))
                    .when_some(hold_up, |d, up| d.on_mouse_up(MouseButton::Left, move |_, window, cx| up(window, cx)))
                    .when_some(hold_away, |d, up| d.on_mouse_up_out(MouseButton::Left, move |_, window, cx| up(window, cx)))
                    .on_hover(move |on, _, cx| update_motion(&hover, cx, |m| m.hovered = *on))
                    .on_mouse_down(MouseButton::Left, move |_, _, cx| update_motion(&press, cx, |m| m.pressed = true))
                    .on_mouse_up(MouseButton::Left, move |_, _, cx| update_motion(&release, cx, |m| m.pressed = false))
                    .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
                        update_motion(&release_out, cx, |m| m.pressed = false)
                    })
                    .when_some(self.on_click, |d, handler| d.on_click(move |event, window, cx| handler(event, window, cx)))
            })
    }
}
