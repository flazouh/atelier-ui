use std::{collections::HashMap, sync::Arc};

use gpui_kit::{
    App,
    AppContext,
    Bounds,
    Context,
    ElementId,
    Entity,
    EventEmitter,
    FocusHandle,
    Focusable,
    FontWeight,
    Hsla,
    InteractiveElement,
    IntoElement,
    KeyDownEvent,
    MouseButton,
    ParentElement,
    Pixels,
    Point,
    Render,
    ScrollHandle,
    SharedString,
    StatefulInteractiveElement,
    Styled,
    Subscription,
    Window,
    component::input::{Input, InputEvent, InputState},
    div,
    prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    icon::{Icon, IconName},
    layout_motion::shifted,
    motion::{Channel, Curve, Spring, ease},
    placement::measure,
    popover::Popover,
    theme::{ActiveTheme, mix},
    typography::TextSize,
};
use super::types::{
    BORDER, BORDER_STRONG, CHIP_GAP, CHIP_HEIGHT, CHIP_RADIUS, CHIP_RISE, ENTER_FADE, FIELD_GAP,
    FIELD_HEIGHT, FIELD_INPUT_BOX_MIN, FIELD_INPUT_MIN, FIELD_PAD_X, FIELD_PAD_Y, FIELD_RADIUS,
    GROUP_PAD, LIST_MAX, LIST_PAD, MultiSelectEvent, REMOVE, RING, ROW, SIDE_OFFSET, WIPE,
};
use super::helpers::{active, content_height, move_active, tracked, visible};

/// One option: what it is, what it says, the group it is listed under, and words that also find it.
#[derive(Clone, Debug)]
pub struct MultiOption {
    pub value: SharedString,
    pub label: SharedString,
    pub group: Option<SharedString>,
    pub keywords: Vec<SharedString>,
    pub disabled: bool,
    /// A 10px dot of this colour before the label.
    pub dot: Option<Hsla>,
}

impl MultiOption {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self { value: value.into(), label: label.into(), group: None, keywords: Vec::new(), disabled: false, dot: None }
    }

    pub fn group(mut self, group: impl Into<SharedString>) -> Self {
        self.group = Some(group.into());
        self
    }

    pub fn keywords(mut self, words: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.keywords = words.into_iter().map(Into::into).collect();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn dot(mut self, color: impl Into<Hsla>) -> Self {
        self.dot = Some(color.into());
        self
    }
}

/// A chip on its way out, drawn where it was.
pub(super) struct Leaving {
    pub(super) value: SharedString,
    pub(super) label: SharedString,
    pub(super) bounds: Bounds<Pixels>,
    pub(super) wipe: Channel,
}

pub struct MultiSelect {
    pub(super) id: ElementId,
    pub(super) options: Vec<MultiOption>,
    pub(super) values: Vec<SharedString>,
    pub(super) open: bool,
    pub(super) placeholder: SharedString,
    pub(super) empty: SharedString,
    pub(super) disabled: bool,
    pub(super) input: Entity<InputState>,
    pub(super) query: SharedString,
    /// The row the pointer or the keys last moved to, and the query it was placed under.
    pub(super) cursor: Option<(SharedString, SharedString)>,
    pub(super) trigger: Option<Bounds<Pixels>>,
    pub(super) chips: HashMap<SharedString, Bounds<Pixels>>,
    pub(super) entering: HashMap<SharedString, Channel>,
    pub(super) leaving: Vec<Leaving>,
    pub(super) rows: HashMap<SharedString, Bounds<Pixels>>,
    pub(super) list_origin: Option<Point<Pixels>>,
    pub(super) list_view: Option<Bounds<Pixels>>,
    pub(super) scroll: ScrollHandle,
    pub(super) highlight: Channel,
    pub(super) highlighted: Option<SharedString>,
    /// The panel: its height, and its gap from the field.
    pub(super) height: Channel,
    pub(super) gap: Channel,
    pub(super) hovered: bool,
    pub(super) _subscriptions: Vec<Subscription>,
}

