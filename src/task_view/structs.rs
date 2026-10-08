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
    SharedString,
    StatefulInteractiveElement,
    Styled,
    Subscription,
    Window,
    div,
    prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    agent_text::AgentText,
    button::{Button, ButtonSize, ButtonVariant},
    comment_composer::{CommentComposer, CommentComposerEvent},
    icon::{Icon, IconName},
    keys::{self, Press},
    popover::Hang,
    pr_chip::PrChip,
    session_row::status_mark,
    sidebar_model::since,
    task_edit::{self, Change, Field, Picker},
    task_keys::{self, TaskCommand},
    task_marks::{PriorityMark, TaskStatusMark},
    task_model::{Activity, Assignee, Label, TaskData, TaskStatus, sub_tasks},
    task_picker::{Outcome, handle_key, picker_popover},
    task_row::{assignee_mark, label_chip},
    theme::{ActiveTheme, radius},
    typography::{MONO_FONT_FAMILY, TextSize},
};
use super::types::{RAIL, TaskViewEvent};

pub struct TaskView {
    pub(super) task: Option<TaskData>,
    all: Vec<TaskData>,
    me: SharedString,
    people: Vec<Assignee>,
    pub(super) labels: Vec<Label>,
    now: u64,
    picker: Option<Picker>,
    /// The hover tone of each property's button, by [`Field`] slot.
    tones: [crate::hover_tone::HoverTone; 4],
    /// The open picker growing out of its property's button.
    morph: crate::task_picker::PickerMorph,
    editing: bool,
    pub(super) description: Entity<CommentComposer>,
    comment: Entity<CommentComposer>,
    focus: FocusHandle,
    _subscriptions: [Subscription; 2],
}

impl EventEmitter<TaskViewEvent> for TaskView {}

impl Focusable for TaskView {
    fn focus_handle(&self, _: &gpui_kit::App) -> FocusHandle {
        self.focus.clone()
    }
}

impl TaskView {
    pub fn new(me: impl Into<SharedString>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let me: SharedString = me.into();
        let description = cx.new(|cx| CommentComposer::new("Description", me.clone(), window, cx).send_label("Save"));
        let comment = cx.new(|cx| CommentComposer::new("Comment", me.clone(), window, cx));
        let saved = cx.subscribe(&description, |this: &mut Self, _, event: &CommentComposerEvent, cx| {
            let CommentComposerEvent::Submit(text) = event;
            if let Some(task) = &mut this.task {
                task.description = text.clone();
                let id = task.id.clone();
                this.editing = false;
                cx.emit(TaskViewEvent::DescriptionSaved { id, text: text.clone() });
                cx.notify();
            }
        });
        let said = cx.subscribe(&comment, |this: &mut Self, _, event: &CommentComposerEvent, cx| {
            let CommentComposerEvent::Submit(text) = event;
            if let Some(task) = &mut this.task {
                task.activity.push(Activity::Comment { author: this.me.clone(), text: text.clone(), at: this.now });
                let id = task.id.clone();
                cx.emit(TaskViewEvent::Commented { id, text: text.clone() });
                cx.notify();
            }
        });
        Self {
            task: None,
            all: Vec::new(),
            me,
            people: Vec::new(),
            labels: Vec::new(),
            now: 0,
            picker: None,
            tones: Default::default(),
            morph: Default::default(),
            editing: false,
            description,
            comment,
            focus: cx.focus_handle(),
            _subscriptions: [saved, said],
        }
    }

    /// Shows a task. `all` are the tasks it may have sub-tasks among; `people` and `labels` fill the pickers.
    pub fn show(&mut self, task: TaskData, all: Vec<TaskData>, people: Vec<Assignee>, labels: Vec<Label>, now: u64, cx: &mut Context<Self>) {
        if self.task.as_ref().map(|t| &t.id) != Some(&task.id) {
            self.editing = false;
            self.picker = None;
        }
        self.task = Some(task);
        self.all = all;
        self.people = people;
        self.labels = labels;
        self.now = now;
        cx.notify();
    }

