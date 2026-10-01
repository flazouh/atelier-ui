use gpui_kit::{
    AppContext,
    Context,
    Entity,
    EventEmitter,
    FocusHandle,
    Focusable,
    FontWeight,
    InteractiveElement,
    IntoElement,
    KeyDownEvent,
    ParentElement,
    Render,
    SharedString,
    Styled,
    Subscription,
    Window,
    base::input::{IndentInline, MoveDown, MoveRight, MoveUp},
    component::input::{Input, InputEvent, InputState},
    div,
    prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    button::{Button, ButtonSize, ButtonVariant},
    combobox::{ComboEntry, ComboList, ComboRow},
    focus::Field,
    icon::{Icon, IconName},
    theme::ActiveTheme,
    typography::TextSize,
};
use super::types::{FolderError, FolderPickerEvent, ROW};
use super::helpers::{error_words, folder_of, matches, split_path, tab_complete};

/// What the owner sent for a directory.
pub(super) struct Listing {
    pub(super) dir: String,
    pub(super) folders: Result<Vec<SharedString>, FolderError>,
}

pub struct FolderPicker {
    pub(super) input: Entity<InputState>,
    pub(super) listing: Option<Listing>,
    /// The folder in the list that is chosen.
    pub(super) active: usize,
    pub(super) asked: Option<String>,
    /// The text the picker began with.
    pub(super) start: String,
    /// The arrow keys chose the row under `active`: Enter then opens that folder, not the one the field names.
    pub(super) arrowed: bool,
    /// Tab came before the listing did: it completes when the listing arrives.
    pub(super) tab_waits: bool,
    /// The row the pointer is over, which wears the pill until the arrow keys take over.
    pub(super) hover: Option<usize>,
    /// Folders opened before, shown above the list until the reader types.
    pub(super) recent: Vec<SharedString>,
    /// What is going on after Open was pressed, under the field.
    pub(super) working: Option<SharedString>,
    /// Why the folder the reader asked to open could not be opened; gone when they type again.
    pub(super) refused: Option<FolderError>,
    pub(super) _subscription: Subscription,
}

impl EventEmitter<FolderPickerEvent> for FolderPicker {}

impl Focusable for FolderPicker {
    fn focus_handle(&self, cx: &gpui_kit::App) -> FocusHandle {
        self.input.focus_handle(cx)
    }
}