impl EventEmitter<MultiSelectEvent> for MultiSelect {}

impl Focusable for MultiSelect {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.input.focus_handle(cx)
    }
}

impl MultiSelect {
    pub fn new(id: impl Into<ElementId>, options: Vec<MultiOption>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| InputState::new(window, cx).placeholder(""));
        let subscriptions = vec![cx.subscribe_in(&input, window, |this: &mut Self, input, event: &InputEvent, window, cx| match event {
            InputEvent::Change => {
                this.query = input.read(cx).value();
                this.set_open(true, window, cx);
                cx.notify();
            }
            InputEvent::Focus => this.set_open(true, window, cx),
            _ => {}
        })];
        Self {
            id: id.into(),
            options,
            values: Vec::new(),
            open: false,
            placeholder: "Select options".into(),
            empty: "No options found.".into(),
            disabled: false,
            input,
            query: SharedString::default(),
            cursor: None,
            trigger: None,
            chips: HashMap::new(),
            entering: HashMap::new(),
            leaving: Vec::new(),
            rows: HashMap::new(),
            list_origin: None,
            list_view: None,
            scroll: ScrollHandle::new(),
            highlight: Channel::new(0.),
            highlighted: None,
            height: Channel::new(0.),
            gap: Channel::new(0.),
            hovered: false,
            _subscriptions: subscriptions,
        }
    }

    pub fn placeholder(mut self, words: impl Into<SharedString>) -> Self {
        self.placeholder = words.into();
        self
    }

    /// What the list says when nothing matches.
    pub fn empty(mut self, words: impl Into<SharedString>) -> Self {
        self.empty = words.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn with_values(mut self, values: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.values = values.into_iter().map(Into::into).collect();
        self
    }

    pub fn values(&self) -> &[SharedString] {
        &self.values
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub(super) fn label_of(&self, value: &SharedString) -> SharedString {
        self.options.iter().find(|o| &o.value == value).map_or_else(|| value.clone(), |o| o.label.clone())
    }

    /// Opens or closes the list. Closing clears the field, and hands it back focus when asked.
    pub fn set_open(&mut self, open: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled && open || self.open == open {
            return;
        }
        self.open = open;
        let reduce = cx.reduce_motion();
        if open {
            let spring = Curve::Spring(Spring::select_morph());
            self.gap.animate(SIDE_OFFSET, spring, 0., reduce);
        } else {
            self.gap.animate(0., Curve::Spring(Spring::select_morph()), 0., reduce);
            self.height.animate(0., Curve::Spring(Spring::select_morph()), 0., reduce);
            self.query = SharedString::default();
            self.cursor = None;
            self.input.update(cx, |input, cx| input.set_value("", window, cx));
        }
        cx.notify();
    }

    pub(super) fn commit(&mut self, values: Vec<SharedString>, window: &mut Window, cx: &mut Context<Self>) {
        // A chip that goes leaves by a wipe, drawn where it was.
        let reduce = cx.reduce_motion();
        for gone in self.values.iter().filter(|v| !values.contains(v)) {
            if let (Some(bounds), false) = (self.chips.get(gone).copied(), reduce) {
                let mut wipe = Channel::new(0.);
                wipe.animate(1., Curve::Ease(WIPE, ease::OUT), 0., false);
                self.leaving.push(Leaving { value: gone.clone(), label: self.label_of(gone), bounds, wipe });
            }
        }
        for new in values.iter().filter(|v| !self.values.contains(v)) {
            let mut enter = Channel::new(0.);
            enter.animate(1., Curve::Ease(ENTER_FADE, ease::OUT), 0., reduce);
            self.entering.insert(new.clone(), enter);
        }
        self.values = values.clone();
        cx.emit(MultiSelectEvent::Change(values));
        let _ = window;
        cx.notify();
    }

    /// Chooses the option if it is not chosen, else lets it go. The field is cleared and the list stays open.
    pub fn toggle(&mut self, value: &SharedString, window: &mut Window, cx: &mut Context<Self>) {
        if self.options.iter().any(|o| &o.value == value && o.disabled) {
            return;
        }
        let mut values = self.values.clone();
        if let Some(at) = values.iter().position(|v| v == value) {
            values.remove(at);
        } else {
            values.push(value.clone());
        }
        self.query = SharedString::default();
        self.input.update(cx, |input, cx| input.set_value("", window, cx));
        self.commit(values, window, cx);
        self.input.focus_handle(cx).focus(window, cx);
    }

    pub(super) fn remove(&mut self, value: &SharedString, window: &mut Window, cx: &mut Context<Self>) {
        let values: Vec<SharedString> = self.values.iter().filter(|v| *v != value).cloned().collect();
        self.commit(values, window, cx);
        self.input.focus_handle(cx).focus(window, cx);
    }

    pub(super) fn active_value(&self) -> Option<SharedString> {
        let shown = visible(&self.options, &self.query);
        active(self.cursor.as_ref(), &self.query, &shown, &self.values).cloned()
    }

    pub(super) fn step(&mut self, direction: i32, window: &mut Window, cx: &mut Context<Self>) {
        if !self.open {
            self.set_open(true, window, cx);
            return;
        }
        let shown = visible(&self.options, &self.query);
        let from = active(self.cursor.as_ref(), &self.query, &shown, &self.values).cloned();
        if let Some(next) = move_active(from.as_ref(), &shown, direction).cloned() {
            self.cursor = Some((next, self.query.clone()));
            cx.notify();
        }
    }

    pub(super) fn jump(&mut self, last: bool, cx: &mut Context<Self>) {
        let shown = visible(&self.options, &self.query);
        let enabled: Vec<&&MultiOption> = shown.iter().filter(|o| !o.disabled).collect();
        let pick = if last { enabled.last() } else { enabled.first() };
        if let Some(option) = pick {
            self.cursor = Some((option.value.clone(), self.query.clone()));
            cx.notify();
        }
    }

    pub(super) fn key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        match event.keystroke.key.as_str() {
            "backspace" if self.query.is_empty() && !self.values.is_empty() => {
                let last = self.values.last().cloned().unwrap_or_default();
                self.remove(&last, window, cx);
            }
            "down" => self.step(1, window, cx),
            "up" => self.step(-1, window, cx),
            "home" if self.open => self.jump(false, cx),
            "end" if self.open => self.jump(true, cx),
            "enter" => {
                if self.open {
                    if let Some(value) = self.active_value() {
                        self.toggle(&value, window, cx);
                    }
                } else {
                    self.set_open(true, window, cx);
                }
            }
            "escape" if self.open => self.set_open(false, window, cx),
            _ => return,
        }
        cx.stop_propagation();
    }
}

