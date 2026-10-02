use std::rc::Rc;

use gpui_kit::{
    App,
    ElementId,
    FocusHandle,
    FontWeight,
    InteractiveElement,
    IntoElement,
    ParentElement,
    RenderOnce,
    SharedString,
    StatefulInteractiveElement,
    Styled,
    Window,
    div,
    prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    focus::row_ring,
    icon::{Icon, IconName},
    menu::{Entry, Menu, MenuItem, Origin},
    motion::{Channel, Curve, ease},
    placement::measure,
    popover::{Align, Popover},
    theme::{ActiveTheme, radius},
    typography::FONT_FAMILY,
};
use super::types::{DEFAULT_SHOWN, ENTER_SECONDS, ENTER_Y, HEIGHT, ICON, LINK_PAD, Press};
use super::helpers::{hidden, key_id, shown};

/// One part of the path.
#[derive(Clone)]
pub struct Crumb {
    label: SharedString,
    pub(super) icon: Option<IconName>,
}

impl Crumb {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self { label: label.into(), icon: None }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }
}

struct State {
    seen: bool,
    pub(super) open: bool,
    anchor: Option<gpui_kit::Bounds<gpui_kit::Pixels>>,
    focus: Vec<FocusHandle>,
}

struct Entering {
    channel: Channel,
}

#[derive(IntoElement)]
pub struct Breadcrumb {
    pub(super) id: ElementId,
    pub(super) crumbs: Vec<Crumb>,
    pub(super) max_items: usize,
    pub(super) on_press: Option<Press>,
    selector: Option<&'static str>,
}

impl Breadcrumb {
    pub fn new(id: impl Into<ElementId>, crumbs: impl IntoIterator<Item = Crumb>) -> Self {
        Self { id: id.into(), crumbs: crumbs.into_iter().collect(), max_items: DEFAULT_SHOWN, on_press: None, selector: None }
    }

    pub fn max_items(mut self, max_items: usize) -> Self {
        self.max_items = max_items;
        self
    }

    /// A press on the part at this index. The last part is the page and is not pressed.
    pub fn on_press(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_press = Some(Rc::new(f));
        self
    }

    /// The name a test finds the parts by with `debug_bounds`: `<name>-<index>`; the ellipsis is `<name>-more`.
    pub fn debug_name(mut self, name: &'static str) -> Self {
        self.selector = Some(name);
        self
    }
}