impl FolderPicker {
    /// A picker that starts at `start` (a path, or `~/`).
    pub fn new(start: &str, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| InputState::new(window, cx));
        // The caret goes to the end and the prefill is selected, so typing a whole path replaces it and the keys that
        // complete a name go on from its end.
        input.update(cx, |input, cx| {
            input.set_value(start.to_string(), window, cx);
            input.select_all(window, cx);
        });
        let subscription = cx.subscribe_in(&input, window, |this: &mut Self, _, event: &InputEvent, _, cx| {
            if matches!(event, InputEvent::Change) {
                this.active = 0;
                this.arrowed = false;
                this.hover = None;
                this.refused = None;
                this.ask(cx);
                cx.notify();
            }
        });
        Self { input, listing: None, active: 0, asked: None, arrowed: false, hover: None, tab_waits: false, recent: Vec::new(), working: None, start: start.to_string(), refused: None, _subscription: subscription }
    }

    /// Asks the owner for the folders of the directory in the field, if they are not the ones shown. The owner calls this
    /// once it hears the picker, to get the first listing.
    pub fn ask(&mut self, cx: &mut Context<Self>) {
        let (dir, _) = split_path(&self.text(cx));
        let known = self.listing.as_ref().is_some_and(|l| l.dir == dir);
        if !known && self.asked.as_deref() != Some(dir.as_str()) {
            self.asked = Some(dir.clone());
            cx.emit(FolderPickerEvent::Want(dir.into()));
        }
    }

    /// The owner's answer for `dir`: its folders (files are left out here), or why they could not be read.
    pub fn show(&mut self, dir: &str, entries: Result<Vec<(SharedString, bool)>, FolderError>, window: &mut Window, cx: &mut Context<Self>) {
        if self.asked.as_deref() == Some(dir) {
            self.asked = None;
        }
        let folders = entries.map(|all| all.into_iter().filter(|(_, is_dir)| *is_dir).map(|(name, _)| name).collect());
        self.listing = Some(Listing { dir: dir.to_string(), folders });
        self.active = 0;
        self.arrowed = false;
        self.hover = None;
        cx.notify();
        // The reader may have typed on while the answer came.
        self.ask(cx);
        if std::mem::take(&mut self.tab_waits) {
            self.complete(window, cx);
        }
    }

    /// Folders to offer first, most recent first: they show above the list while the field is as it began.
    pub fn with_recent(mut self, folders: Vec<SharedString>) -> Self {
        self.recent = folders;
        self
    }

    /// Says what is going on after Open (or nothing, when it ends), so a slow open is not a silent one.
    pub fn working(&mut self, words: Option<SharedString>, cx: &mut Context<Self>) {
        self.working = words;
        cx.notify();
    }

    /// The folder the reader asked to open could not be opened: the picker stays, with the path as typed, and says why.
    pub fn refuse(&mut self, error: FolderError, cx: &mut Context<Self>) {
        self.refused = Some(error);
        self.working = None;
        cx.notify();
    }

    pub fn text(&self, cx: &gpui_kit::App) -> String {
        self.input.read(cx).value().to_string()
    }

    pub(super) fn set_text(&mut self, text: String, window: &mut Window, cx: &mut Context<Self>) {
        self.input.update(cx, |input, cx| input.set_value(text, window, cx));
        self.active = 0;
        self.arrowed = false;
        self.hover = None;
        self.refused = None;
        self.ask(cx);
        cx.notify();
    }

    /// The folders that match what is typed, if the listing is the typed directory's.
    pub(super) fn found(&self, cx: &gpui_kit::App) -> Vec<SharedString> {
        let (dir, prefix) = split_path(&self.text(cx));
        match &self.listing {
            Some(Listing { dir: listed, folders: Ok(folders) }) if *listed == dir => matches(folders, &prefix).into_iter().cloned().collect(),
            _ => Vec::new(),
        }
    }

    pub(super) fn descend(&mut self, name: &SharedString, window: &mut Window, cx: &mut Context<Self>) {
        let (dir, _) = split_path(&self.text(cx));
        self.set_text(format!("{dir}{name}/"), window, cx);
    }

    /// Tab: completes the name as far as the folders agree. Before the listing has come, it waits for it.
    pub(super) fn complete(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (dir, _) = split_path(&self.text(cx));
        if !self.listing.as_ref().is_some_and(|l| l.dir == dir) {
            self.tab_waits = true;
            return;
        }
        let found = self.found(cx);
        let refs: Vec<&SharedString> = found.iter().collect();
        let text = self.text(cx);
        let completed = tab_complete(&text, &refs);
        if completed != text {
            self.set_text(completed, window, cx);
        }
    }

    /// Up and Down: choose a row.
    pub(super) fn step(&mut self, down: bool, cx: &mut Context<Self>) {
        let count = self.found(cx).len();
        self.active = if down { (self.active + 1).min(count.saturating_sub(1)) } else { self.active.saturating_sub(1) };
        self.arrowed = true;
        self.hover = None;
        cx.notify();
    }

    /// Right, on a row the arrow keys chose: goes into that folder. False when there is no such row.
    pub(super) fn go_into_row(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let found = self.found(cx);
        if !self.arrowed || found.is_empty() {
            return false;
        }
        let name = found[self.active.min(found.len() - 1)].clone();
        self.descend(&name, window, cx);
        true
    }

    /// Enter: opens the folder the field names, or, when the arrow keys chose a row, that row's folder.
    pub(super) fn open(&mut self, cx: &mut Context<Self>) {
        if self.arrowed
            && let Some(name) = self.found(cx).get(self.active)
        {
            let (dir, _) = split_path(&self.text(cx));
            cx.emit(FolderPickerEvent::Choose(folder_of(&format!("{dir}{name}")).into()));
            return;
        }
        cx.emit(FolderPickerEvent::Choose(folder_of(&self.text(cx)).into()));
    }


    pub(super) fn key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        match event.keystroke.key.as_str() {
            "tab" => self.complete(window, cx),
            "down" => self.step(true, cx),
            "up" => self.step(false, cx),
            "right" => {
                if !self.go_into_row(window, cx) {
                    return;
                }
            }
            "enter" => self.open(cx),
            "escape" => cx.emit(FolderPickerEvent::Cancel),
            _ => return,
        }
        cx.stop_propagation();
        cx.notify();
    }
}