impl Render for MultiSelect {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let reduce = cx.reduce_motion();
        let this = cx.entity().downgrade();
        let input_focus = self.input.focus_handle(cx);
        let focused = input_focus.contains_focused(window, cx) || self.open;
        let dark = theme.appearance == crate::theme::Appearance::Dark;
        let border = theme.foreground.opacity(if dark { BORDER - 0.01 } else { BORDER });
        let strong = theme.foreground.opacity(if dark { BORDER_STRONG - 0.02 } else { BORDER_STRONG });

        // Motion that has run out is dropped.
        self.leaving.retain(|l| l.wipe.is_running());
        self.entering.retain(|_, c| c.is_running());
        let shown = visible(&self.options, &self.query);
        let want = content_height(&shown).min(LIST_MAX);
        if self.open && (self.height.target() - want).abs() > 0.5 {
            self.height.animate(want, Curve::Spring(Spring::select_morph()), 0., reduce);
        }
        let active_value = self.active_value();
        if self.open && self.highlighted != active_value && let (Some(now), Some(origin)) = (active_value.as_ref().and_then(|v| self.rows.get(v)), self.list_origin) {
            let y = f32::from(now.top() - origin.y);
            if self.highlighted.is_none() || reduce {
                self.highlight = Channel::new(y);
            } else {
                self.highlight.animate(y, Curve::Spring(Spring::LAYOUT), 0., false);
            }
            self.highlighted = active_value.clone();
        }
        if !self.open {
            self.highlighted = None;
        }
        let animating = self.height.is_running() || self.gap.is_running() || self.highlight.is_running() || !self.leaving.is_empty() || !self.entering.is_empty();
        if animating {
            window.request_animation_frame();
        }
        let panel_open = self.open || self.height.value() > 0.5;

