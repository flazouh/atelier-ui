use std::rc::Rc;

use gpui_kit::{
    AnyElement, App, Bounds, ElementId, FontWeight, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Pixels, RenderOnce, ScrollHandle, SharedString, StatefulInteractiveElement,
    Styled, Window, div, point, prelude::FluentBuilder,
};

use super::helpers::{check_at, scroll_to_show};
use super::types::{
    CHECK_SECONDS, ComboEntry, ComboStyle, LINE, MAX_HEIGHT, Pick, ROW_GAP_BETWEEN, TEXT,
};
use crate::scale::px;
use crate::{
    icon::{Icon, IconName},
    motion::{Animated, Channel, Curve, FrameClock, ease},
    placement::measure,
    theme::ActiveTheme,
    typography::FONT_FAMILY,
};

pub struct ComboRow {
    label: SharedString,
    detail: Option<SharedString>,
    leading: Option<AnyElement>,
    pub(super) selected: bool,
    pub(super) disabled: bool,
    selector: Option<String>,
}

impl ComboRow {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            detail: None,
            leading: None,
            selected: false,
            disabled: false,
            selector: None,
        }
    }

    /// Muted mono words after the label, which take the room the label leaves.
    pub fn detail(mut self, detail: impl Into<SharedString>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// A mark before the words.
    pub fn leading(mut self, element: impl IntoElement) -> Self {
        self.leading = Some(element.into_any_element());
        self
    }

    /// The row is chosen: it wears a check.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn debug_name(mut self, name: impl Into<String>) -> Self {
        self.selector = Some(name.into());
        self
    }
}

struct State {
    rects: Vec<Option<Bounds<Pixels>>>,
    content: Option<Bounds<Pixels>>,
    pub(super) view: Option<Bounds<Pixels>>,
    pub(super) top: Animated,
    pub(super) height: Animated,
    seeded: bool,
    clock: FrameClock,
    pub(super) scroll: ScrollHandle,
    last_active: Option<usize>,
    last_key: Option<u64>,
    checks: Vec<Channel>,
}

#[derive(IntoElement)]
pub struct ComboList {
    id: ElementId,
    pub(super) entries: Vec<ComboEntry>,
    pub(super) active: Option<usize>,
    padded: bool,
    pub(super) style: ComboStyle,
    checks: bool,
    max_height: f32,
    scroll_key: Option<u64>,
    pub(super) empty: Option<SharedString>,
    pub(super) on_pick: Option<Pick>,
    pub(super) on_hover: Option<Pick>,
    footer: Vec<AnyElement>,
    selector: Option<&'static str>,
}

impl ComboList {
    pub fn new(id: impl Into<ElementId>, entries: impl IntoIterator<Item = ComboEntry>) -> Self {
        Self {
            id: id.into(),
            entries: entries.into_iter().collect(),
            active: None,
            padded: true,
            style: ComboStyle::default(),
            checks: true,
            max_height: MAX_HEIGHT,
            scroll_key: None,
            empty: None,
            on_pick: None,
            on_hover: None,
            footer: Vec::new(),
            selector: None,
        }
    }

    /// The active entry, an index into the entries (a row).
    pub fn active(mut self, active: Option<usize>) -> Self {
        self.active = active;
        self
    }

    /// Whether the list has its own 6px of padding (the default).
    pub fn padded(mut self, padded: bool) -> Self {
        self.padded = padded;
        self
    }

    pub fn style(mut self, style: ComboStyle) -> Self {
        self.style = style;
        self
    }

    /// Whether a chosen row wears a check at its end (the default). A list that never chooses has no use for the room.
    pub fn checks(mut self, checks: bool) -> Self {
        self.checks = checks;
        self
    }

    /// The tallest the list gets before it scrolls.
    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = height;
        self
    }

    /// The list goes back to the top when this changes, such as a hash of the filter's words.
    pub fn scroll_key(mut self, key: u64) -> Self {
        self.scroll_key = Some(key);
        self
    }

    /// What to say when there is no row.
    pub fn empty(mut self, words: impl Into<SharedString>) -> Self {
        self.empty = Some(words.into());
        self
    }

    pub fn on_pick(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_pick = Some(Rc::new(f));
        self
    }

    pub fn on_hover(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_hover = Some(Rc::new(f));
        self
    }

    /// Something under the rows, inside the scroll: a note, a status.
    pub fn footer(mut self, element: impl IntoElement) -> Self {
        self.footer.push(element.into_any_element());
        self
    }

    pub fn debug_name(mut self, name: &'static str) -> Self {
        self.selector = Some(name);
        self
    }
}

