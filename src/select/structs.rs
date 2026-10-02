use std::{rc::Rc, time::Instant};

use gpui_kit::{
    App,
    Bounds,
    ElementId,
    FocusHandle,
    InteractiveElement,
    IntoElement,
    ParentElement,
    Pixels,
    RenderOnce,
    SharedString,
    StatefulInteractiveElement,
    Styled,
    Window,
    base::{Select as BaseSelect, actions::{SelectDown, SelectFirst, SelectLast, SelectUp}},
    div,
    prelude::FluentBuilder,
    transparent_black,
};

use crate::scale::px;
use crate::{
    icon::{Icon, IconName},
    motion::{Channel, Curve, Spring, cubic_bezier, ease},
    placement::{measure, opens_upward},
    popover::{Popover, Side},
    theme::{ActiveTheme, mix, popover_shadow, radius},
    typography::TextSize,
};
use super::types::{
    HEADING_HEIGHT, ITEM_DELAY, ITEM_FADE, ITEM_HEIGHT, ITEM_RISE, ITEM_STEP, PANEL_PAD,
    RISE_DAMPING, RISE_STIFFNESS, ROW_GAP, SelectHandler, TYPE_AHEAD,
};
use super::helpers::{
    header_inset, list_height_of, monogram, opens_in, should_light, spring_unit, step_active,
    surface_at, trigger_tone, type_ahead,
};

/// One option: a label, the icon beui shows before it (the model picker's provider mark), and the
/// group it is listed under.
#[derive(Clone)]
pub struct SelectOption {
    pub label: SharedString,
    pub icon: Option<IconName>,
    /// A lab's or an agent's mark before the label, in the theme's own copy.
    pub mark: Option<crate::model_badge::BrandMark>,
    pub group: Option<SharedString>,
}

impl<T: Into<SharedString>> From<T> for SelectOption {
    fn from(label: T) -> Self {
        Self { label: label.into(), icon: None, mark: None, group: None }
    }
}

impl SelectOption {
    pub fn new(label: impl Into<SharedString>, icon: IconName) -> Self {
        Self { label: label.into(), icon: Some(icon), mark: None, group: None }
    }

    pub fn mark(mut self, mark: crate::model_badge::BrandMark) -> Self {
        self.mark = Some(mark);
        self
    }

    /// Lists the option under `group`'s heading. Options of one group must be next to each other.
    pub fn group(mut self, group: impl Into<SharedString>) -> Self {
        self.group = Some(group.into());
        self
    }
}

/// A select's options and which one is chosen.
#[derive(IntoElement)]
pub struct Select {
    pub(super) id: ElementId,
    pub(super) options: Vec<SelectOption>,
    pub(super) selected: Option<usize>,
    placeholder: SharedString,
    disabled: bool,
    pub(super) default_open: bool,
    pub(super) compact: bool,
    pub(super) chevron: bool,
    pub(super) shadow: bool,
    pub(super) panel_width: Option<Pixels>,
    pub(super) upward: bool,
    pub(super) on_change: Option<SelectHandler>,
    pub(super) focus: Option<FocusHandle>,
}

impl Select {
    pub fn new(id: impl Into<ElementId>, options: impl IntoIterator<Item = impl Into<SelectOption>>) -> Self {
        Self {
            id: id.into(),
            options: options.into_iter().map(Into::into).collect(),
            selected: None,
            placeholder: "Select".into(),
            disabled: false,
            default_open: false,
            compact: false,
            chevron: true,
            shadow: true,
            panel_width: None,
            upward: false,
            on_change: None,
            focus: None,
        }
    }