impl RenderOnce for Breadcrumb {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let reduce = cx.reduce_motion();
        let count = self.crumbs.len();
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| State { seen: false, open: false, anchor: None, focus: Vec::new() });
        let (seen, open, anchor) = state.update(cx, |s, cx| {
            while s.focus.len() < count + 1 {
                s.focus.push(cx.focus_handle());
            }
            (s.seen, s.open, s.anchor)
        });
        // Parts that arrive after the first frame come in; the first frame's parts stand.
        state.update(cx, |s, _| s.seen = true);
        let keyboard = window.last_input_was_keyboard();
        let crumbs = self.crumbs.clone();
        let layout = shown(count, self.max_items);
        let hidden_range = hidden(count, self.max_items);
        let press = self.on_press.clone();
        let selector = self.selector;
        let mut moving = false;
        let mut items: Vec<gpui_kit::AnyElement> = Vec::new();
        for (slot, entry) in layout.iter().enumerate() {
            let separator = (slot > 0).then(|| {
                div().flex_none().flex().items_center().child(
                    Icon::new(IconName::ChevronRight)
                        .size(px(12.))
                        .color(theme.faint()),
                )
            });
            let body: gpui_kit::AnyElement = match entry {
                Some(index) => {
                    let index = *index;
                    let crumb = &crumbs[index];
                    let last = index == count - 1;
                    let key = ElementId::NamedChild(std::sync::Arc::new(self.id.clone()), format!("crumb-{index}-{}", crumb.label).into());
                    let enter = window.use_keyed_state(key.clone(), cx, move |_, _| Entering { channel: Channel::new(if seen && !reduce { 0. } else { 1. }) });
                    let v = enter.update(cx, |e, _| {
                        if e.channel.target() < 1. {
                            e.channel.animate(1., Curve::Ease(ENTER_SECONDS, ease::OUT), 0., reduce);
                        }
                        e.channel.value()
                    });
                    moving |= v < 1.;
                    let handle = state.read(cx).focus[index].clone();
                    let focused = handle.is_focused(window) && keyboard;
                    let pressed = press.clone().filter(|_| !last);
                    let words = div()
                        .id(key_id(&self.id, index))
                        .relative()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .min_h(px(HEIGHT))
                        .min_w_0()
                        .px(px(LINK_PAD))
                        .rounded(radius::md())
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(if last { theme.foreground } else { theme.muted_foreground })
                        .when_some(pressed.clone(), |d, _| {
                            d.cursor_pointer().hover(|s| s.bg(theme.card_strong.opacity(0.6)).text_color(theme.foreground)).track_focus(&handle.tab_stop(true))
                        })
                        .when(selector.is_some(), |d| {
                            let name = format!("{}-{index}", selector.unwrap_or_default());
                            d.debug_selector(move || name.clone())
                        })
                        .children(crumb.icon.map(|icon| Icon::new(icon).size(px(ICON))))
                        .child(crumb.label.clone())
                        .when(focused, |d| d.child(row_ring(&theme, theme.background, radius::md())))
                        .when_some(pressed, |d, f| {
                            let key = f.clone();
                            d.on_click(move |_, window, cx| f(index, window, cx)).on_key_down(move |event, window, cx| {
                                if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                                    cx.stop_propagation();
                                    key(index, window, cx);
                                }
                            })
                        });
                    div().flex().items_center().gap(px(4.)).min_w_0().relative().top(px(ENTER_Y * (1. - v))).opacity(v).children(separator).child(words).into_any_element()
                }
                None => {
                    let toggle = state.clone();
                    let close = state.clone();
                    let picks = press.clone();
                    let entries: Vec<Entry> = hidden_range
                        .clone()
                        .map(|i| {
                            let picks = picks.clone();
                            Entry::from(MenuItem::new(crumbs[i].label.clone()).on_select(move |window, cx| {
                                if let Some(f) = &picks {
                                    f(i, window, cx)
                                }
                            }))
                        })
                        .collect();
                    let rows = entries.len();
                    let more_name = format!("{}-more", selector.unwrap_or_default());
                    let button = div()
                        .id(key_id(&self.id, usize::MAX))
                        .relative()
                        .size(px(HEIGHT))
                        .flex()
                        .flex_none()
                        .items_center()
                        .justify_center()
                        .rounded(radius::md())
                        .cursor_pointer()
                        .text_color(theme.muted_foreground)
                        .hover(|s| s.bg(theme.card_strong.opacity(0.6)).text_color(theme.foreground))
                        .when(selector.is_some(), |d| d.debug_selector(move || more_name.clone()))
                        .child(measure({
                            let state = state.clone();
                            move |b, cx| state.update(cx, |s, _| s.anchor = Some(b))
                        }))
                        .child(Icon::new(IconName::MoreHoriz).size(px(16.)))
                        .on_click(move |_, _, cx| {
                            toggle.update(cx, |s, cx| {
                                s.open = !s.open;
                                cx.notify();
                            })
                        });
                    let menu = Menu::new((self.id.clone(), "hidden"), entries).origin(Origin::TopLeft).on_dismiss({
                        let close = close.clone();
                        move |_, cx| {
                            close.update(cx, |s, cx| {
                                s.open = false;
                                cx.notify();
                            })
                        }
                    });
                    div()
                        .flex()
                        .items_center()
                        .gap(px(4.))
                        .flex_none()
                        .children(separator)
                        .child(button)
                        .child(
                            Popover::new((self.id.clone(), "overflow"))
                                .open(open)
                                .anchor(anchor)
                                .align(Align::Start)
                                .gap(6.)
                                .height(crate::menu::height(rows))
                                .on_close(move |_, cx| {
                                    close.update(cx, |s, cx| {
                                        s.open = false;
                                        cx.notify();
                                    })
                                })
                                .child(menu),
                        )
                        .into_any_element()
                }
            };
            items.push(body);
        }
        if moving {
            window.request_animation_frame();
        }
        div().id(self.id.clone()).flex().flex_wrap().items_center().gap_x(px(4.)).gap_y(px(4.)).min_w_0().text_size(px(14.)).line_height(px(20.)).font_family(FONT_FAMILY).children(items)
    }
}
