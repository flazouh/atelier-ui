use gpui_kit::{
    AppContext,
    Context,
    Entity,
    EventEmitter,
    FocusHandle,
    Focusable,
    InteractiveElement,
    IntoElement,
    KeyDownEvent,
    ParentElement,
    Render,
    SharedString,
    Styled,
    Subscription,
    Window,
    component::input::{Input, InputEvent, InputState},
    div,
};

use crate::scale::px;
use crate::{
    combobox::{ComboEntry, ComboList, ComboRow, ComboStyle},
    file_icon::FileIcon,
    fuzzy,
    icon::{Icon, IconName},
    kbd::Kbd,
    keys::Command,
    motion::{Animated, FrameClock},
    theme::{ActiveTheme, radius},
    typography::TextSize,
};
use super::types::{ENTER, Filter, FinderEvent, ROWS, ROW_HEIGHT, WIDTH};
use super::helpers::pick_hover;

/// One row: what it is, where it is, and the file whose icon leads it.
#[derive(Clone, Debug, PartialEq)]
pub struct FinderItem {
    pub label: SharedString,
    /// Muted, after the label: a path and a line, or what the name sits inside.
    pub detail: SharedString,
    pub icon: Option<SharedString>,
}

impl FinderItem {
    pub fn new(label: impl Into<SharedString>, detail: impl Into<SharedString>) -> Self {
        Self { label: label.into(), detail: detail.into(), icon: None }
    }

    /// Leads the row with the file icon for `path`.
    pub fn icon(mut self, path: impl Into<SharedString>) -> Self {
        self.icon = Some(path.into());
        self
    }
}

pub struct Finder {
    pub(super) title: SharedString,
    command: Option<Command>,
    pub(super) filter: Filter,
    input: Entity<InputState>,
    pub(super) items: Vec<FinderItem>,
    /// The items shown, as indices, best first.
    pub(super) shown: Vec<usize>,
    pub(super) selected: usize,
    pub(super) note: SharedString,
    /// The panel's entrance: 0 above and clear, 1 in its place.
    pub(super) enter: Animated,
    clock: FrameClock,
    _input: Subscription,
}

impl EventEmitter<FinderEvent> for Finder {}

impl Focusable for Finder {
    fn focus_handle(&self, cx: &gpui_kit::App) -> FocusHandle {
        self.input.focus_handle(cx)
    }
}

impl Finder {
    pub fn new(title: impl Into<SharedString>, placeholder: impl Into<SharedString>, filter: Filter, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let placeholder: SharedString = placeholder.into();
        let input = cx.new(|cx| InputState::new(window, cx).placeholder(placeholder));
        let _input = cx.subscribe(&input, |this: &mut Self, input, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                let query = input.read(cx).value();
                this.refilter(&query);
                cx.emit(FinderEvent::Query(query));
                cx.notify();
            }
        });
        Self {
            title: title.into(),
            command: None,
            filter,
            input,
            items: Vec::new(),
            shown: Vec::new(),
            selected: 0,
            note: SharedString::default(),
            enter: Animated::new(ENTER, 0.),
            clock: FrameClock::default(),
            _input,
        }
    }

    /// The command whose key opens it; the title wears its cap.
    pub fn command(mut self, command: Command) -> Self {
        self.command = Some(command);
        self
    }

    pub fn set_items(&mut self, items: Vec<FinderItem>, cx: &mut Context<Self>) {
        self.items = items;
        let query = self.query(cx);
        self.refilter(&query);
        cx.notify();
    }

    /// What shows while there are no rows.
    pub fn set_note(&mut self, note: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.note = note.into();
        cx.notify();
    }

    pub fn query(&self, cx: &gpui_kit::App) -> SharedString {
        self.input.read(cx).value()
    }

    /// The items shown, as indices, best first.
    pub fn shown(&self) -> &[usize] {
        &self.shown
    }

    /// The item the arrow keys are on.
    pub fn selected(&self) -> Option<usize> {
        self.shown.get(self.selected).copied()
    }

    fn refilter(&mut self, query: &str) {
        self.shown = match self.filter {
            // No words: the rows in the owner's order.
            Filter::Here if query.trim().is_empty() => (0..self.items.len().min(ROWS)).collect(),
            Filter::Here => {
                let texts: Vec<String> = self.items.iter().map(|i| format!("{} {}", i.label, i.detail)).collect();
                fuzzy::rank(query, texts.iter().map(String::as_str), ROWS)
            }
            Filter::Owner => (0..self.items.len().min(ROWS)).collect(),
        };
        self.selected = 0;
    }

    fn step(&mut self, by: isize, cx: &mut Context<Self>) {
        if self.shown.is_empty() {
            return;
        }
        let last = self.shown.len() as isize - 1;
        self.selected = (self.selected as isize + by).clamp(0, last) as usize;
        cx.notify();
    }

    pub(super) fn pick(&mut self, at: usize, cx: &mut Context<Self>) {
        if let Some(&item) = self.shown.get(at) {
            cx.emit(FinderEvent::Pick(item));
        }
    }

    pub(super) fn key(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        match event.keystroke.key.as_str() {
            "up" => self.step(-1, cx),
            "down" => self.step(1, cx),
            "enter" => self.pick(self.selected, cx),
            "escape" => cx.emit(FinderEvent::Dismiss),
            _ => return,
        }
        cx.stop_propagation();
    }
}