impl RenderOnce for ComboList {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let reduce = cx.reduce_motion();
        let count = self.entries.len();
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| State {
            rects: Vec::new(),
            content: None,
            view: None,
            top: Animated::new(self.style.spring(), 0.),
            height: Animated::new(self.style.spring(), 0.),
            seeded: false,
            clock: FrameClock::default(),
            scroll: ScrollHandle::new(),
            last_active: None,
            last_key: None,
            checks: Vec::new(),
        });
        let scroll = state.update(cx, |s, _| {
            s.rects.resize(count, None);
            s.scroll.clone()
        });
        let (glide, moving, checks) = state.update(cx, |s, _| {
            let target = self
                .active
                .and_then(|i| s.rects.get(i).copied().flatten())
                .zip(s.content)
                .map(|(r, c)| {
                    (
                        crate::scale::design(r.origin.y - c.origin.y),
                        crate::scale::design(r.size.height),
                        crate::scale::design(r.origin.x - c.origin.x),
                        crate::scale::design(r.size.width),
                    )
                });
            let mut moving = false;
            if s.last_key != self.scroll_key {
                s.last_key = self.scroll_key;
                s.scroll.set_offset(point(px(0.), px(0.)));
            }
            match target {
                Some((y, h, ..)) => {
                    if !s.seeded {
                        s.top = Animated::new(self.style.spring(), y);
                        s.height = Animated::new(self.style.spring(), h);
                        s.seeded = true;
                    } else {
                        s.top.set_target(y);
                        s.height.set_target(h);
                    }
                    // Keep the active row in sight.
                    if s.last_active != self.active
                        && let Some(view) = s.view
                        && let Some(to) = scroll_to_show(
                            -f32::from(s.scroll.offset().y),
                            f32::from(view.size.height),
                            y,
                            y + h,
                        )
                    {
                        s.scroll.set_offset(point(px(0.), px(-to)));
                    }
                }
                None => {
                    if self.active.is_none() {
                        s.seeded = false;
                    } else {
                        moving = true;
                    }
                }
            }
            s.last_active = self.active;
            let dt = s.clock.tick();
            moving |= s.top.step(dt, reduce) | s.height.step(dt, reduce);
            // The checks.
            s.checks.resize_with(count, || Channel::new(1.));
            let mut shown = Vec::with_capacity(count);
            for (i, entry) in self.entries.iter().enumerate() {
                let want = matches!(entry, ComboEntry::Row(r) if r.selected);
                let to = f32::from(u8::from(want));
                if s.checks[i].target() != to {
                    s.checks[i].animate(to, Curve::Ease(CHECK_SECONDS, ease::OUT), 0., reduce);
                }
                moving |= s.checks[i].is_running();
                shown.push(s.checks[i].value());
            }
            if !moving {
                s.clock.rest();
            }
            (
                target.map(|(_, _, x, w)| (s.top.value(), s.height.value(), x, w)),
                moving,
                shown,
            )
        });
        if moving {
            window.request_animation_frame();
        }
        let keyboard = window.last_input_was_keyboard();
        let _ = keyboard;

        let (pick, hover) = (self.on_pick.clone(), self.on_hover.clone());
        let pill_radius = self.style.radius();
        let pill_fill = self.style.fill(&theme);
        let rows: Vec<AnyElement> = self
            .entries
            .into_iter()
            .enumerate()
            .map(|(i, entry)| match entry {
                ComboEntry::Group(words) => div()
                    .w_full()
                    .px(px(8.))
                    .py(px(6.))
                    .text_size(px(10.88))
                    .line_height(px(16.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.muted_foreground)
                    .child(words.to_uppercase())
                    .into_any_element(),
                ComboEntry::Separator => div()
                    .mx(px(-4.))
                    .my(px(4.))
                    .h(px(1.))
                    .bg(crate::text_input::edge(&theme, theme.background))
                    .into_any_element(),
                ComboEntry::Row(row) => {
                    let active = self.active == Some(i);
                    let ink = if active {
                        theme.foreground
                    } else {
                        theme.muted_foreground
                    };
                    let report = {
                        let state = state.clone();
                        move |b: Bounds<Pixels>, cx: &mut App| {
                            state.update(cx, |s, _| s.rects[i] = Some(b))
                        }
                    };
                    let (scale, opacity) = check_at(checks[i]);
                    let (pick, hover) = (pick.clone(), hover.clone());
                    div()
                        .id(ElementId::NamedInteger("combo-row".into(), i as u64))
                        .relative()
                        .flex()
                        .items_center()
                        .gap(px(self.style.gap()))
                        .w_full()
                        .px(px(self.style.row_x()))
                        .py(px(self.style.row_y()))
                        .rounded(pill_radius)
                        .text_size(px(TEXT))
                        .line_height(px(LINE))
                        .text_color(ink)
                        .when(row.disabled, |d| d.opacity(0.45))
                        .when(!row.disabled && (pick.is_some() || hover.is_some()), |d| {
                            d
                                // A press on a row must not take focus from the field.
                                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                                .when_some(hover, |d, hover| {
                                    d.on_hover(move |on, window, cx| {
                                        if *on {
                                            hover(i, window, cx)
                                        }
                                    })
                                })
                                .when_some(pick, |d, pick| {
                                    d.cursor_pointer()
                                        .on_click(move |_, window, cx| pick(i, window, cx))
                                })
                        })
                        .when_some(row.selector, |d, name| {
                            d.debug_selector(move || name.clone())
                        })
                        .child(measure(report))
                        .children(row.leading)
                        .child(match row.detail {
                            Some(detail) => div()
                                .relative()
                                .flex()
                                .min_w_0()
                                .flex_1()
                                .gap(px(8.))
                                .child(
                                    div()
                                        .flex_none()
                                        .max_w(px(300.))
                                        .truncate()
                                        .child(row.label),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .truncate()
                                        .font_family(crate::typography::MONO_FONT_FAMILY)
                                        .text_size(px(12.))
                                        .line_height(px(20.))
                                        .text_color(theme.muted_foreground)
                                        .child(detail),
                                ),
                            None => div()
                                .relative()
                                .min_w_0()
                                .flex_1()
                                .truncate()
                                .child(row.label),
                        })
                        .when(self.checks, |d| {
                            d.child(
                                div()
                                    .relative()
                                    .size(px(20.))
                                    .flex_none()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .opacity(opacity)
                                    .child(
                                        Icon::new(IconName::Check)
                                            .size(px(16. * scale))
                                            .color(theme.foreground),
                                    ),
                            )
                        })
                        .into_any_element()
                }
            })
            .collect();
        let empty = (count == 0 || rows.is_empty()).then(|| {
            self.empty.map(|words| {
                div()
                    .px(px(12.))
                    .py(px(32.))
                    .text_center()
                    .text_size(px(TEXT))
                    .line_height(px(LINE))
                    .text_color(theme.muted_foreground)
                    .child(words)
            })
        });

        let pill = glide.map(|(top, height, x, width)| {
            div()
                .absolute()
                .left(px(x))
                .top(px(top))
                .w(px(width))
                .h(px(height))
                .rounded(pill_radius)
                .bg(pill_fill)
                .debug_selector(|| "combo-pill".into())
        });
        let content = {
            let state = state.clone();
            div()
                .relative()
                .flex()
                .flex_col()
                .gap(px(ROW_GAP_BETWEEN))
                .when(self.padded, |d| d.p(px(self.style.pad())))
                .child(measure(move |b, cx| {
                    state.update(cx, |s, _| s.content = Some(b))
                }))
                .children(pill)
                .children(rows)
                .children(empty.flatten())
                .children(self.footer)
        };
        let view_state = state.clone();
        div()
            .id(ElementId::NamedChild(
                std::sync::Arc::new(self.id.clone()),
                "scroll".into(),
            ))
            .relative()
            .max_h(px(self.max_height))
            .overflow_y_scroll()
            .track_scroll(&scroll)
            .font_family(FONT_FAMILY)
            .when_some(self.selector, |d, name| {
                d.debug_selector(move || name.into())
            })
            .child(measure(move |b, cx| {
                view_state.update(cx, |s, _| s.view = Some(b))
            }))
            .child(content)
    }
}
