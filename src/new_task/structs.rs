use super::SubmitTask;

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyElement,
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
    StatefulInteractiveElement,
    Styled,
    Subscription,
    Window,
    component::input::{Input, InputState, Textarea, TextareaState},
    div,
};

use crate::scale::px;
use crate::{
    button::{Button, ButtonSize, ButtonVariant},
    new_task_model::{Draft, Submit},
    popover::Hang,
    task_edit::{Change, Field, Picker},
    task_marks::{PriorityMark, TaskStatusMark},
    task_model::{Assignee, Label},
    task_picker::{Outcome, handle_key, picker_popover},
    task_row::{assignee_mark, label_chip},
    theme::{ActiveTheme, radius},
    typography::TextSize,
};
use super::types::NewTaskEvent;

pub struct NewTask {
    pub(super) draft: Draft,
    people: Vec<Assignee>,
    pub(super) labels: Vec<Label>,
    picker: Option<Picker>,
    /// The hover tone of each field's button, by [`Field`] slot.
    tones: [crate::hover_tone::HoverTone; 4],
    /// The open picker growing out of its field's chip.
    morph: crate::task_picker::PickerMorph,
    pub(super) title: Entity<InputState>,
    pub(super) description: Entity<TextareaState>,
    /// Escape was pressed on a draft with words in it: the dialog asks before it drops them.
    confirming: bool,
    focus: FocusHandle,
    _subscription: Subscription,
}

impl EventEmitter<NewTaskEvent> for NewTask {}

impl Focusable for NewTask {
    /// The title: a modal that takes this handle on open puts the caret where the reader types.
    fn focus_handle(&self, cx: &gpui_kit::App) -> FocusHandle {
        self.title.focus_handle(cx)
    }
}

impl NewTask {
    pub fn new(people: Vec<Assignee>, labels: Vec<Label>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let title = cx.new(|cx| InputState::new(window, cx).placeholder("Task title"));
        let description = cx.new(|cx| TextareaState::new(window, cx).auto_grow(4, 10).placeholder("Add a description"));
        let subscription = cx.subscribe_in(&title, window, |this: &mut Self, _, event: &gpui_kit::component::input::InputEvent, _, cx| {
            if let gpui_kit::component::input::InputEvent::Change = event {
                this.draft.title = this.title.read(cx).value().to_string();
                cx.notify();
            }
        });
        Self { draft: Draft::default(), people, labels, picker: None, tones: Default::default(), morph: Default::default(), title, description, confirming: false, focus: cx.focus_handle(), _subscription: subscription }
    }