    pub fn selected(mut self, index: Option<usize>) -> Self {
        self.selected = index;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Opens on first render, for demos and screenshots. The user's choice wins after that.
    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// The inline chip trigger PromptInput's model picker uses: `h-8` fixed height, `text-xs`,
    /// transparent until hovered, instead of the full `card`-filled field.
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    /// Hides the trigger's chevron, for a trigger that shows its own affordance instead.
    pub fn chevron(mut self, chevron: bool) -> Self {
        self.chevron = chevron;
        self
    }

    /// Opens the panel above the trigger, for a select at the foot of a pane.
    pub fn upward(mut self, upward: bool) -> Self {
        self.upward = upward;
        self
    }

    /// Turns off the panel's `shadow-lg`, for a picker that sits inside an already-raised surface.
    pub fn shadow(mut self, shadow: bool) -> Self {
        self.shadow = shadow;
        self
    }

    /// Anchors the panel to the trigger's left edge at a fixed width, instead of stretching it to
    /// the trigger's own (possibly narrow) width.
    pub fn panel_width(mut self, width: Pixels) -> Self {
        self.panel_width = Some(width);
        self
    }

    /// The focus handle of the trigger, when the owner wants to focus it or return focus to it.
    pub fn focus_handle(mut self, handle: &FocusHandle) -> Self {
        self.focus = Some(handle.clone());
        self
    }
    /// Called with the chosen index. The select closes itself; the caller stores the choice.
    pub fn on_change(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

/// Open state and every animated value, kept across frames by element id.
struct SelectMotion {
    pub(super) open: bool,
    /// When the surface last opened, for the item stagger.
    opened: Option<Instant>,
    /// Hovered or keyboard-highlighted option.
    pub(super) active: Option<usize>,
    /// The letters typed in the open list, and when the last one came: type-ahead.
    pub(super) typed: String,
    typed_at: Option<Instant>,
    /// Pointer over the trigger itself. The compact chip tints on this, and a press here is not an
    /// outside press, so the trigger alone decides whether the panel toggles.
    trigger_hovered: bool,
    /// 0 the trigger, 1 the panel: the shared layout's spring.
    pub(super) morph: Channel,
    pub(super) tint: Channel,
    /// Per option, 0 at rest and 1 lit (hovered, highlighted, or chosen): beui's `transition-colors`
    /// fade to `bg-muted text-foreground`.
    item_tints: Vec<Channel>,
    pub(super) focus: Option<FocusHandle>,
    content_focus: Option<FocusHandle>,
    /// The trigger's bounds in its last layout, to choose where the panel opens.
    pub(super) anchor: Option<Bounds<Pixels>>,
}

impl SelectMotion {
    pub(super) fn new() -> Self {
        Self {
            open: false,
            opened: None,
            active: None,
            typed: String::new(),
            typed_at: None,
            trigger_hovered: false,
            morph: Channel::new(0.),
            tint: Channel::new(0.),
            item_tints: Vec::new(),
            focus: None,
            content_focus: None,
            anchor: None,
        }
    }

    /// Opens or closes: the surface grows to the panel, or shrinks back, on the morph spring.
    fn set_open(&mut self, open: bool, reduce: bool) {
        if open == self.open {
            return;
        }
        self.open = open;
        self.morph.animate(if open { 1. } else { 0. }, Curve::Spring(Spring::select_morph()), 0., reduce);
        if open {
            self.opened = Some(Instant::now());
        } else {
            // Nothing is highlighted until the arrow keys move through the list again.
            self.active = None;
        }
    }
    fn set_trigger_hovered(&mut self, hovered: bool, reduce: bool) {
        self.trigger_hovered = hovered;
        self.tint.animate(if hovered { 1. } else { 0. }, Curve::Spring(Spring::TINT), 0., reduce);
    }

    /// Moves the highlighted option by one, matching native `<select>`: it stops at the ends instead of
    /// wrapping.
    pub(super) fn step_active(&mut self, delta: i32, len: usize) {
        self.active = step_active(self.active, delta, len);
    }

    /// Fades each option toward lit or unlit. Call every render, so hover, arrow keys, and a new
    /// choice all fade the same way.
    fn retarget_items(&mut self, selected: Option<usize>, len: usize, reduce: bool) {
        self.item_tints.resize_with(len, || Channel::new(0.));
        for (i, tint) in self.item_tints.iter_mut().enumerate() {
            let lit = if selected == Some(i) || self.active == Some(i) { 1. } else { 0. };
            if tint.target() != lit {
                tint.animate(lit, Curve::Spring(Spring::TINT), 0., reduce);
            }
        }
    }

    /// Opacity and rise of option `i` as it comes in. Once in, an option stays in, so a close does not undo it.
    pub(super) fn item(&self, i: usize, reduce: bool) -> (f32, f32) {
        let Some(opened) = self.opened.filter(|_| !reduce) else { return (1., 0.) };
        let t = opened.elapsed().as_secs_f32() - ITEM_DELAY - ITEM_STEP * i as f32;
        if t <= 0. {
            return (0., -ITEM_RISE);
        }
        let fade = cubic_bezier(ease::MOTION_DEFAULT, (t / ITEM_FADE).clamp(0., 1.));
        (fade, -ITEM_RISE * (1. - spring_unit(RISE_STIFFNESS, RISE_DAMPING, t)))
    }
    fn is_moving(&self, items: usize) -> bool {
        self.morph.is_running()
            || self.tint.is_running()
            || self.item_tints.iter().any(|c| c.is_running())
            || self.opened.is_some_and(|at| at.elapsed().as_secs_f32() < opens_in(items) + 0.6)
    }
}

impl Select {
    /// For each option, the heading that opens before it: its group, when that differs from the one
    /// before.
    pub(super) fn headings(&self) -> Vec<Option<SharedString>> {
        let mut last: Option<&SharedString> = None;
        self.options
            .iter()
            .map(|option| {
                let group = option.group.as_ref().filter(|g| last != Some(*g)).cloned();
                last = option.group.as_ref();
                group
            })
            .collect()
    }
}

impl RenderOnce for Select {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let reduce = cx.reduce_motion();
        let headings = self.headings();
        let heading_rows = headings.iter().flatten().count() as f32;
        let list_h = list_height_of(self.options.len(), heading_rows as usize);
        let default_open = self.default_open;
        let motion = window.use_keyed_state(self.id.clone(), cx, move |_, _| {
            let mut m = SelectMotion::new();
            if default_open {
                m.set_open(true, reduce);
            }
            m
        });
        let selected = self.selected;
        let len = self.options.len();
        let given = self.focus.clone();
        let (focus, content_focus) = motion.update(cx, |m, cx| {
            m.retarget_items(selected, len, reduce);
            if given.is_some() {
                m.focus = given;
            }
            let focus = m.focus.get_or_insert_with(|| cx.focus_handle()).clone();
            let content = m.content_focus.get_or_insert_with(|| cx.focus_handle()).clone();
            (focus, content)
        });
        // The panel that covered the trigger is gone and the pointer is still on it: no move came to say so, so the
        // trigger lights now. Without this it stays dull until the pointer stirs.
        let (hovered, anchor_now) = {
            let m = motion.read(cx);
            (m.trigger_hovered, m.anchor)
        };
        if should_light(motion.read(cx).open || motion.read(cx).morph.is_running(), hovered, anchor_now, window.mouse_position()) {
            motion.update(cx, |m, _| m.set_trigger_hovered(true, reduce));
        }
        let m = motion.read(cx);
        if m.is_moving(self.options.len()) {
            window.request_animation_frame();
        }
        // Up when asked, or when the window has no room below the trigger and more above it.
        let window_height = f32::from(window.viewport_size().height);
        let upward = self.upward || m.anchor.is_some_and(|a| opens_upward(a, list_h * crate::scale::zoom(), 0., window_height));
        let theme = cx.theme().clone();
        let open = m.open;
        let p = m.morph.value();
        let tint = m.tint.value();
        // The trigger's size, to begin the surface from: measured, or a guess before the first layout.
        let compact = self.compact;
        // The surface is worked out in design pixels: what was measured is in the window's, so it is brought back.
        let (tw, th) = m.anchor.map_or((self.panel_width.map_or(200., crate::scale::design), if compact { 32. } else { 36. }), |a| {
            (crate::scale::design(a.size.width), crate::scale::design(a.size.height))
        });
        let pw = self.panel_width.map_or(tw, crate::scale::design).max(tw);
        let (surface_w, surface_h) = surface_at((tw, th), pw, list_h, p);
        let shown = open || m.morph.is_running() || p.abs() > 0.002;
        // design preview: remove after Alex picks (the elevation)
        let pill = crate::design_preview::row_tone(&theme, crate::design_preview::panel_fill(&theme, crate::design_preview::elevation(), theme.card));
        let items: Vec<(f32, f32)> = (0..self.options.len()).map(|i| m.item(i, reduce)).collect();
        let item_tints: Vec<f32> = m.item_tints.iter().map(|c| c.value()).collect();

        let selected_option = self.selected.and_then(|i| self.options.get(i).cloned());
        // In a list where some options carry a mark, one without shows its first letter in the mark's place.
        let monograms = self.options.iter().any(|o| o.mark.is_some());
        let has_value = selected_option.is_some();
        let toggle = {
            let motion = motion.clone();
            move |open: bool, _: &mut Window, cx: &mut App| {
                let reduce = cx.reduce_motion();
                motion.update(cx, |m, cx| {
                    let was_open = m.open;
                    m.set_open(open, reduce);
                    // Opening lights the chosen option, so Down and Up move from where the value is.
                    if open && !was_open {
                        m.active = selected;
                        m.typed.clear();
                    }
                    cx.notify();
                });
            }
        };
        let toggle = Rc::new(toggle);

        // One tone for every trigger, the Ghost button's hover; the open surface holds it.
        let held = tint.max(p.clamp(0., 1.));
        let trigger_bg = trigger_tone(&theme, if compact { transparent_black() } else { theme.card }, held);
        let anchor = {
            let motion = motion.clone();
            measure(move |bounds, cx| motion.update(cx, |m, _| m.anchor = Some(bounds)))
        };
        let placeholder = self.placeholder.clone();
        // The row of the trigger, which is also the header of the open surface: one row, so the morph is one surface.
        let face = || {
                div()
                    .flex()
                    .min_w_0()
                    .items_center()
                    .gap(px(6.))
                    .when_some(selected_option.as_ref().and_then(|o| o.icon), |d, icon| {
                        d.child(div().flex_none().text_color(theme.muted_foreground).child(Icon::new(icon).size(px(14.))))
                    })
                    .when_some(selected_option.as_ref().and_then(|o| o.mark.clone()), |d, mark| {
                        d.child(gpui_kit::img(mark.for_theme(theme.appearance)).flex_none().size(px(12.)))
                    })
                    .when(monograms && selected_option.as_ref().is_some_and(|o| o.mark.is_none() && o.icon.is_none()), |d| {
                        d.child(monogram(&selected_option.as_ref().map(|o| o.label.clone()).unwrap_or_default(), 12., &theme))
                    })
                    .child(
                        div()
                            .truncate()
                            .text_color(if has_value { theme.foreground } else { theme.muted_foreground })
                            .child(selected_option.clone().map(|o| o.label).unwrap_or_else(|| placeholder.clone())),
                    )
        };
        let chevron_at = |turn: f32| {
            div().flex_none().text_color(theme.muted_foreground).child(Icon::new(IconName::ChevronDown).size(px(16.)).turn(turn))
        };
        let trigger = div()
            .id("trigger")
            .relative()
            .child(anchor)
            .flex()
            .when(!compact, |d| d.w_full())
            .items_center()
            .justify_between()
            .gap(px(8.))
            .when(!compact, |d| d.px(px(12.)).py(px(8.)))
            .when(compact, |d| d.h(px(32.)).px(px(8.)))
            .rounded(radius::lg())
            .bg(trigger_bg)
            // The surface drawn over it is the trigger while it is open or moving.
            .when(shown, |d| d.opacity(0.))
            .text_size(if compact { TextSize::Xs.font_size() } else { TextSize::Sm.font_size() })
            .line_height(TextSize::Sm.line_height())
            .child(face())
            .when(self.chevron, |d| d.child(chevron_at(0.)))
            .when(!self.disabled, |d| {
                let toggle = toggle.clone();
                let hover_motion = motion.clone();
                let click_motion = motion.clone();
                d.cursor_pointer()
                    // Read the state at click time: the frame this closure was built in may be stale.
                    .on_click(move |_, window, cx| {
                        let open = click_motion.read(cx).open;
                        toggle(!open, window, cx)
                    })
                    .on_hover(move |on, _, cx| {
                        let reduce = cx.reduce_motion();
                        hover_motion.update(cx, |m, cx| {
                            m.set_trigger_hovered(*on, reduce);
                            cx.notify();
                        });
                    })
            });

        let on_change = self.on_change.clone();
        let heading = |group: &SharedString| {
            div()
                .flex()
                .items_end()
                .h(px(HEADING_HEIGHT))
                .px(px(10.))
                .pb(px(4.))
                .text_size(TextSize::Xs.font_size())
                .text_color(theme.muted_foreground)
                .child(group.clone())
                .into_any_element()
        };
        let options = self.options.iter().enumerate().flat_map(|(i, option)| {
            let head = headings[i].as_ref().map(heading);
            let (alpha, rise) = items[i];
            let selected = self.selected == Some(i);
            let lit = item_tints.get(i).copied().unwrap_or(0.);
            let (on_change, toggle, motion, back_to) = (on_change.clone(), toggle.clone(), motion.clone(), focus.clone());
            let row = div()
                .id(("option", i))
                .debug_selector(move || format!("select-option-{i}"))
                .relative()
                .top(px(rise))
                .opacity(alpha)
                .flex()
                .items_center()
                .justify_between()
                .gap(px(8.))
                .h(px(ITEM_HEIGHT))
                .px(px(10.))
                .rounded(radius::md())
                .text_size(TextSize::Sm.font_size())
                .text_color(mix(theme.muted_foreground, theme.foreground, lit))
                .bg(mix(transparent_black(), pill, lit))
                .cursor_pointer()
                .on_hover(move |on, _, cx| {
                    motion.update(cx, |m, cx| {
                        m.active = if *on { Some(i) } else { m.active.filter(|a| *a != i) };
                        cx.notify();
                    })
                })
                .on_click(move |_, window, cx| {
                    if let Some(f) = on_change.as_ref() {
                        f(i, window, cx);
                    }
                    toggle(false, window, cx);
                    // A pick hands focus back to the trigger, so the keys go on working from there.
                    back_to.focus(window, cx);
                })
                .child(
                    div()
                        .flex()
                        .min_w_0()
                        .items_center()
                        .gap(px(8.))
                        .when_some(option.icon, |d, icon| {
                            d.child(div().flex_none().child(Icon::new(icon).size(px(16.))))
                        })
                        .when_some(option.mark.clone(), |d, mark| {
                            d.child(gpui_kit::img(mark.for_theme(theme.appearance)).flex_none().size(px(14.)))
                        })
                        .when(monograms && option.mark.is_none() && option.icon.is_none(), |d| d.child(monogram(&option.label, 14., &theme)))
                        .child(div().truncate().child(option.label.clone())),
                )
                .when(selected, |d| d.child(div().size(px(20.)).flex_none().flex().items_center().justify_center().child(Icon::new(IconName::Check).size(px(16.)).color(theme.foreground))));
            head.into_iter().chain(std::iter::once(row.into_any_element()))
        });

        let pop_id = ElementId::NamedChild(std::sync::Arc::new(self.id.clone()), "pop".into());
        let len = self.options.len();
        // The panel covers what is under it: a click on an option must not also reach that.
        // design preview: remove after Alex picks (the elevation)
        let level = crate::design_preview::elevation();
        let lift = p.clamp(0., 1.);
        let surface_fill = mix(trigger_bg, crate::design_preview::panel_fill(&theme, level, theme.card), lift);
        let surface_shadows: Vec<gpui_kit::BoxShadow> = crate::design_preview::panel_shadows(&theme, level, popover_shadow(&theme))
            .into_iter()
            .map(|mut shadow| {
                shadow.color.a *= lift;
                shadow
            })
            .collect();
        let mut edge = crate::design_preview::panel_edge(&theme, level);
        edge.a *= lift;
        // The header is the trigger's row, at the trigger's end of the surface; the options grow away from it.
        let header = div()
            .id("select-header")
            .debug_selector(|| "select-header".into())
            .absolute()
            .left_0()
            .w_full()
            .h(px(th))
            .when(!upward, |d| d.top_0())
            .when(upward, |d| d.bottom_0())
            .flex()
            .items_center()
            .justify_between()
            .gap(px(8.))
            // The row's inset eases from the trigger's to the options', so header and rows line up once open.
            .px(px(header_inset(if compact { 8. } else { 12. }, p)))
            .text_size(if compact { TextSize::Xs.font_size() } else { TextSize::Sm.font_size() })
            .line_height(TextSize::Sm.line_height())
            .cursor_pointer()
            .on_click({
                let (toggle, motion) = (toggle.clone(), motion.clone());
                move |_, window, cx| {
                    let open = motion.read(cx).open;
                    toggle(!open, window, cx)
                }
            })
            .child(div().flex().min_w_0().debug_selector(|| "select-header-face".into()).child(face()))
            .when(self.chevron, |d| d.child(chevron_at(0.5 * p)));
        let list = div()
            .absolute()
            .left_0()
            .w_full()
            .h(px(list_h))
            .when(!upward, |d| d.top(px(th)))
            .when(upward, |d| d.bottom(px(th)))
            .flex()
            .flex_col()
            .gap(px(ROW_GAP))
            .p(px(PANEL_PAD))
            .children(options);
        let edge_line = div().absolute().inset_0().rounded(radius::lg()).border_1().border_color(edge);
        // The surface covers what is under it: a click on an option must not also reach that.
        let panel = div()
            .id("panel")
            .debug_selector(|| "select-surface".into())
            .relative()
            .w(px(surface_w))
            .h(px(surface_h))
            .rounded(radius::lg())

            .overflow_hidden()
            .bg(surface_fill)
            .when(self.shadow, |d| d.shadow(surface_shadows))
            .track_focus(&content_focus)
            // The base opens and focuses the panel on these; moving the highlight within it is ours.
            .on_action({
                let motion = motion.clone();
                move |_: &SelectUp, _, cx| {
                    motion.update(cx, |m, cx| {
                        m.step_active(-1, len);
                        cx.notify();
                    });
                }
            })
            .on_action({
                let motion = motion.clone();
                move |_: &SelectDown, _, cx| {
                    motion.update(cx, |m, cx| {
                        m.step_active(1, len);
                        cx.notify();
                    });
                }
            })
            .on_action({
                let motion = motion.clone();
                move |_: &SelectFirst, _, cx| {
                    motion.update(cx, |m, cx| {
                        m.active = (len > 0).then_some(0);
                        cx.notify();
                    });
                }
            })
            .on_action({
                let motion = motion.clone();
                move |_: &SelectLast, _, cx| {
                    motion.update(cx, |m, cx| {
                        m.active = len.checked_sub(1);
                        cx.notify();
                    });
                }
            })
            .on_key_down({
                let motion = motion.clone();
                let labels: Vec<SharedString> = self.options.iter().map(|o| o.label.clone()).collect();
                move |event, _, cx| {
                    let keys = &event.keystroke;
                    let Some(letter) = keys.key_char.as_deref().filter(|c| c.chars().count() == 1 && *c != " ") else { return };
                    if keys.modifiers.control || keys.modifiers.platform || keys.modifiers.alt {
                        return;
                    }
                    motion.update(cx, |m, cx| {
                        if m.typed_at.is_none_or(|at| at.elapsed() > TYPE_AHEAD) {
                            m.typed.clear();
                        }
                        m.typed.push_str(letter);
                        m.typed_at = Some(Instant::now());
                        if let Some(i) = type_ahead(&labels, m.active, &m.typed) {
                            m.active = Some(i);
                        }
                        cx.notify();
                    });
                }
            })
            .child(list)
            .child(header)
            .child(edge_line);

        let anchor_bounds = motion.read(cx).anchor;
        let popover = Popover::new(pop_id)
            .open(open)
            .shown(shown)
            .anchor(anchor_bounds)
            .switchable()
            .side(if upward { Side::CoverAbove } else { Side::CoverBelow })
            .height(list_h)
            .width(px(pw))
            .return_focus(&focus)
            .panel_focus(&content_focus)
            .on_close({
                let toggle = toggle.clone();
                move |window, cx| toggle(false, window, cx)
            })
            .child(panel);
        let toggle_open = toggle.clone();
        let confirm_motion = motion.clone();
        let confirm_on_change = self.on_change.clone();
        let confirm_focus = focus.clone();
        BaseSelect::new(self.id)
            .open(open)
            .disabled(self.disabled)
            .focus_handle(&focus)
            .content_focus_handle(&content_focus)
            .on_open_change(move |open, window, cx| toggle_open(open, window, cx))
            .on_confirm(move |window, cx| {
                let Some(i) = confirm_motion.read(cx).active else { return };
                if let Some(f) = confirm_on_change.as_ref() {
                    f(i, window, cx);
                }
                toggle(false, window, cx);
                // After the base's own refocus of the panel, which runs when this returns.
                let back = confirm_focus.clone();
                window.defer(cx, move |window, cx| back.focus(window, cx));
            })
            .relative()
            .when(!compact, |d| d.w_full())
            .when(self.disabled, |d| d.opacity(0.5))
            .child(trigger)
            .child(popover)
    }
}
