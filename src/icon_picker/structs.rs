use std::path::PathBuf;

use gpui_kit::StyledImage as _;
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
    base::input::{MoveDown, MoveUp},
    component::input::{Input, InputEvent, InputState},
    div,
    prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    button::{Button, ButtonVariant},
    combobox::{ComboEntry, ComboList, ComboRow},
    focus::Field,
    icon::{Icon, IconName},
    icon_candidates,
    theme::ActiveTheme,
    typography::TextSize,
};
use super::types::IconPickerEvent;

pub struct IconPicker {
    pub(super) input: Entity<InputState>,
    /// Every candidate, most likely first.
    pub(super) all: Vec<String>,
    /// Where a local project's files are, for the thumbnails.
    pub(super) root: Option<PathBuf>,
    pub(super) active: usize,
    pub(super) _subscription: Subscription,
}

impl EventEmitter<IconPickerEvent> for IconPicker {}

impl Focusable for IconPicker {
    fn focus_handle(&self, cx: &gpui_kit::App) -> FocusHandle {
        self.input.focus_handle(cx)
    }
}

impl IconPicker {
    /// A chooser over the image files among `paths`, most likely first. `root` is the project folder when the files
    /// are on this machine.
    pub fn new(paths: Vec<String>, root: Option<PathBuf>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| InputState::new(window, cx).placeholder("Filter the images"));
        let subscription = cx.subscribe_in(&input, window, |this: &mut Self, _, event: &InputEvent, _, cx| {
            if matches!(event, InputEvent::Change) {
                this.active = 0;
                cx.notify();
            }
        });
        let refs: Vec<&str> = paths.iter().map(String::as_str).collect();
        Self { input, all: icon_candidates::rank(&refs), root, active: 0, _subscription: subscription }
    }
    pub(super) fn found(&self, cx: &gpui_kit::App) -> Vec<String> {
        icon_candidates::filter(&self.all, self.input.read(cx).value().as_ref())
    }
    pub(super) fn step(&mut self, down: bool, cx: &mut Context<Self>) {
        let count = self.found(cx).len();
        self.active = if down { (self.active + 1).min(count.saturating_sub(1)) } else { self.active.saturating_sub(1) };
        cx.notify();
    }
    pub(super) fn choose(&mut self, cx: &mut Context<Self>) {
        if let Some(path) = self.found(cx).get(self.active) {
            cx.emit(IconPickerEvent::Choose(path.clone().into()));
        }
    }
    pub(super) fn key(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        match event.keystroke.key.as_str() {
            "down" => self.step(true, cx),
            "up" => self.step(false, cx),
            "enter" => self.choose(cx),
            "escape" => cx.emit(IconPickerEvent::Cancel),
            _ => return,
        }
        cx.stop_propagation();
        cx.notify();
    }
}

impl Render for IconPicker {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let this = cx.entity().downgrade();
        let found = self.found(cx);
        let entries: Vec<ComboEntry> = found
            .iter()
            .enumerate()
            .map(|(i, path)| {
                let (dir, name) = path.rsplit_once('/').map_or(("", path.as_str()), |(d, n)| (d, n));
                let thumb = match &self.root {
                    Some(root) => div()
                        .debug_selector(move || format!("icon-thumb-{i}"))
                        .size(px(20.))
                        .child(gpui_kit::img(root.join(path)).size(px(20.)).object_fit(gpui_kit::ObjectFit::Contain))
                        .into_any_element(),
                    None => Icon::new(IconName::Image).size(px(16.)).color(muted).into_any_element(),
                };
                let mut row = ComboRow::new(name.to_string()).leading(div().flex().size(px(20.)).items_center().justify_center().child(thumb)).debug_name(format!("icon-row-{i}"));
                if !dir.is_empty() {
                    row = row.detail(dir.to_string());
                }
                ComboEntry::from(row)
            })
            .collect();
        let (pick, cancel, clear) = (this.clone(), this.clone(), this.clone());
        let picked = found.clone();
        let empty = found.is_empty();
        let focus = self.input.focus_handle(cx);
        div()
            .id("icon-picker")
            .key_context("IconPicker")
            .flex()
            .flex_col()
            .gap(px(12.))
            .w_full()
            .child(div().text_size(TextSize::Sm.font_size()).font_weight(FontWeight::MEDIUM).child("Choose an icon"))
            .child(
                div()
                    .capture_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| this.key(event, cx)))
                    // The field binds the arrows to its own actions, and an action runs before a key listener.
                    .capture_action(cx.listener(|this, _: &MoveDown, _, cx| {
                        this.step(true, cx);
                        cx.stop_propagation();
                    }))
                    .capture_action(cx.listener(|this, _: &MoveUp, _, cx| {
                        this.step(false, cx);
                        cx.stop_propagation();
                    }))
                    .child(Field::new(focus, Input::new(&self.input).appearance(false).px(px(10.)).text_size(TextSize::Sm.font_size()))),
            )
            .child(
                div().min_h(px(32. * 3.)).child(
                    ComboList::new("icon-list", entries)
                        .style(crate::combobox::ComboStyle::Folder)
                        .padded(false)
                        .active((!empty).then_some(self.active))
                        .on_pick(move |i, _, cx| {
                            if let Some(path) = picked.get(i) {
                                let path: SharedString = path.clone().into();
                                pick.update(cx, |_, cx| cx.emit(IconPickerEvent::Choose(path))).ok();
                            }
                        })
                        .footer(div().when(empty, |d| {
                            d.debug_selector(|| "icon-empty".into())
                                .px(px(10.))
                                .py(px(8.))
                                .text_size(TextSize::Xs.font_size())
                                .text_color(muted)
                                .child("No images match in this project.")
                        })),
                ),
            )
            .child(
                div().flex().items_center().justify_end().gap(px(8.)).child(
                    Button::new("icon-clear").debug_name("icon-clear").label("Use the letter").variant(ButtonVariant::Ghost).on_click(move |_, _, cx| {
                        clear.update(cx, |_, cx| cx.emit(IconPickerEvent::Clear)).ok();
                    }),
                ).child(
                    Button::new("icon-cancel").label("Cancel").variant(ButtonVariant::Ghost).cap("Esc").on_click(move |_, _, cx| {
                        cancel.update(cx, |_, cx| cx.emit(IconPickerEvent::Cancel)).ok();
                    }),
                ),
            )
    }
}