    /// Empties the dialog and puts the caret in the title. Call it each time the dialog opens.
    pub fn reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.draft = Draft::default();
        self.picker = None;
        self.confirming = false;
        self.title.update(cx, |t, cx| {
            t.set_value("", window, cx);
            t.focus(window, cx);
        });
        self.description.update(cx, |t, cx| t.set_value("", window, cx));
        cx.notify();
    }

    pub fn draft(&self) -> &Draft {
        &self.draft
    }

    pub fn set_people(&mut self, people: Vec<Assignee>, labels: Vec<Label>) {
        self.people = people;
        self.labels = labels;
    }

    fn open_picker(&mut self, field: Field, window: &mut Window, cx: &mut Context<Self>) {
        self.picker = Some(match field {
            Field::Status => Picker::status(Some(self.draft.status)),
            Field::Priority => Picker::priority(Some(self.draft.priority)),
            Field::Assignee => Picker::assignee(&self.people, self.draft.assignee.as_ref().map(Assignee::name)),
            Field::Labels => Picker::labels(&self.labels, &self.draft.labels),
        });
        // Keys go to the dialog while the picker is open, not into the title.
        window.focus(&self.focus, cx);
        cx.notify();
    }

    fn close_picker(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.picker = None;
        self.title.update(cx, |t, cx| t.focus(window, cx));
        cx.notify();
    }

    fn choose(&mut self, change: Change) {
        match change {
            Change::Status(s) => self.draft.status = s,
            Change::Priority(p) => self.draft.priority = p,
            Change::Assignee(a) => self.draft.assignee = a,
            Change::ToggleLabel(label) => {
                if let Some(at) = self.draft.labels.iter().position(|l| l.name == label.name) {
                    self.draft.labels.remove(at);
                } else {
                    self.draft.labels.push(label);
                }
            }
        }
    }

    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.draft.title = self.title.read(cx).value().to_string();
        self.draft.description = self.description.read(cx).value().to_string();
        if !self.draft.can_create() {
            return;
        }
        let start_session = self.draft.submit() == Submit::CreateAndStart;
        cx.emit(NewTaskEvent::Create { draft: self.draft.clone(), start_session });
        let _ = window;
    }

    /// A press on a row of the open picker: the cursor goes there and Enter follows.
    fn pick_row(&mut self, at: usize, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(picker) = self.picker.as_mut() {
            picker.set_cursor(at);
        }
        self.key(&crate::task_picker::enter(), window, cx);
    }
    /// Whether the dialog holds words the reader typed.
    pub fn dirty(&self, cx: &gpui_kit::App) -> bool {
        !self.title.read(cx).value().trim().is_empty() || !self.description.read(cx).value().trim().is_empty()
    }
    /// The reader asked to leave: a draft with words in it is asked about first, an empty one goes.
    pub fn ask_cancel(&mut self, cx: &mut Context<Self>) {
        if self.dirty(cx) {
            self.confirming = true;
            cx.notify();
        } else {
            cx.emit(NewTaskEvent::Cancel);
        }
    }
    /// What the description holds.
    pub fn description_text(&self, cx: &gpui_kit::App) -> String {
        self.description.read(cx).value().to_string()
    }
    pub(super) fn key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();
        if let Some(picker) = self.picker.as_mut() {
            match handle_key(picker, key, event.keystroke.key_char.as_deref()) {
                Outcome::Open => {}
                Outcome::Close => self.close_picker(window, cx),
                Outcome::Chosen { change, stays_open } => {
                    self.choose(change);
                    if !stays_open {
                        self.close_picker(window, cx);
                    }
                }
            }
            cx.notify();
            cx.stop_propagation();
            return;
        }
        if key == "enter" && event.keystroke.modifiers.secondary() {
            self.submit(window, cx);
            cx.stop_propagation();
        } else if key == "escape" {
            if self.confirming {
                self.confirming = false;
                cx.notify();
            } else {
                self.ask_cancel(cx);
            }
            cx.stop_propagation();
        }
    }

    /// What a field's chip shows: the value in it, or its name when it has none.
    fn face(&self, field: Field, theme: &crate::theme::Theme) -> AnyElement {
        let muted = theme.muted_foreground;
        match field {
            Field::Status => {
                div().flex().items_center().gap(px(6.)).child(TaskStatusMark::new(self.draft.status)).child(self.draft.status.words()).into_any_element()
            }
            Field::Priority => {
                div().flex().items_center().gap(px(6.)).child(PriorityMark::new(self.draft.priority)).child(self.draft.priority.words()).into_any_element()
            }
            Field::Assignee => {
                match &self.draft.assignee {
                    Some(a) => div().flex().items_center().gap(px(6.)).child(assignee_mark("new-assignee", a, 16., theme)).child(a.name().clone()).into_any_element(),
                    None => div().text_color(muted).child("Assignee").into_any_element(),
                }
            }
            Field::Labels => {
                if self.draft.labels.is_empty() {
                    div().text_color(muted).child("Labels").into_any_element()
                } else {
                    div().flex().gap(px(4.)).children(self.draft.labels.iter().map(|l| label_chip(l, theme))).into_any_element()
                }
            }
        }
    }
    fn property_button(&self, field: Field, value: AnyElement, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let this = cx.entity();
        let anchor = {
            let this = this.clone();
            crate::placement::measure(move |bounds, cx| {
                this.update(cx, |s, _| s.morph.set_anchor(field, bounds));
            })
        };
        div()
            .id(field.words())
            .debug_selector(move || format!("new-task-{}", field.words().to_lowercase()))
            .relative()
            .when(self.morph.hides(field), |d| d.opacity(0.))
            .child(anchor)
            .flex()
            .items_center()
            .gap(px(6.))
            .h(px(28.))
            .px(px(10.))
            .rounded(radius::md())
            // The Ghost hover tone on the field, held while its picker is open.
            .bg(crate::select::trigger_tone(&theme, theme.card_strong, self.tones[field.slot()].level()))
            .cursor_pointer()
            .text_size(TextSize::Sm.font_size())
            .on_hover({
                let this = this.clone();
                move |on, _, cx| {
                    this.update(cx, |s, cx| {
                        s.tones[field.slot()].set_hovered(*on);
                        cx.notify();
                    })
                }
            })
            .on_click(move |_, window, cx| this.update(cx, |s, cx| s.open_picker(field, window, cx)))
            .child(value)
            .into_any_element()
    }
}