    pub fn task(&self) -> Option<&TaskData> {
        self.task.as_ref()
    }

    fn open_picker(&mut self, field: Field, cx: &mut Context<Self>) {
        let Some(task) = &self.task else { return };
        let one = [task];
        self.picker = Some(match field {
            Field::Status => Picker::status(Some(task.status)),
            Field::Priority => Picker::priority(Some(task.priority)),
            Field::Assignee => Picker::assignee(&self.people, task.assignee.as_ref().map(|a| a.name())),
            Field::Labels => Picker::labels(&self.labels, &task_edit::shared_labels(&one)),
        });
        cx.notify();
    }

    pub(super) fn change(&mut self, change: Change, cx: &mut Context<Self>) {
        let Some(task) = &mut self.task else { return };
        let ids = [task.id.clone()];
        if task_edit::apply(std::slice::from_mut(task), &ids, &change, &self.me, self.now) > 0 {
            let id = task.id.clone();
            cx.emit(TaskViewEvent::Changed { id, change });
        }
        cx.notify();
    }

    fn edit_description(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(task) = &self.task else { return };
        let text = task.description.clone();
        self.editing = true;
        self.description.update(cx, |d, cx| {
            d.set_text(text, window, cx);
            d.open(window, cx);
        });
        cx.notify();
    }