impl Render for FolderPicker {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let this = cx.entity().downgrade();
        let found = self.found(cx);
        let (dir, _) = split_path(&self.text(cx));
        let typed = self.text(cx);
        // What sits under the field: why an open was refused, why the folder cannot be listed, or that it is being read.
        let (status, quiet): (Option<SharedString>, bool) = match (&self.refused, &self.listing) {
            (Some(error), _) => (Some(error_words(&typed, error).into()), error.is_quiet()),
            (None, Some(Listing { dir: listed, folders: Err(error) })) if *listed == dir => (Some(error_words(&dir, error).into()), error.is_quiet()),
            (None, Some(Listing { dir: listed, .. })) if *listed == dir => (None, true),
            _ => (Some("Reading the folder…".into()), true),
        };
        let empty = status.is_none() && found.is_empty();
        let at_start = typed == self.start;
        let recent: Vec<SharedString> = if at_start { self.recent.clone() } else { Vec::new() };
        // The heading and the recent folders come first, then the folders here.
        let offset = if recent.is_empty() { 0 } else { 1 + recent.len() };
        let mut entries: Vec<ComboEntry> = Vec::new();
        if !recent.is_empty() {
            entries.push(ComboEntry::Group("Recent".into()));
            entries.extend(recent.iter().enumerate().map(|(i, path)| {
                ComboEntry::from(
                    ComboRow::new(path.clone()).leading(Icon::new(IconName::Folder).size(px(16.)).color(muted)).debug_name(format!("folder-recent-{i}")),
                )
            }));
        }
        entries.extend(found.iter().enumerate().map(|(i, name)| {
            ComboEntry::from(ComboRow::new(name.clone()).leading(Icon::new(IconName::Folder).size(px(16.)).color(muted)).debug_name(format!("folder-row-{i}")))
        }));
        let pill = if self.arrowed { Some(offset + self.active) } else { self.hover.map(|i| offset + i) };
        let (click, hover) = (this.clone(), this.clone());
        let recent_pick = recent.clone();
        let picked = found.clone();
        let working = self.working.clone();
        let (cancel, open) = (this.clone(), this.clone());
        let focus = self.input.focus_handle(cx);
        div()
            .id("folder-picker")
            .key_context("FolderPicker")
            .flex()
            .flex_col()
            .gap(px(12.))
            .w_full()
            .child(div().text_size(TextSize::Sm.font_size()).font_weight(FontWeight::MEDIUM).child("Open a folder"))
            .child(
                div()
                    .capture_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| this.key(event, window, cx)))
                    // The field binds these keys to its own actions, and an action runs before a key listener: take the
                    // actions first, so Tab completes and the arrows choose a row.
                    .capture_action(cx.listener(|this, _: &IndentInline, window, cx| {
                        this.complete(window, cx);
                        cx.stop_propagation();
                    }))
                    .capture_action(cx.listener(|this, _: &MoveDown, _, cx| {
                        this.step(true, cx);
                        cx.stop_propagation();
                    }))
                    .capture_action(cx.listener(|this, _: &MoveUp, _, cx| {
                        this.step(false, cx);
                        cx.stop_propagation();
                    }))
                    .capture_action(cx.listener(|this, _: &MoveRight, window, cx| {
                        if this.go_into_row(window, cx) {
                            cx.stop_propagation();
                        }
                    }))
                    .child(Field::new(focus, Input::new(&self.input).appearance(false).px(px(10.)).text_size(TextSize::Sm.font_size()))),
            )
            .child(
                div().min_h(px(ROW * 3.)).child(
                    ComboList::new("folder-list", entries).style(crate::combobox::ComboStyle::Folder)
                        .padded(false)
                        .active(pill)
                        .debug_name("folder-list")
                        .on_hover(move |entry, _, cx| {
                            if entry >= offset {
                                hover.update(cx, |p, cx| {
                                    p.hover = Some(entry - offset);
                                    cx.notify();
                                })
                                .ok();
                            }
                        })
                        .on_pick(move |entry, window, cx| {
                            if entry >= offset {
                                if let Some(name) = picked.get(entry - offset) {
                                    let name = name.clone();
                                    click.update(cx, |p, cx| p.descend(&name, window, cx)).ok();
                                }
                            } else if let Some(path) = recent_pick.get(entry - 1) {
                                let path = path.clone();
                                click.update(cx, |_, cx| cx.emit(FolderPickerEvent::Choose(path))).ok();
                            }
                        })
                        .footer(div().children(working.map(|words| div().px(px(10.)).py(px(8.)).text_size(TextSize::Xs.font_size()).text_color(muted).child(words))))
                        .footer(div().children(status.map(|words| {
                            div().px(px(10.)).py(px(8.)).text_size(TextSize::Xs.font_size()).text_color(if quiet { muted } else { theme.warning }).child(words)
                        })))
                        .footer(div().when(empty, |d| d.child(div().px(px(10.)).py(px(8.)).text_size(TextSize::Xs.font_size()).text_color(muted).child("No folder here starts with that.")))),
                ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(div().flex_1().text_size(TextSize::Xs.font_size()).text_color(muted).child("Tab completes the name. Enter opens the folder."))
                    .child(Button::new("folder-cancel").label("Cancel").variant(ButtonVariant::Ghost).cap("Esc").on_click(move |_, _, cx| {
                        cancel.update(cx, |_, cx| cx.emit(FolderPickerEvent::Cancel)).ok();
                    }))
                    .child(Button::new("folder-open").label("Open").size(ButtonSize::Md).variant(ButtonVariant::Primary).cap("↵").on_click(move |_, _, cx| {
                        open.update(cx, |p, cx| p.open(cx)).ok();
                    })),
            )
    }
}