impl Render for NewTask {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let reduce = cx.reduce_motion();
        for field in [Field::Status, Field::Priority, Field::Assignee, Field::Labels] {
            let held = self.picker.as_ref().is_some_and(|p| p.field() == field);
            self.tones[field.slot()].sync(held, reduce);
        }
        self.morph.sync(self.picker.as_ref(), reduce);
        if self.tones.iter().any(|t| t.is_moving()) || self.morph.is_moving() {
            window.request_animation_frame();
        }
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let draft = self.draft.clone();
        let this = cx.entity();
        let cancel = cx.entity();
        let status = self.property_button(Field::Status, self.face(Field::Status, &theme), cx);
        let priority = self.property_button(Field::Priority, self.face(Field::Priority, &theme), cx);
        let assignee = self.property_button(Field::Assignee, self.face(Field::Assignee, &theme), cx);
        let labels = self.property_button(Field::Labels, self.face(Field::Labels, &theme), cx);
        // The picker grows out of its field's chip; one with no chip to grow from hangs as before.
        let window_size = window.viewport_size();
        let picker = self
            .morph
            .field()
            .and_then(|field| {
                let chip = crate::task_picker::Chip {
                    fill: crate::select::trigger_tone(&theme, theme.card_strong, self.tones[field.slot()].level()),
                    radius: 6.,
                    inset: 10.,
                };
                crate::task_picker::morph_popover(
                    "new-task-picker",
                    &self.morph,
                    &theme,
                    chip,
                    self.face(field, &theme),
                    window_size,
                    cx.entity().downgrade(),
                    |t: &mut Self| t.picker = None,
                    Self::pick_row,
                )
            })
            .or_else(|| {
                self.picker.as_ref().filter(|p| !self.morph.can_morph(p.field())).map(|p| {
                    picker_popover("new-task-picker", p, &theme, Hang::Left(16., 170.), cx.entity().downgrade(), |t: &mut Self| t.picker = None, Self::pick_row)
                })
            });
        let footer = if self.confirming {
            let (keep, discard) = (cx.entity(), cx.entity());
            div()
                .flex()
                .items_center()
                .justify_end()
                .gap(px(8.))
                .child(div().debug_selector(|| "discard-question".into()).flex_1().text_size(TextSize::Sm.font_size()).child("Discard this task?"))
                .child(
                    Button::new("keep-new-task")
                        .label("Keep editing")
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .cap("Esc")
                        .on_click(move |_, _, cx| keep.update(cx, |s, cx| {
                            s.confirming = false;
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("discard-new-task")
                        .debug_name("discard-new-task")
                        .label("Discard")
                        .variant(ButtonVariant::Secondary)
                        .size(ButtonSize::Sm)
                        .on_click(move |_, _, cx| discard.update(cx, |_, cx| cx.emit(NewTaskEvent::Cancel))),
                )
                .into_any_element()
        } else {
            div()
                    .flex()
                    .justify_end()
                    .gap(px(8.))
                    .child(
                        Button::new("cancel-new-task")
                            .label("Cancel")
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Sm)
                            .on_click(move |_, _, cx| cancel.update(cx, |s, cx| s.ask_cancel(cx))),
                    )
                    .child(
                        Button::new("create-task")
                            .label(draft.submit_words())
                            .variant(ButtonVariant::Primary)
                            .size(ButtonSize::Sm)
                            .disabled(!draft.can_create())
                            .cap(crate::keys::cap("⌘↵"))
                            .on_click(move |_, window, cx| this.update(cx, |s, cx| s.submit(window, cx))),
                    )
                .into_any_element()
        };
        div()
            .id("new-task")
            .key_context("NewTask")
            .track_focus(&self.focus)
            .on_action(cx.listener(|this, _: &SubmitTask, window, cx| this.submit(window, cx)))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| this.key(event, window, cx)))
            .relative()
            .flex()
            .flex_col()
            .gap(px(12.))
            .w_full()
            .child(div().text_size(TextSize::Xs.font_size()).text_color(muted).font_weight(FontWeight::MEDIUM).child("New task"))
            .child(Input::new(&self.title).appearance(false).bordered(false).text_size(TextSize::Lg.font_size()))
            .child(
                div()
                    .id("new-task-description")
                    .debug_selector(|| "new-task-description".into())
                    .cursor_text()
                    .on_click(cx.listener(|this, _, window, cx| this.description.update(cx, |t, cx| t.focus(window, cx))))
                    .child(Textarea::new(&self.description).appearance(false).text_size(TextSize::Sm.font_size())),
            )
            .child(div().flex().flex_wrap().gap(px(6.)).child(status).child(priority).child(assignee).child(labels))
            .child(footer)
            // Out of the flow: a surface over its chip must not add a gap to the column.
            .children(picker.map(|p| gpui_kit::div().absolute().child(p)))
    }
}
