use std::rc::Rc;

use gpui_kit::{
    Anchor, App, Bounds, ElementId, FocusHandle, FontWeight, InteractiveElement, IntoElement,
    ParentElement, Pixels, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window,
    anchored, deferred, div, point, prelude::FluentBuilder,
};

use super::helpers::{
    collapsed, fill_opacity, jump, lead_slot, panel_shadow, panel_size, pill_fill, unfolded, walk,
};
use super::types::{
    Choice, Choose, Ends, Entry, LINE, Lead, Origin, SLOT, Select, TEXT, TYPED_FOR, Tone, UNFOLD,
};
use crate::scale::px;
use crate::{
    icon::{Icon, IconName},
    kbd::Kbd,
    motion::{Animated, Channel, Curve, FrameClock, Spring, ease, now},
    placement::measure,
    switch::Switch,
    theme::ActiveTheme,
    typography::FONT_FAMILY,
};

/// How a site's menu looks: the numbers its own menu had before the Menu part. Motion is the same for all.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenuLook {
    pub min_width: f32,
    pub pad: f32,
    pub row_x: f32,
    pub row_y: f32,
    pub gap: f32,
    pub row_radius: f32,
    pub panel_radius: f32,
    /// The space that parts two groups of rows (there is no line).
    pub group: f32,
    /// The popover shadow, times this.
    pub shadow: f32,
}

impl MenuLook {
    /// The review bar's ⋯ menu, and the default.
    pub const BAR: MenuLook = MenuLook {
        min_width: 180.,
        pad: 4.,
        row_x: 10.,
        row_y: 6.,
        gap: 12.,
        row_radius: 8.,
        panel_radius: 12.,
        group: 8.,
        shadow: 1.,
    };
    /// The merge button's menu.
    pub const MERGE: MenuLook = MenuLook {
        min_width: 240.,
        pad: 4.,
        row_x: 10.,
        row_y: 5.,
        gap: 8.,
        row_radius: 8.,
        panel_radius: 12.,
        group: 6.,
        shadow: 1.,
    };
    /// The prompt's add menu.
    pub const PROMPT: MenuLook = MenuLook {
        min_width: 224.,
        pad: 6.,
        row_x: 10.,
        row_y: 8.,
        gap: 10.,
        row_radius: 8.,
        panel_radius: 12.,
        group: 8.,
        shadow: 1.4,
    };
    /// The prompt's pickers, as the model select's list looks: 28px rows on a 208px panel.
    pub const SELECT: MenuLook = MenuLook {
        min_width: 208.,
        pad: 4.,
        row_x: 10.,
        row_y: 4.,
        gap: 8.,
        row_radius: 6.,
        panel_radius: 12.,
        group: 8.,
        shadow: 1.,
    };
    /// A project's ⋯ menu.
    pub const PROJECT: MenuLook = MenuLook {
        min_width: 180.,
        pad: 4.,
        row_x: 8.,
        row_y: 4.,
        gap: 8.,
        row_radius: 6.,
        panel_radius: 8.,
        group: 8.,
        shadow: 1.,
    };
}

pub struct MenuItem {
    label: SharedString,
    description: Option<SharedString>,
    pub(super) icon: Option<IconName>,
    ends: Option<Box<Ends>>,
    pub(super) shortcut: Option<SharedString>,
    cap: Option<SharedString>,
    pub(super) choice: Option<Choice>,
    pub(super) tone: Tone,
    pub(super) disabled: bool,
    close: bool,
    selector: Option<String>,
    pub(super) on_select: Option<Select>,
    /// The rows of the menu this row opens beside itself.
    pub(super) submenu: Option<Vec<Entry>>,
    lead: Option<Lead>,
}