    /// A press on a row of the open picker: the cursor goes there and Enter follows.
    fn pick_row(&mut self, at: usize, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(picker) = self.picker.as_mut() {
            picker.set_cursor(at);
        }
        self.key(&crate::task_picker::enter(), window, cx);
    }
    fn key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(picker) = self.picker.as_mut() {
            match handle_key(picker, event.keystroke.key.as_str(), event.keystroke.key_char.as_deref()) {
                Outcome::Open => {}
                Outcome::Close => self.picker = None,
                Outcome::Chosen { change, stays_open } => {
                    if !stays_open {
                        self.picker = None;
                    }
                    self.change(change, cx);
                }
            }
            cx.notify();
            cx.stop_propagation();
            return;
        }
        let press = Press::from_keystroke(&event.keystroke);
        let field = match task_keys::read(&press, keys::typing(window)) {
            Some(TaskCommand::SetStatus) => Field::Status,
            Some(TaskCommand::SetPriority) => Field::Priority,
            Some(TaskCommand::SetAssignee) => Field::Assignee,
            Some(TaskCommand::SetLabels) => Field::Labels,
            Some(TaskCommand::AssignToMe) => {
                if let Some(me) = self.people.iter().find(|p| *p.name() == self.me).cloned() {
                    self.change(Change::Assignee(Some(me)), cx);
                }
                cx.stop_propagation();
                return;
            }
            _ => return,
        };
        self.open_picker(field, cx);
        cx.stop_propagation();
    }

    /// A property of the rail: its name, and a button with its value that opens the picker.
    fn property(&self, field: Field, value: AnyElement, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let this = cx.entity();
        div()
            .flex()
            .items_center()
            .gap(px(8.))
            .min_h(px(32.))
            .child(div().w(px(70.)).flex_none().text_size(TextSize::Xs.font_size()).text_color(theme.muted_foreground).child(field.words()))
            .child(
                div()
                    .id(field.words())
                    .debug_selector(move || format!("rail-{}", field.words().to_lowercase()))
                    .relative()
                    .when(self.morph.hides(field), |d| d.opacity(0.))
                    .child({
                        let this = this.clone();
                        crate::placement::measure(move |bounds, cx| {
                            this.update(cx, |s, _| s.morph.set_anchor(field, bounds));
                        })
                    })
                    .flex()
                    .flex_1()
                    .min_w_0()
                    .items_center()
                    .gap(px(8.))
                    .min_h(px(28.))
                    .px(px(8.))
                    .rounded(radius::md())
                    .cursor_pointer()
                    // The Ghost hover tone, eased, and held while this field's picker is open.
                    .bg(crate::select::trigger_tone(&theme, gpui_kit::transparent_black(), self.tones[field.slot()].level()))
                    .on_hover({
                        let this = this.clone();
                        move |on, _, cx| {
                            this.update(cx, |s, cx| {
                                s.tones[field.slot()].set_hovered(*on);
                                cx.notify();
                            })
                        }
                    })
                    .on_click(move |_, _, cx| this.update(cx, |s, cx| s.open_picker(field, cx)))
                    .child(value),
            )
            .into_any_element()
    }

    /// What a property's button shows: the value in it, or its name when it has none.
    fn face(&self, field: Field, theme: &crate::theme::Theme) -> AnyElement {
        let muted = theme.muted_foreground;
        let Some(task) = self.task.as_ref() else { return div().into_any_element() };
        match field {
            Field::Status => div().flex().items_center().gap(px(8.)).child(TaskStatusMark::new(task.status)).child(task.status.words()).into_any_element(),
            Field::Priority => div().flex().items_center().gap(px(8.)).child(PriorityMark::new(task.priority)).child(task.priority.words()).into_any_element(),
            Field::Assignee => match &task.assignee {
            Some(a) => div()
                .flex()
                .items_center()
                .gap(px(8.))
                .child(assignee_mark("rail-assignee", a, 18., theme))
                .child(div().truncate().child(a.name().clone()))
                .into_any_element(),
            None => div().text_color(muted).child("Unassigned").into_any_element(),
        },
            Field::Labels => if task.labels.is_empty() {
            div().text_color(muted).child("No labels").into_any_element()
        } else {
            div().flex().flex_wrap().gap(px(4.)).children(task.labels.iter().map(|l| label_chip(l, theme))).into_any_element()
        },
        }
    }
    pub(super) fn rail(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let Some(task) = self.task.clone() else { return div().into_any_element() };
        let reduce = cx.reduce_motion();
        for field in [Field::Status, Field::Priority, Field::Assignee, Field::Labels] {
            let held = self.picker.as_ref().is_some_and(|p| p.field() == field);
            self.tones[field.slot()].sync(held, reduce);
        }
        self.morph.sync(self.picker.as_ref(), reduce);
        if self.tones.iter().any(|t| t.is_moving()) || self.morph.is_moving() {
            window.request_animation_frame();
        }
        let this = cx.entity();
        let sessions: Vec<AnyElement> = task.sessions.iter().enumerate().map(|(i, s)| {
            let this = this.clone();
            let id = s.id.clone();
            let mark = status_mark(("linked-session", i), &s.look, &s.status, window, cx);
            div()
                .id(("session-link", i))
                .flex()
                .items_center()
                .gap(px(8.))
                .h(px(28.))
                .px(px(8.))
                .rounded(radius::md())
                .cursor_pointer()
                .hover(|st| st.bg(theme.card_strong))
                .on_click(move |_, _, cx| {
                    let id = id.clone();
                    this.update(cx, |_, cx| cx.emit(TaskViewEvent::OpenSession(id)))
                })
                .child(mark)
                .child(div().flex_1().min_w_0().truncate().child(s.title.clone()))
                .into_any_element()
        }).collect();
        let start = task.assignee.as_ref().is_some_and(Assignee::is_agent).then(|| {
            let this = this.clone();
            let id = task.id.clone();
            Button::new("start-session")
                .label("Start a session")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .on_click(move |_, _, cx| {
                    let id = id.clone();
                    this.update(cx, |_, cx| cx.emit(TaskViewEvent::StartSession(id)))
                })
        });
        let status_row = self.property(Field::Status, self.face(Field::Status, &theme), cx);
        let priority_row = self.property(Field::Priority, self.face(Field::Priority, &theme), cx);
        let assignee_row = self.property(Field::Assignee, self.face(Field::Assignee, &theme), cx);
        let labels_row = self.property(Field::Labels, self.face(Field::Labels, &theme), cx);
        div()
            .flex_none()
            .w(px(RAIL))
            .h_full()
            .flex()
            .flex_col()
            .gap(px(2.))
            .px(px(16.))
            .py(px(24.))
            .bg(theme.card.opacity(0.6))
            .text_size(TextSize::Sm.font_size())
            .child(status_row)
            .child(priority_row)
            .child(assignee_row)
            .child(labels_row)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .min_h(px(32.))
                    .child(div().w(px(70.)).flex_none().text_size(TextSize::Xs.font_size()).text_color(muted).child("Project"))
                    .child(div().px(px(8.)).truncate().child(task.project.clone().unwrap_or_else(|| "No project".into()))),
            )
            .when(!task.sessions.is_empty() || start.is_some(), |d| {
                d.child(div().mt(px(16.)).mb(px(4.)).text_size(TextSize::Xs.font_size()).text_color(muted).child("Sessions"))
            })
            .children(sessions)
            .children(start)
            .when(!task.prs.is_empty(), |d| d.child(div().mt(px(16.)).mb(px(4.)).text_size(TextSize::Xs.font_size()).text_color(muted).child("Pull requests")))
            .child(div().flex().flex_col().items_start().gap(px(4.)).children(task.prs.iter().enumerate().map(|(i, pr)| {
                let this = this.clone();
                PrChip::new(("linked-pr", i), pr.clone()).on_open(move |data, _, cx| {
                    let number = data.number;
                    this.update(cx, |_, cx| cx.emit(TaskViewEvent::OpenPr(number)))
                })
            })))
            .into_any_element()
    }

    pub(super) fn activity(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let Some(task) = &self.task else { return div().into_any_element() };
        let mut entries: Vec<&Activity> = task.activity.iter().collect();
        entries.sort_by_key(|a| a.at());
        div()
            .flex()
            .flex_col()
            .gap(px(2.))
            .children(entries.into_iter().map(|entry| {
                let comment = if let Activity::Comment { text, .. } = entry { Some(text.clone()) } else { None };
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .py(px(6.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .text_size(TextSize::Xs.font_size())
                            .text_color(theme.muted_foreground)
                            .child(Icon::new(match entry {
                                Activity::Comment { .. } => IconName::ChatBubble,
                                Activity::StatusChanged { .. } => IconName::Check,
                                Activity::SessionStarted { .. } => IconName::Bot,
                                Activity::PrOpened { .. } | Activity::PrMerged { .. } => IconName::PrMerged,
                                Activity::Created { .. } => IconName::Add,
                                Activity::Committed { .. } => IconName::Check,
                            }).size(px(14.)))
                            .child(entry.words())
                            .child(div().flex_1())
                            .child(since(self.now, entry.at())),
                    )
                    .children(comment.map(|text| div().pl(px(22.)).text_size(TextSize::Sm.font_size()).text_color(theme.foreground).child(text)))
            }))
            .into_any_element()
    }
}

