use std::{rc::Rc, sync::Arc};

use gpui_kit::{
    App, Bounds, ElementId, FocusHandle, InteractiveElement, IntoElement, ParentElement, Pixels,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    button::{Button, ButtonVariant},
    button_group::ButtonGroup,
    icon::{Icon, IconName},
    menu::{self, Choice as MenuChoice, Entry, Menu, MenuItem, Origin},
    merge::{Action, Choice, MergeFacts, button, offered},
    placement::measure,
    popover::{Align, Popover, Side},
    theme::ActiveTheme,
    tooltip::Tooltip,
};
use super::types::{ActionHandler, ChoiceHandler, MENU_GAP, Pick};
use super::helpers::close_menu;

#[derive(IntoElement)]
pub struct MergeButton {
    pub(super) id: ElementId,
    pub(super) facts: MergeFacts,
    pub(super) choice: Choice,
    pub(super) upward: bool,
    pub(super) on_action: Option<ActionHandler>,
    pub(super) on_choice: Option<ChoiceHandler>,
}

impl MergeButton {
    pub fn new(id: impl Into<ElementId>, facts: MergeFacts, choice: Choice) -> Self {
        Self { id: id.into(), facts, choice, upward: false, on_action: None, on_choice: None }
    }

    /// Always opens the menu above the button. Without it the menu opens above only when the window
    /// has no room below.
    pub fn upward(mut self, upward: bool) -> Self {
        self.upward = upward;
        self
    }

    /// What a press on the main part asks for.
    pub fn on_action(mut self, f: impl Fn(Action, &mut Window, &mut App) + 'static) -> Self {
        self.on_action = Some(Rc::new(f));
        self
    }

    /// The reader's new choice, each time the menu changes it.
    pub fn on_choice(mut self, f: impl Fn(Choice, &mut Window, &mut App) + 'static) -> Self {
        self.on_choice = Some(Rc::new(f));
        self
    }
}

pub(super) struct MenuState {
    pub(super) open: bool,
    /// The button's bounds in its last layout, to choose where the menu opens.
    pub(super) anchor: Option<Bounds<Pixels>>,
    /// The arrow's focus, which the menu hands back when it closes from the keyboard.
    pub(super) arrow: FocusHandle,
}

impl MenuState {
    pub(super) fn new(cx: &mut App) -> Self {
        Self { open: false, anchor: None, arrow: cx.focus_handle() }
    }

    pub(super) fn set_open(&mut self, open: bool) {
        self.open = open;
    }
}

impl RenderOnce for MergeButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let Some(state) = button(&self.facts, &self.choice) else { return div().into_any_element() };
        let theme = cx.theme().clone();
        let menu = window.use_keyed_state(self.id.clone(), cx, |_, cx| MenuState::new(cx));
        let row_count = offered(&self.facts).len() + usize::from(self.facts.auto_merge.is_some()) + 1;
        let (open, arrow_focus) = menu.read_with(cx, |m, _| (m.open, m.arrow.clone()));
        let child = |name: &str| ElementId::NamedChild(Arc::new(self.id.clone()), name.to_string().into());

        // The action and the arrow are one control: a button group whose action greys when blocked
        // while the arrow stays live, so the method can still change.
        let group = {
            let (action, on_action) = (state.action, self.on_action.clone());
            let main = Button::new(child("main"))
                .label(state.label.clone())
                .debug_name("merge-main")
                .disabled(action.is_none())
                .when_some(state.reason.clone(), |b, reason| b.tooltip(reason))
                .on_click(move |_, window, cx| {
                    if let (Some(action), Some(f)) = (action, on_action.as_ref()) {
                        f(action, window, cx);
                    }
                });
            let (toggle, down) = (menu.clone(), menu.clone());
            let arrow = Button::new(child("arrow"))
                .icon(IconName::ChevronDown)
                .debug_name("merge-arrow")
                .tooltip("Merge options")
                .open(menu.read(cx).open)
                .focus_handle(arrow_focus.clone())
                .on_click(move |_, _, cx| {
                    toggle.update(cx, |m, cx| {
                        m.set_open(!m.open);
                        cx.notify();
                    })
                })
                .on_key(move |event, _, cx| {
                    if event.keystroke.key == "down" {
                        cx.stop_propagation();
                        down.update(cx, |m, cx| {
                            m.set_open(true);
                            cx.notify();
                        })
                    }
                });
            ButtonGroup::new(child("group")).variant(ButtonVariant::Primary).child(main).child(arrow)
        };

        let choose = |next: Choice| -> Pick {
            let on_choice = self.on_choice.clone();
            Rc::new(move |window, cx| {
                if let Some(f) = on_choice.as_ref() {
                    f(next, window, cx);
                }
            })
        };
        let choice = self.choice;
        let dismiss = menu.clone();
        let mut entries: Vec<Entry> = offered(&self.facts)
            .into_iter()
            .map(|method| {
                let pick = choose(Choice { method, ..choice });
                MenuItem::new(method.menu_word())
                    .choice(MenuChoice::Radio(method == choice.method))
                    .debug_name(format!("merge-method-{method:?}"))
                    .on_select(move |window, cx| pick(window, cx))
                    .into()
            })
            .collect();
        entries.push(Entry::Separator);
        if self.facts.auto_merge.is_some() {
            let pick = choose(Choice { auto: !choice.auto, ..choice });
            entries.push(
                MenuItem::new("Merge when ready")
                    .choice(MenuChoice::Check(choice.auto))
                    .debug_name("merge-auto")
                    .on_select(move |window, cx| pick(window, cx))
                    .into(),
            );
        }
        let pick = choose(Choice { delete_branch: !choice.delete_branch, ..choice });
        entries.push(
            MenuItem::new("Delete branch after merging")
                .choice(MenuChoice::Check(choice.delete_branch))
                .debug_name("merge-delete-branch")
                .on_select(move |window, cx| pick(window, cx))
                .into(),
        );
        let panel_height = menu::height_in(menu::MenuLook::MERGE, row_count) + menu::MenuLook::MERGE.group;
        let anchor = menu.read(cx).anchor;
        let panel = Menu::new(child("menu"), entries)
            .look(menu::MenuLook::MERGE)
            .origin(Origin::TopRight)
            .on_dismiss(move |window, cx| close_menu(&dismiss, window.last_input_was_keyboard(), window, cx));

        let close = menu.clone();
        let popover = Popover::new(child("popover"))
            .open(open)
            .anchor(anchor)
            .align(Align::End)
            .side(if self.upward { Side::Above } else { Side::Auto })
            .gap(MENU_GAP)
            .height(panel_height)
            .return_focus(&arrow_focus)
            .on_close({
                let close = close.clone();
                move |window, cx| close_menu(&close, false, window, cx)
            })
            .child(panel);
        let measured = {
            let menu = menu.clone();
            measure(move |bounds, cx| menu.update(cx, |m, _| m.anchor = Some(bounds)))
        };
        div()
            .id(self.id.clone())
            .relative()
            .child(measured)
            .flex()
            .flex_none()
            .items_center()
            .gap(px(6.))
            .when(state.ready, |d| {
                d.child(
                    div()
                        .id(child("ready"))
                        .flex_none()
                        .tooltip(Tooltip::text("Ready to merge"))
                        .text_color(theme.success)
                        .child(Icon::new(IconName::CheckCircle).size(px(14.))),
                )
            })
            .child(group)
            .child(popover)
            .into_any_element()
    }
}