impl MenuItem {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            description: None,
            icon: None,
            ends: None,
            shortcut: None,
            cap: None,
            choice: None,
            tone: Tone::Default,
            disabled: false,
            close: true,
            selector: None,
            on_select: None,
            submenu: None,
            lead: None,
        }
    }

    /// A mark, or a monogram where there is none, before the words.
    pub fn lead(mut self, lead: Lead) -> Self {
        self.lead = Some(lead);
        self
    }

    /// Makes the row open `entries` as a menu beside it, on hover, on a press or on Right. Left or Escape closes that
    /// menu and the row has focus again. The row itself chooses nothing, so it has no `on_select`.
    pub fn submenu(mut self, entries: impl IntoIterator<Item = Entry>) -> Self {
        self.submenu = Some(entries.into_iter().collect());
        self
    }

    /// A line under the words, in the muted tone.
    pub fn description(mut self, words: impl Into<SharedString>) -> Self {
        self.description = Some(words.into());
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// A small element in the icon's place, drawn each time the menu draws: a project's badge.
    pub fn lead_element(mut self, lead: impl Fn(&App) -> gpui_kit::AnyElement + 'static) -> Self {
        self.ends.get_or_insert_default().lead = Some(Rc::new(lead));
        self
    }

    /// A small element at the end of the row, drawn each time the menu draws: a project's host or how many
    /// of its sessions wait, on the row's one line.
    pub fn trailing(mut self, trailing: impl Fn(&App) -> gpui_kit::AnyElement + 'static) -> Self {
        self.ends.get_or_insert_default().trailing = Some(Rc::new(trailing));
        self
    }

    /// The keys of the row's command as small text at the end (beui.dev ContextMenuShortcut).
    pub fn shortcut(mut self, keys: impl Into<SharedString>) -> Self {
        self.shortcut = Some(crate::keys::cap(&keys.into()));
        self
    }

    /// The keys of the row's command as atelier's key cap at the end.
    pub fn cap(mut self, keys: impl Into<SharedString>) -> Self {
        self.cap = Some(keys.into());
        self
    }

    pub fn choice(mut self, choice: Choice) -> Self {
        self.choice = Some(choice);
        self
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Whether choosing the row closes the menu (the default).
    pub fn close_on_select(mut self, close: bool) -> Self {
        self.close = close;
        self
    }

    /// The name a test finds the row by with `debug_bounds`.
    pub fn debug_name(mut self, name: impl Into<String>) -> Self {
        self.selector = Some(name.into());
        self
    }

    pub fn on_select(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
}

/// How far the clip is pulled in from each side of the panel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Inset {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

struct State {
    handles: Vec<FocusHandle>,
    rects: Vec<Option<Bounds<Pixels>>>,
    list: Option<Bounds<Pixels>>,
    pub(super) size: Option<(f32, f32)>,
    seeded: bool,
    pub(super) top: Animated,
    pub(super) height: Animated,
    pub(super) reveal: Channel,
    started: bool,
    took_focus: bool,
    pub(super) typed: String,
    typed_at: Option<std::time::Instant>,
    pub(super) clock: FrameClock,
    /// The row whose submenu is open.
    pub(super) sub: Option<usize>,
}

impl State {
    pub(super) fn new() -> Self {
        Self {
            handles: Vec::new(),
            rects: Vec::new(),
            list: None,
            size: None,
            seeded: false,
            top: Animated::new(Spring::LAYOUT, 0.),
            height: Animated::new(Spring::LAYOUT, 0.),
            reveal: Channel::new(0.),
            started: false,
            took_focus: false,
            typed: String::new(),
            typed_at: None,
            clock: FrameClock::default(),
            sub: None,
        }
    }
}

#[derive(IntoElement)]
pub struct Menu {
    id: ElementId,
    entries: Vec<Entry>,
    pub(super) width: Option<f32>,
    pub(super) look: MenuLook,
    pub(super) origin: Option<Origin>,
    pub(super) on_dismiss: Option<Select>,
    /// Set on a submenu: what Left and Escape do.
    on_back: Option<Select>,
    selector: Option<&'static str>,
}

impl Menu {
    pub fn new(id: impl Into<ElementId>, entries: impl IntoIterator<Item = Entry>) -> Self {
        Self {
            id: id.into(),
            entries: entries.into_iter().collect(),
            width: None,
            look: MenuLook::BAR,
            origin: None,
            on_dismiss: None,
            on_back: None,
            selector: None,
        }
    }

    /// The least width of the panel.
    pub fn min_width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
    /// The look of the site's menu.
    pub fn look(mut self, look: MenuLook) -> Self {
        self.look = look;
        self
    }

    /// Unfolds the panel from `origin` when it opens. Leave it off when the keys opened the menu.
    pub fn origin(mut self, origin: Origin) -> Self {
        self.origin = Some(origin);
        self
    }

    /// Runs when a row that closes the menu is chosen, before the row's own action.
    pub fn on_dismiss(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(f));
        self
    }

    /// Makes this menu a submenu: Left and Escape run `f` instead of closing everything.
    pub(super) fn on_back(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_back = Some(Rc::new(f));
        self
    }

    /// The name a test finds the panel by with `debug_bounds`.
    pub fn debug_name(mut self, name: &'static str) -> Self {
        self.selector = Some(name);
        self
    }
}

impl RenderOnce for Menu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let look = self.look;
        let theme = cx.theme().clone();
        let reduce = cx.reduce_motion();
        let count = self.entries.len();
        let menu_id = self.id.clone();
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| State::new());
        let handles = state.update(cx, |s, cx| {
            while s.handles.len() < count {
                s.handles.push(cx.focus_handle());
            }
            s.rects.resize(count, None);
            s.handles[..count].to_vec()
        });
        let reachable: Vec<usize> = self
            .entries
            .iter()
            .enumerate()
            .filter_map(|(i, e)| matches!(e, Entry::Item(item) if !item.disabled).then_some(i))
            .collect();
        // The first row has focus when the menu opens.
        if !state.read(cx).took_focus {
            state.update(cx, |s, _| s.took_focus = true);
            // Deferred: a popover round the menu focuses its own panel in a deferral of its own, and this one runs after it.
            if let Some(first) = reachable.first().map(|i| handles[*i].clone()) {
                window.defer(cx, move |window, cx| window.focus(&first, cx));
            }
        }
        // A row whose submenu is open keeps its pill while focus is in the submenu.
        let active = reachable
            .iter()
            .copied()
            .find(|i| handles[*i].is_focused(window))
            .or(state.read(cx).sub);
        let active_tone = active.and_then(|i| match &self.entries[i] {
            Entry::Item(item) => Some(item.tone),
            _ => None,
        });
        let morph = self.origin.filter(|_| !reduce);
        let (list_at, moving, reveal) = state.update(cx, |s, _| {
            let mut moving = false;
            let target = active.and_then(|i| s.rects[i]).zip(s.list).map(|(r, l)| {
                (
                    crate::scale::design(r.origin.y - l.origin.y),
                    crate::scale::design(r.size.height),
                    crate::scale::design(r.origin.x - l.origin.x),
                    crate::scale::design(r.size.width),
                )
            });
            match target {
                Some((y, h, ..)) => {
                    // While the panel unfolds its rows still settle (a description wraps once the width is
                    // known), so the pill follows them at once and glides only after the unfold.
                    let unfolding = morph.is_some() && (!s.started || s.reveal.is_running());
                    if !s.seeded || unfolding {
                        s.top = Animated::new(Spring::LAYOUT, y);
                        s.height = Animated::new(Spring::LAYOUT, h);
                        s.seeded = true;
                    } else {
                        s.top.set_target(y);
                        s.height.set_target(h);
                    }
                }
                None => {
                    if active.is_none() {
                        s.seeded = false;
                    } else {
                        moving = true;
                    }
                }
            }
            let dt = s.clock.tick();
            moving |= s.top.step(dt, reduce) | s.height.step(dt, reduce);
            if !moving {
                s.clock.rest();
            }
            let reveal = match morph {
                None => 1.,
                Some(_) => {
                    if !s.started {
                        if s.size.is_some() {
                            s.reveal
                                .animate(1., Curve::Ease(UNFOLD, ease::OUT), 0., false);
                            s.started = true;
                        }
                        moving = true;
                    }
                    moving |= s.reveal.is_running();
                    if s.started { s.reveal.value() } else { 0. }
                }
            };
            (
                target.map(|(_, _, x, w)| (s.top.value(), s.height.value(), x, w)),
                moving,
                reveal,
            )
        });
        if moving {
            window.request_animation_frame();
        }

        // The choose action of each row.
        let dismiss = self.on_dismiss.clone();
        let sub_dismiss = dismiss.clone();
        let picks: Vec<Option<(bool, Option<Select>)>> = self
            .entries
            .iter()
            .map(|e| match e {
                Entry::Item(item) if !item.disabled => Some((item.close, item.on_select.clone())),
                _ => None,
            })
            .collect();
        let opens: Rc<Vec<bool>> = Rc::new(
            self.entries
                .iter()
                .map(|e| matches!(e, Entry::Item(item) if item.submenu.is_some()))
                .collect(),
        );
        let open_sub = {
            let state = state.clone();
            move |at: Option<usize>, cx: &mut App| {
                state.update(cx, |s, cx| {
                    if s.sub != at {
                        s.sub = at;
                        cx.notify();
                    }
                })
            }
        };
        let choose: Choose = {
            let (picks, opens, open_sub) = (Rc::new(picks), opens.clone(), open_sub.clone());
            Rc::new(move |i, window, cx| {
                if opens.get(i) == Some(&true) {
                    return open_sub(Some(i), cx);
                }
                let Some(Some((close, f))) = picks.get(i) else {
                    return;
                };
                if *close && let Some(d) = &dismiss {
                    d(window, cx);
                }
                if let Some(f) = f {
                    f(window, cx);
                }
            })
        };
        let labels: Rc<Vec<(usize, String)>> = Rc::new(
            self.entries
                .iter()
                .enumerate()
                .filter_map(|(i, e)| match e {
                    Entry::Item(item) if !item.disabled => Some((i, item.label.to_string())),
                    _ => None,
                })
                .collect(),
        );
        let keyboard = window.last_input_was_keyboard();

        let rows: Vec<_> = self
            .entries
            .into_iter()
            .enumerate()
            .map(|(i, entry)| match entry {
                Entry::Label(words) => div()
                    .px(px(look.row_x))
                    .pt(px(6.))
                    .pb(px(4.))
                    .text_size(px(10.))
                    .line_height(px(16.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.muted_foreground)
                    .child(words.to_uppercase())
                    .into_any_element(),
                // No line: a gap parts the groups.
                Entry::Separator => div().h(px(look.group)).into_any_element(),
                Entry::Item(mut item) => {
                    let handle = handles[i].clone();
                    let sub_entries = item.submenu.take();
                    let has_sub = sub_entries.is_some();
                    let ink = match item.tone {
                        Tone::Default => theme.foreground,
                        Tone::Destructive => theme.danger,
                    };
                    let (pick, hover) = (choose.clone(), handle.clone());
                    let hover_sub = open_sub.clone();
                    let nested = sub_entries
                        .filter(|_| state.read(cx).sub == Some(i))
                        .zip(state.read(cx).rects[i])
                        .map(|(entries, row)| {
                            let (back_state, back_to) = (state.clone(), handle.clone());
                            let mut menu = Menu::new(
                                ElementId::NamedChild(
                                    std::sync::Arc::new(menu_id.clone()),
                                    format!("sub-{i}").into(),
                                ),
                                entries,
                            )
                            .look(look)
                            .on_back(move |window, cx| {
                                back_state.update(cx, |s, cx| {
                                    s.sub = None;
                                    cx.notify();
                                });
                                window.focus(&back_to, cx);
                            });
                            if let Some(d) = sub_dismiss.clone() {
                                menu = menu.on_dismiss(move |window, cx| d(window, cx));
                            }
                            let at = point(row.right() + px(2.), row.top() - px(look.pad));
                            deferred(
                                anchored()
                                    .position(at)
                                    .anchor(Anchor::TopLeft)
                                    .snap_to_window_with_margin(px(8.))
                                    .child(div().occlude().child(menu)),
                            )
                            .with_priority(crate::popover::PRIORITY + 2)
                        });
                    let report = {
                        let state = state.clone();
                        move |b: Bounds<Pixels>, cx: &mut App| {
                            state.update(cx, |s, _| s.rects[i] = Some(b))
                        }
                    };
                    let focused = handle.is_focused(window) && keyboard;
                    let chevron = has_sub.then(|| {
                        div()
                            .flex_none()
                            .ml_auto()
                            .h(px(LINE))
                            .flex()
                            .items_center()
                            .child(
                                Icon::new(IconName::ChevronRight)
                                    .size(px(16.))
                                    .color(theme.muted_foreground),
                            )
                            .into_any_element()
                    });
                    let tail = chevron.or_else(|| {
                        item.choice.and_then(|c| match c {
                            Choice::Selected(on) => Some(
                                div()
                                    .size(px(20.))
                                    .flex_none()
                                    .ml_auto()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .when(on, |d| {
                                        d.child(Icon::new(IconName::Check).size(px(16.)).color(ink))
                                    })
                                    .into_any_element(),
                            ),
                            Choice::Switch(on) => Some(
                                div()
                                    .flex_none()
                                    .ml_auto()
                                    .flex()
                                    .items_center()
                                    .h(px(LINE))
                                    .child(
                                        Switch::new(
                                            ElementId::NamedInteger("menu-switch".into(), i as u64),
                                            on,
                                        )
                                        .compact(true),
                                    )
                                    .into_any_element(),
                            ),
                            _ => None,
                        })
                    });
                    let mark = item.choice.and_then(|c| match c {
                        Choice::Selected(_) | Choice::Switch(_) => None,
                        Choice::Check(on) | Choice::Radio(on) => Some(
                            div()
                                .size(px(SLOT))
                                .flex_none()
                                .flex()
                                .items_center()
                                .justify_center()
                                .when(on, |d| {
                                    d.child(Icon::new(IconName::Check).size(px(14.)).color(ink))
                                }),
                        ),
                    });
                    div()
                        .id(ElementId::NamedInteger("menu-row".into(), i as u64))
                        .relative()
                        .flex()
                        .items_start()
                        .gap(px(look.gap))
                        .w_full()
                        .px(px(look.row_x))
                        .py(px(look.row_y))
                        .rounded(px(look.row_radius))
                        .text_size(px(TEXT))
                        .line_height(px(LINE))
                        .text_color(ink)
                        .when(item.disabled, |d| d.opacity(0.4))
                        .when(!item.disabled, |d| {
                            d.cursor_pointer()
                                .track_focus(&handle)
                                .on_hover(move |on, window, cx| {
                                    if *on {
                                        window.focus(&hover, cx);
                                        hover_sub(has_sub.then_some(i), cx);
                                    }
                                })
                                .on_click(move |_, window, cx| pick(i, window, cx))
                        })
                        .when_some(item.selector, |d, name| {
                            d.debug_selector(move || name.clone())
                        })
                        .child(measure(report))
                        .children(mark)
                        .when_some(item.lead, |d, lead| {
                            d.child(lead_slot(&item.label, lead, &theme))
                        })
                        .when_some(
                            item.ends
                                .as_ref()
                                .and_then(|e| e.lead.as_ref())
                                .map(|lead| lead(cx)),
                            |d, lead| {
                                d.child(
                                    div()
                                        .flex_none()
                                        .mt(px(1.))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(lead),
                                )
                            },
                        )
                        .when_some(item.icon, |d, icon| {
                            let name = format!("menu-icon-{}", item.label);
                            d.child(
                                div()
                                    .debug_selector(move || name.clone())
                                    .flex_none()
                                    .mt(px(2.))
                                    .size(px(SLOT))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(Icon::new(icon).size(px(SLOT)).color(ink)),
                            )
                        })
                        .child(div().flex_1().min_w_0().child(item.label).when_some(
                            item.description,
                            |d, words| {
                                d.child(
                                    div()
                                        .mt(px(2.))
                                        .text_size(px(12.))
                                        .line_height(px(16.))
                                        .text_color(theme.muted_foreground)
                                        .child(words),
                                )
                            },
                        ))
                        .children(tail)
                        .children(item.ends.and_then(|e| e.trailing).map(|trailing| {
                            div()
                                .flex_none()
                                .ml_auto()
                                .pl(px(16.))
                                .flex()
                                .items_center()
                                .h(px(LINE))
                                .child(trailing(cx))
                        }))
                        .children(item.shortcut.map(|keys| {
                            div()
                                .flex_none()
                                .ml_auto()
                                .pl(px(16.))
                                .text_size(px(10.))
                                .line_height(px(16.))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.muted_foreground)
                                .child(keys)
                        }))
                        .children(item.cap.map(|keys| {
                            div()
                                .flex_none()
                                .ml_auto()
                                .pl(px(16.))
                                .child(Kbd::new(keys))
                        }))
                        .when(focused, |d| {
                            d.child(crate::focus::row_ring(
                                &theme,
                                theme.popover,
                                px(look.row_radius),
                            ))
                        })
                        .children(nested)
                        .into_any_element()
                }
            })
            .collect();

        let pill = list_at
            .zip(active_tone)
            .map(|((top, height, x, width), tone)| {
                div()
                    .absolute()
                    .left(px(x))
                    .top(px(top))
                    .w(px(width))
                    .h(px(height))
                    .rounded(px(look.row_radius))
                    .bg(pill_fill(&theme, tone))
            });
        let list = {
            let state = state.clone();
            div()
                .relative()
                .flex()
                .flex_col()
                .child(measure(move |b, cx| {
                    state.update(cx, |s, _| s.list = Some(b))
                }))
                .children(pill)
                .children(rows)
        };

        let (key_state, key_handles, key_choose, key_reach) = (
            state.clone(),
            handles.clone(),
            choose.clone(),
            reachable.clone(),
        );
        let (key_opens, back) = (opens.clone(), self.on_back.clone());
        let on_keys = move |event: &gpui_kit::KeyDownEvent, window: &mut Window, cx: &mut App| {
            let key = event.keystroke.key.as_str();
            let now_focused = key_reach
                .iter()
                .copied()
                .find(|i| key_handles[*i].is_focused(window));
            let go = |to: Option<usize>, window: &mut Window, cx: &mut App| {
                if let Some(to) = to {
                    window.focus(&key_handles[to], cx);
                }
            };
            match key {
                "left" | "escape" if back.is_some() => {
                    if let Some(back) = &back {
                        back(window, cx);
                    }
                }
                "right" => match now_focused {
                    Some(i) if key_opens.get(i) == Some(&true) => key_choose(i, window, cx),
                    _ => return,
                },
                "down" => go(walk(&key_reach, now_focused, 1), window, cx),
                "up" => go(walk(&key_reach, now_focused, -1), window, cx),
                "home" => go(key_reach.first().copied(), window, cx),
                "end" => go(key_reach.last().copied(), window, cx),
                "enter" | "space" => match now_focused {
                    Some(i) => key_choose(i, window, cx),
                    None => return,
                },
                _ => {
                    let mods = event.keystroke.modifiers;
                    if key.chars().count() != 1 || mods.control || mods.platform || mods.alt {
                        return;
                    }
                    let at = now();
                    let typed = key_state.update(cx, |s, _| {
                        if s.typed_at
                            .is_none_or(|t| at.saturating_duration_since(t) > TYPED_FOR)
                        {
                            s.typed.clear();
                        }
                        s.typed.push_str(key);
                        s.typed_at = Some(at);
                        s.typed.clone()
                    });
                    go(jump(&labels, &typed), window, cx);
                }
            }
            cx.stop_propagation();
        };

        let width = self.width.unwrap_or(look.min_width);
        let selector = self.selector;
        let elevation = crate::design_preview::elevation(); // design preview: remove after Alex picks
        let full = move |shadow: Vec<gpui_kit::BoxShadow>,
                         size: Option<gpui_kit::Size<Pixels>>,
                         size_state: gpui_kit::Entity<State>| {
            div()
                .relative()
                .flex()
                .flex_col()
                .min_w(px(width))
                .p(px(look.pad))
                .rounded(px(look.panel_radius))
                .border_1()
                .border_color(crate::design_preview::panel_edge(&theme, elevation)) // design preview: remove after Alex picks
                .bg(crate::design_preview::panel_fill(
                    &theme,
                    elevation,
                    theme.popover,
                ))
                .shadow(crate::design_preview::panel_shadows(
                    &theme, elevation, shadow,
                ))
                .text_color(theme.foreground)
                .font_family(FONT_FAMILY)
                .when_some(size, |d, size| d.w(size.width))
                .when_some(selector, |d, name| d.debug_selector(move || name.into()))
                .on_key_down(on_keys)
                .child(measure(move |b, cx| {
                    // The probe sits inside the panel's border, so it reads the panel less its border; the size kept is the
                    // whole panel, or each frame the panel would be forced a border narrower than the last.
                    size_state.update(cx, |s, _| {
                        s.size = Some(panel_size(b.size.width, b.size.height))
                    })
                }))
                .child(list)
        };
        let size = state.read(cx).size;
        match (morph, size) {
            (Some(origin), Some(size)) if reveal < 1. => {
                let start = collapsed(origin.point(size), size);
                let (inset, corner) = unfolded(start, reveal);
                let (w, h) = size;
                let panel = full(
                    Vec::new(),
                    Some(gpui_kit::Size {
                        width: px(w),
                        height: px(h),
                    }),
                    state.clone(),
                );
                div()
                    .relative()
                    .w(px(w))
                    .h(px(h))
                    .opacity(fill_opacity(reveal))
                    .child(
                        div()
                            .absolute()
                            .left(px(inset.left))
                            .top(px(inset.top))
                            .w(px(w - inset.left - inset.right))
                            .h(px(h - inset.top - inset.bottom))
                            .overflow_hidden()
                            .rounded(px(corner))
                            .shadow(crate::design_preview::panel_shadows(
                                &cx.theme().clone(),
                                elevation,
                                panel_shadow(&cx.theme().clone(), look.shadow),
                            ))
                            .child(
                                div()
                                    .absolute()
                                    .left(px(-inset.left))
                                    .top(px(-inset.top))
                                    .w(px(w))
                                    .h(px(h))
                                    .child(panel),
                            ),
                    )
                    .into_any_element()
            }
            _ => {
                let theme = cx.theme().clone();
                div()
                    .opacity(reveal)
                    .child(full(panel_shadow(&theme, look.shadow), None, state.clone()))
                    .into_any_element()
            }
        }
    }
}