impl Render for TaskView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let Some(task) = self.task.clone() else {
            return div().size_full().flex().items_center().justify_center().text_color(muted).child("Pick a task").into_any_element();
        };
        let this = cx.entity();
        let (subs, done) = {
            let (subs, done) = sub_tasks(&self.all, &task);
            (subs.into_iter().cloned().collect::<Vec<_>>(), done)
        };
        let description: AnyElement = if self.editing {
            self.description.clone().into_any_element()
        } else if task.description.is_empty() {
            let this = this.clone();
            div()
                .id("add-description")
                .px(px(10.))
                .py(px(8.))
                .rounded(radius::md())
                .cursor_pointer()
                .text_color(muted)
                .hover(|s| s.bg(theme.card_strong))
                .on_click(move |_, window, cx| this.update(cx, |s, cx| s.edit_description(window, cx)))
                .child("Add a description")
                .into_any_element()
        } else {
            let this = this.clone();
            div()
                .id("description")
                .group("description")
                .relative()
                .child(AgentText::new(("task-description", task.id.to_string().len()), task.description.clone()))
                .child(
                    div().absolute().top_0().right_0().child(
                        Button::new("edit-description")
                            .label("Edit")
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Sm)
                            .on_click(move |_, window, cx| this.update(cx, |s, cx| s.edit_description(window, cx))),
                    ),
                )
                .into_any_element()
        };
        let sub_section = (!subs.is_empty()).then(|| {
            let share = done as f32 / subs.len() as f32;
            let this = this.clone();
            div()
                .flex()
                .flex_col()
                .gap(px(6.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .text_size(TextSize::Sm.font_size())
                        .font_weight(FontWeight::MEDIUM)
                        .child("Sub-tasks")
                        .child(div().text_size(TextSize::Xs.font_size()).text_color(muted).child(format!("{done} of {}", subs.len())))
                        .child(div().w(px(96.)).h(px(4.)).rounded_full().bg(theme.card_strong).child(div().h_full().w(px(96. * share)).rounded_full().bg(theme.success))),
                )
                .children(subs.into_iter().enumerate().map(|(i, sub)| {
                    let this = this.clone();
                    let id = sub.id.clone();
                    div()
                        .id(("sub-task", i))
                        .flex()
                        .text_size(TextSize::Sm.font_size())
                        .items_center()
                        .gap(px(8.))
                        .h(px(28.))
                        .px(px(8.))
                        .rounded(radius::md())
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.card_strong))
                        .on_click(move |_, _, cx| {
                            let id = id.clone();
                            this.update(cx, |_, cx| cx.emit(TaskViewEvent::OpenTask(id)))
                        })
                        .child(TaskStatusMark::new(sub.status))
                        .child(div().w(px(58.)).font_family(MONO_FONT_FAMILY).text_size(TextSize::Xs.font_size()).text_color(muted).child(sub.key.clone()))
                        .child(div().flex_1().min_w_0().truncate().text_color(if sub.status == TaskStatus::Done { muted } else { theme.foreground }).child(sub.title.clone()))
                }))
        });
        let rail = self.rail(window, cx);
        // The picker grows out of its property's button; one with no button to grow from hangs as before.
        let window_size = window.viewport_size();
        let picker = self
            .morph
            .field()
            .and_then(|field| {
                let chip = crate::task_picker::Chip {
                    fill: crate::select::trigger_tone(&theme, gpui_kit::transparent_black(), self.tones[field.slot()].level()),
                    radius: 6.,
                    inset: 8.,
                };
                crate::task_picker::morph_popover(
                    "task-view-picker",
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
                    picker_popover("task-view-picker", p, &theme, Hang::Right(RAIL + 8., 72.), cx.entity().downgrade(), |t: &mut Self| t.picker = None, Self::pick_row)
                })
            });
        div()
            .id("task-view")
            .key_context("TaskView")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| this.key(event, window, cx)))
            .relative()
            .size_full()
            .flex()
            .child(
                div()
                    .id("task-main")
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .overflow_y_scroll()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(20.))
                            .max_w(px(720.))
                            .px(px(32.))
                            .py(px(24.))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(6.))
                                    .child(div().font_family(MONO_FONT_FAMILY).text_size(TextSize::Xs.font_size()).text_color(muted).child(task.key.clone()))
                                    .child(div().text_size(TextSize::Xl.font_size()).font_weight(FontWeight::SEMIBOLD).text_color(theme.foreground).child(task.title.clone())),
                            )
                            .child(description)
                            .children(sub_section)
                            .child(div().text_size(TextSize::Sm.font_size()).font_weight(FontWeight::MEDIUM).child("Activity"))
                            .child(self.activity(cx))
                            .child(self.comment.clone()),
                    ),
            )
            .child(rail)
            // Out of the flow: a surface over its button adds nothing to the column.
            .children(picker.map(|p| div().absolute().child(p)))
            .into_any_element()
    }
}