        // ---- the field ----
        let chips = self.values.iter().map(|value| {
            let label = self.label_of(value);
            let enter = self.entering.get(value).map_or(1., |c| c.value().clamp(0., 1.));
            let measured = {
                let (this, value) = (this.clone(), value.clone());
                measure(move |b, cx| {
                    this.update(cx, |s, _| {
                        s.chips.insert(value.clone(), b);
                    })
                    .ok();
                })
            };
            let (remove_this, remove_value) = (this.clone(), value.clone());
            shifted(
                ElementId::NamedChild(Arc::new(self.id.clone()), format!("chip-{value}").into()),
                div()
                    .debug_selector({
                        let value = value.clone();
                        move || format!("multi-chip-{value}")
                    })
                    .relative()
                    .top(px(CHIP_RISE * (1. - enter)))
                    .opacity(enter)
                    .flex_none()
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .h(px(CHIP_HEIGHT))
                    .pl(px(8.))
                    .pr(px(4.))
                    .rounded(px(CHIP_RADIUS))
                    .bg(theme.card)
                    .text_size(TextSize::Xs.font_size())
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.foreground)
                    .child(measured)
                    .child(div().flex_none().whitespace_nowrap().child(label.clone()))
                    .child(
                        div()
                            .id(ElementId::NamedChild(Arc::new(self.id.clone()), format!("remove-{value}").into()))
                            .debug_selector({
                                let value = value.clone();
                                move || format!("multi-remove-{value}")
                            })
                            .flex_none()
                            .size(px(REMOVE))
                            .rounded(px(6.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(theme.muted_foreground)
                            .cursor_pointer()
                            .hover(|s| s.bg(theme.foreground.opacity(0.1)).text_color(theme.foreground))
                            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                            .on_click(move |_, window, cx| {
                                cx.stop_propagation();
                                remove_this.update(cx, |s, cx| s.remove(&remove_value, window, cx)).ok();
                            })
                            .child(Icon::new(IconName::Close).size(px(12.))),
                    ),
            )
            .into_any_element()
        });
        let ghosts = self.leaving.iter().map(|l| {
            let origin = self.trigger.map_or(l.bounds.origin, |t| t.origin);
            let (w, p) = (f32::from(l.bounds.size.width), l.wipe.value().clamp(0., 1.));
            div()
                .absolute()
                .left(l.bounds.origin.x - origin.x + px(w * p))
                .top(l.bounds.origin.y - origin.y)
                .w(px(w * (1. - p)))
                .h(l.bounds.size.height)
                .overflow_hidden()
                .child(
                    div()
                        .absolute()
                        .left(px(-w * p))
                        .w(px(w))
                        .h(px(CHIP_HEIGHT))
                        .flex()
                        .items_center()
                        .px(px(8.))
                        .rounded(px(CHIP_RADIUS))
                        .bg(theme.card)
                        .text_size(TextSize::Xs.font_size())
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.foreground)
                        .child(l.label.clone()),
                )
                .id(ElementId::NamedChild(Arc::new(self.id.clone()), format!("ghost-{}", l.value).into()))
        });
        let placeholder = (self.values.is_empty() && !self.open).then(|| div().text_color(theme.muted_foreground).child(self.placeholder.clone()));