impl Render for Finder {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let reduce = cx.reduce_motion();
        // The entrance: a spring from 8px above and clear.
        self.enter.set_target(1.);
        let dt = self.clock.tick();
        if self.enter.step(dt, reduce) {
            window.request_animation_frame();
        } else {
            self.clock.rest();
        }
        let enter = self.enter.value();
        let count = (!self.items.is_empty()).then(|| format!("{} of {}", self.shown.len(), self.items.len()));
        let search = div()
            .flex()
            .items_center()
            .gap(px(12.))
            .px(px(16.))
            .border_b_1()
            .border_color(crate::text_input::edge(&theme, theme.card))
            .child(Icon::new(IconName::Search).size(px(16.)).color(muted))
            .child(div().flex_1().h(px(48.)).flex().items_center().text_size(TextSize::Sm.font_size()).child(Input::new(&self.input).appearance(false).bordered(false).px(px(0.))))
            .children(count.map(|c| div().text_size(TextSize::Xs.font_size()).text_color(muted).child(c)))
            .child(Kbd::new("Esc"));
        let mut entries = vec![ComboEntry::Group(self.title.clone())];
        entries.extend(self.shown.iter().map(|&item| {
            let row = &self.items[item];
            let entry = ComboRow::new(row.label.clone()).detail(row.detail.clone());
            ComboEntry::from(match row.icon.as_ref() {
                Some(path) => entry.leading(FileIcon::file(path)),
                None => entry,
            })
        }));
        let (pick, hover) = (cx.entity().downgrade(), cx.entity().downgrade());
        let query_hash = {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            self.query(cx).hash(&mut h);
            h.finish()
        };
        let empty = self.shown.is_empty().then(|| {
            let note = if self.note.is_empty() { SharedString::from("Nothing matches") } else { self.note.clone() };
            div().p(px(32.)).text_center().text_size(TextSize::Sm.font_size()).text_color(muted).child(note)
        });
        let list = ComboList::new("finder-list", entries)
            .debug_name("finder-list")
            .style(ComboStyle::Palette)
            .checks(false)
            .max_height(ROW_HEIGHT * 11.)
            .scroll_key(query_hash)
            .active((!self.shown.is_empty()).then_some(self.selected + 1))
            .on_hover(move |i, _, cx| {
                pick_hover(&hover, i, cx);
            })
            .on_pick(move |i, _, cx| {
                pick.update(cx, |this, cx| this.pick(i.saturating_sub(1), cx)).ok();
            })
            .footer(div().children(empty));
        div()
            .id("finder")
            .occlude()
            .key_context("Finder")
            .capture_key_down(cx.listener(Self::key))
            .relative()
            .top(px(-8. * (1. - enter)))
            .opacity(enter)
            .w(px(WIDTH))
            .flex()
            .flex_col()
            .overflow_hidden()
            .rounded(radius::xxl())
            .border_1()
            .border_color(crate::text_input::edge(&theme, theme.card))
            .bg(theme.card)
            .shadow(vec![gpui_kit::BoxShadow {
                color: gpui_kit::Hsla { a: 0.25, ..theme.shadow },
                offset: gpui_kit::point(px(0.), px(25.)),
                blur_radius: px(50.),
                spread_radius: px(-12.),
                inset: false,
            }])
            .child(search)
            .child(list)
    }
}