        let (focus_click, hover_this) = (input_focus.clone(), this.clone());
        let field = div()
            .id(self.id.clone())
            .debug_selector(|| "multi-field".into())
            .relative()
            .flex()
            .items_center()
            .gap(px(FIELD_GAP))
            .w_full()
            .min_w(px(208.))
            .min_h(px(FIELD_HEIGHT))
            .px(px(FIELD_PAD_X))
            .py(px(FIELD_PAD_Y))
            .rounded(px(FIELD_RADIUS))
            .bg(theme.background)
            .border_1()
            .border_color(mix(border, strong, if self.hovered || focused { 1. } else { 0. }))
            .when(focused, |d| d.shadow(vec![gpui_kit::BoxShadow { color: theme.foreground.opacity(RING), offset: gpui_kit::point(px(0.), px(0.)), blur_radius: px(0.), spread_radius: px(2.), inset: false }]))
            .text_size(TextSize::Sm.font_size())
            .text_color(theme.foreground)
            .cursor_text()
            .when(self.disabled, |d| d.opacity(0.5))
            .on_hover(move |on, _, cx| {
                hover_this.update(cx, |s, cx| {
                    s.hovered = *on;
                    cx.notify();
                })
                .ok();
            })
            .capture_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| this.key(event, window, cx)))
            .on_mouse_down(MouseButton::Left, {
                let this = this.clone();
                move |_, window, cx| {
                    focus_click.focus(window, cx);
                    this.update(cx, |s, cx| s.set_open(true, window, cx)).ok();
                }
            })
            .child(measure({
                let this = this.clone();
                move |b, cx| {
                    this.update(cx, |s, _| s.trigger = Some(b)).ok();
                }
            }))
            .children(ghosts)
            .child(
                div()
                    
                    .flex()
                    .flex_1()
                    .min_w_0()
                    .flex_wrap()
                    .items_center()
                    .gap(px(CHIP_GAP))
                    .children(placeholder)
                    .children(chips)
                    .child(
                        div()
                            .flex()
                            .flex_1()
                            .items_center()
                            .min_w(px(FIELD_INPUT_BOX_MIN))
                            .child(div().h(px(CHIP_HEIGHT)).flex_1().min_w(px(FIELD_INPUT_MIN)).child(Input::new(&self.input).appearance(false).bordered(false).disabled(self.disabled))),
                    ),
            )
            .child(div().flex_none().text_color(theme.muted_foreground).child(Icon::new(IconName::UnfoldMore).size(px(16.))));

        // ---- the list ----
        let list_measure = {
            let this = this.clone();
            measure(move |b, cx| {
                this.update(cx, |s, _| {
                    s.list_origin = Some(b.origin);
                    s.list_view = Some(b);
                })
                .ok();
            })
        };
        let mut groups: Vec<(Option<SharedString>, Vec<&MultiOption>)> = Vec::new();
        for option in &shown {
            match groups.last_mut() {
                Some((g, rows)) if *g == option.group => rows.push(option),
                _ => groups.push((option.group.clone(), vec![option])),
            }
        }
        let glide = self.highlight.value();
        let wash = self.highlighted.as_ref().and_then(|v| self.rows.get(v)).map(|b| (glide, f32::from(b.size.height), f32::from(b.size.width), f32::from(b.left() - self.list_origin.map_or(b.left(), |o| o.x))));
        let sections = groups.into_iter().enumerate().map(|(gi, (group, rows))| {
            let rows = rows.into_iter().map(|option| {
                let selected = self.values.contains(&option.value);
                let is_active = active_value.as_ref() == Some(&option.value);
                let tick = if selected { 1. } else { 0. };
                let value = option.value.clone();
                let (pick_this, hover_this, measure_this) = (this.clone(), this.clone(), this.clone());
                let (pick_value, hover_value, measure_value) = (value.clone(), value.clone(), value.clone());
                div()
                    .id(ElementId::NamedChild(Arc::new(self.id.clone()), format!("option-{value}").into()))
                    .debug_selector({
                        let value = value.clone();
                        move || format!("multi-option-{value}")
                    })
                    .relative()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .w_full()
                    .h(px(ROW))
                    .px(px(8.))
                    .rounded(px(CHIP_RADIUS))
                    .text_size(TextSize::Sm.font_size())
                    .text_color(if is_active || selected { theme.foreground } else { theme.muted_foreground })
                    .when(option.disabled, |d| d.opacity(0.45))
                    .when(!option.disabled, |d| d.cursor_pointer())
                    .child(measure(move |b, cx| {
                        measure_this
                            .update(cx, |s, _| {
                                s.rows.insert(measure_value.clone(), b);
                            })
                            .ok();
                    }))
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_mouse_move(move |_, _, cx| {
                        hover_this
                            .update(cx, |s, cx| {
                                if s.active_value().as_ref() != Some(&hover_value) {
                                    s.cursor = Some((hover_value.clone(), s.query.clone()));
                                    cx.notify();
                                }
                            })
                            .ok();
                    })
                    .on_click(move |_, window, cx| {
                        pick_this.update(cx, |s, cx| s.toggle(&pick_value, window, cx)).ok();
                    })
                    .child(
                        div().flex().flex_1().min_w_0().items_center().gap(px(10.)).children(option.dot.map(|c| div().flex_none().size(px(10.)).rounded_full().bg(c))).child(div().truncate().child(option.label.clone())),
                    )
                    .child(div().flex_none().size(px(20.)).flex().items_center().justify_center().text_color(theme.foreground).opacity(tick).child(Icon::new(IconName::Check).size(px(16.))))
            });
            div()
                .flex()
                .flex_col()
                .py(px(GROUP_PAD))
                .id(ElementId::NamedChild(Arc::new(self.id.clone()), format!("group-{gi}").into()))
                .children(group.map(|g| div().w_full().px(px(8.)).py(px(6.)).child(tracked(&g, 10.88, theme.muted_foreground).into_any_element())))
                .children(rows)
        });
        let list = div()
            .id(ElementId::NamedChild(Arc::new(self.id.clone()), "list".into()))
            .relative()
            .max_h(px(LIST_MAX))
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .p(px(LIST_PAD))
            .child(list_measure)
            .children(wash.map(|(y, h, w, x)| div().absolute().top(px(y)).left(px(x)).w(px(w)).h(px(h)).rounded(px(CHIP_RADIUS)).bg(theme.card)))
            .children(sections)
            .when(shown.is_empty(), |d| {
                d.child(div().px(px(12.)).py(px(32.)).flex().justify_center().text_size(TextSize::Sm.font_size()).text_color(theme.muted_foreground).child(self.empty.clone()))
            });
        let surface = div()
            .w_full()
            .h(px(self.height.value().max(0.)))
            .overflow_hidden()
            .rounded(px(FIELD_RADIUS))
            .border_1()
            .border_color(border)
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(list);

        let close_this = this.clone();
        let popover = Popover::new(ElementId::NamedChild(Arc::new(self.id.clone()), "panel".into()))
            .open(self.open)
            .shown(panel_open)
            .anchor(self.trigger)
            .switchable()
            .gap(self.gap.value())
            .width(self.trigger.map_or(px(0.), |t| t.size.width))
            .height(want + 2.)
            .hole()
            .keep_focus()
            .on_close(move |window, cx| {
                close_this.update(cx, |s, cx| s.set_open(false, window, cx)).ok();
            })
            .child(surface);
        div().w_full().flex().flex_col().child(field).child(popover)
    }
}
