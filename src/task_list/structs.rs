use std::ops::Range;

use gpui_kit::{
    AnyElement,
    Context,
    EventEmitter,
    FocusHandle,
    Focusable,
    InteractiveElement,
    IntoElement,
    KeyDownEvent,
    ParentElement,
    Render,
    ScrollStrategy,
    SharedString,
    StatefulInteractiveElement,
    Styled,
    UniformListScrollHandle,
    Window,
    div,
    prelude::FluentBuilder,
    uniform_list,
};

use crate::scale::px;
use crate::{
    button::{Button, ButtonSize, ButtonVariant},
    icon::{Icon, IconName},
    keys::{self, Press},
    popover::Hang,
    task_edit::{self, Change, Field, Picker},
    task_keys::{self, TaskCommand},
    task_list_model::{Cursor, Filters, Folds, Group, Move, Row, Sort, SortKey, group, rows},
    task_marks::TaskStatusMark,
    task_model::{Assignee, Label, TaskData, TaskStatus},
    task_picker::{Outcome, handle_key, picker_popover},
    task_row::{ROW_HEIGHT, TaskRow},
    theme::{ActiveTheme, radius},
    typography::TextSize,
};
use super::types::{ChipAction, TaskListEvent};

pub struct TaskList {
    pub(super) tasks: Vec<TaskData>,
    pub(super) me: SharedString,
    pub(super) now: u64,
    pub(super) people: Vec<Assignee>,
    pub(super) filters: Filters,
    pub(super) sort: Sort,
    pub(super) folds: Folds,
    pub(super) groups: Vec<Group>,
    pub(super) rows: Vec<Row>,
    pub(super) cursor: Cursor,
    pub(super) picker: Option<Picker>,
    /// The open picker sets a filter, not a field of the tasks.
    pub(super) picking_filter: bool,
    /// A filter's picker growing out of its chip.
    pub(super) morph: crate::task_picker::PickerMorph,
    pub(super) scroll: UniformListScrollHandle,
    pub(super) focus: FocusHandle,
}

impl EventEmitter<TaskListEvent> for TaskList {}

impl Focusable for TaskList {
    fn focus_handle(&self, _: &gpui_kit::App) -> FocusHandle {
        self.focus.clone()
    }
}

impl TaskList {
    /// `me` is the reader's name, for the Mine filter and for "Assign to me".
    pub fn new(me: impl Into<SharedString>, cx: &mut Context<Self>) -> Self {
        Self {
            tasks: Vec::new(),
            me: me.into(),
            now: 0,
            people: Vec::new(),
            filters: Filters::default(),
            sort: Sort::default(),
            folds: Folds::default(),
            groups: Vec::new(),
            rows: Vec::new(),
            cursor: Cursor::default(),
            picker: None,
            picking_filter: false,
            morph: Default::default(),
            scroll: UniformListScrollHandle::new(),
            focus: cx.focus_handle(),
        }
    }

    /// The tasks, the time to count from, and the people and agents a task can be assigned to.
    pub fn set_tasks(&mut self, tasks: Vec<TaskData>, people: Vec<Assignee>, now: u64, cx: &mut Context<Self>) {
        let under = self.cursor.task(&self.rows).map(|i| self.tasks[i].id.clone());
        self.tasks = tasks;
        self.people = people;
        self.now = now;
        self.recompute(under.as_ref(), cx);
    }

    pub fn tasks(&self) -> &[TaskData] {
        &self.tasks
    }

    pub fn filters(&self) -> &Filters {
        &self.filters
    }

    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    pub fn set_filters(&mut self, filters: Filters, cx: &mut Context<Self>) {
        let under = self.cursor.task(&self.rows).map(|i| self.tasks[i].id.clone());
        self.filters = filters;
        self.recompute(under.as_ref(), cx);
    }

    pub fn set_sort(&mut self, sort: Sort, cx: &mut Context<Self>) {
        let under = self.cursor.task(&self.rows).map(|i| self.tasks[i].id.clone());
        self.sort = sort;
        self.recompute(under.as_ref(), cx);
    }

    /// Puts the cursor on the task `id`, or on the first task when there is none named. A key then acts on it.
    pub fn put_cursor_on(&mut self, id: Option<&SharedString>, cx: &mut Context<Self>) {
        let row = match id {
            Some(id) => self.rows.iter().position(|r| matches!(r, Row::Task { index } if self.tasks[*index].id == *id)),
            None => self.rows.iter().position(|r| matches!(r, Row::Task { .. })),
        };
        if let Some(row) = row {
            self.cursor.row = Some(row);
            self.scroll.scroll_to_item(row, ScrollStrategy::Nearest);
            cx.notify();
        }
    }
    /// Scrolls the list so `top` pixels of rows are above the view. For measuring runs.
    pub fn set_scroll_top(&mut self, top: f32) {
        self.scroll.0.borrow().base_handle.set_offset(gpui_kit::point(px(0.), px(-top)));
    }

    pub(super) fn recompute(&mut self, follow: Option<&SharedString>, cx: &mut Context<Self>) {
        self.groups = group(&self.tasks, &self.filters, &self.me, self.sort, &TaskStatus::LIST_ORDER, false);
        self.rows = rows(&self.groups, &self.folds);
        self.cursor.follow(&self.rows, &self.tasks, follow);
        cx.notify();
    }

    pub(super) fn labels(&self) -> Vec<Label> {
        let mut all: Vec<Label> = Vec::new();
        for label in self.tasks.iter().flat_map(|t| &t.labels) {
            if !all.contains(label) {
                all.push(label.clone());
            }
        }
        all.sort_by(|a, b| a.name.cmp(&b.name));
        all
    }

    pub(super) fn open_picker(&mut self, field: Field, cx: &mut Context<Self>) {
        let ids = self.cursor.acting_on(&self.rows, &self.tasks);
        if ids.is_empty() {
            return;
        }
        let acted: Vec<&TaskData> = self.tasks.iter().filter(|t| ids.contains(&t.id)).collect();
        let picker = match field {
            Field::Status => Picker::status(task_edit::shared_status(&acted)),
            Field::Priority => Picker::priority(task_edit::shared_priority(&acted)),
            Field::Assignee => {
                let first = acted.first().and_then(|t| t.assignee.as_ref()).map(|a| a.name());
                let shared = acted.iter().all(|t| t.assignee.as_ref().map(|a| a.name()) == first);
                Picker::assignee(&self.people, if shared { first } else { None })
            }
            Field::Labels => Picker::labels(&self.labels(), &task_edit::shared_labels(&acted)),
        };
        self.picker = Some(picker);
        cx.notify();
    }

    /// Opens the picker under a filter chip. A press on the chip of an open picker closes it.
    pub(super) fn open_filter(&mut self, field: Field, cx: &mut Context<Self>) {
        if self.picker.as_ref().is_some_and(|p| p.field() == field) && self.picking_filter {
            self.picker = None;
            self.picking_filter = false;
            cx.notify();
            return;
        }
        let f = &self.filters;
        self.picker = Some(match field {
            Field::Priority => Picker::priority(f.priority),
            Field::Assignee => Picker::assignee(&self.people, f.assignee.as_ref()),
            Field::Labels => {
                let all = self.labels();
                let on: Vec<Label> = all.iter().filter(|l| f.label.as_ref() == Some(&l.name)).cloned().collect();
                Picker::labels(&all, &on)
            }
            Field::Status => return,
        });
        self.picking_filter = true;
        cx.notify();
    }

    /// `]` and `[`: each task acted on moves to the next or previous status, from where it is.
    pub(super) fn shift(&mut self, forward: bool, cx: &mut Context<Self>) {
        let ids = self.cursor.acting_on(&self.rows, &self.tasks);
        let shifts = task_edit::shift_status(&self.tasks, &ids, forward);
        if shifts.is_empty() {
            return;
        }
        let under = self.cursor.task(&self.rows).map(|i| self.tasks[i].id.clone());
        let by = self.me.to_string();
        for (id, change) in &shifts {
            task_edit::apply(&mut self.tasks, std::slice::from_ref(id), change, &by, self.now);
        }
        self.recompute(under.as_ref(), cx);
        for (id, change) in shifts {
            cx.emit(TaskListEvent::Changed { ids: vec![id], change });
        }
    }

    /// Applies a change to the tasks the keys act on, and tells the app.
    pub(super) fn change(&mut self, change: Change, cx: &mut Context<Self>) {
        let ids = self.cursor.acting_on(&self.rows, &self.tasks);
        if ids.is_empty() {
            return;
        }
        let under = self.cursor.task(&self.rows).map(|i| self.tasks[i].id.clone());
        let by = self.me.to_string();
        if task_edit::apply(&mut self.tasks, &ids, &change, &by, self.now) > 0 {
            self.recompute(under.as_ref(), cx);
            cx.emit(TaskListEvent::Changed { ids, change });
        }
    }

    pub(super) fn assign_to_me(&mut self, cx: &mut Context<Self>) {
        if let Some(me) = self.people.iter().find(|p| *p.name() == self.me).cloned() {
            self.change(Change::Assignee(Some(me)), cx);
        }
    }

    pub(super) fn command(&mut self, command: TaskCommand, cx: &mut Context<Self>) {
        match command {
            TaskCommand::Down => self.go(Move::Down, cx),
            TaskCommand::Up => self.go(Move::Up, cx),
            TaskCommand::First => self.go(Move::First, cx),
            TaskCommand::Last => self.go(Move::Last, cx),
            TaskCommand::Select => {
                self.cursor.toggle(&self.rows, &self.tasks);
                cx.notify();
            }
            TaskCommand::SelectAll => {
                self.cursor.select_all(&self.groups, &self.tasks);
                cx.notify();
            }
            TaskCommand::Clear => {
                if self.cursor.clear() {
                    cx.notify();
                }
            }
            TaskCommand::Open => match self.cursor.row.and_then(|r| self.rows.get(r)).copied() {
                Some(Row::Task { index }) => cx.emit(TaskListEvent::Open(self.tasks[index].id.clone())),
                Some(Row::Header { status, .. }) => {
                    self.folds.toggle(status);
                    self.recompute(None, cx);
                }
                None => {}
            },
            TaskCommand::Fold | TaskCommand::Unfold => {
                if let Some(Row::Header { status, .. }) = self.cursor.row.and_then(|r| self.rows.get(r)).copied() {
                    self.folds.set(status, command == TaskCommand::Fold);
                    self.recompute(None, cx);
                }
            }
            TaskCommand::SetStatus => self.open_picker(Field::Status, cx),
            TaskCommand::SetPriority => self.open_picker(Field::Priority, cx),
            TaskCommand::SetAssignee => self.open_picker(Field::Assignee, cx),
            TaskCommand::SetLabels => self.open_picker(Field::Labels, cx),
            TaskCommand::AssignToMe => self.assign_to_me(cx),
            TaskCommand::MoveNext => self.shift(true, cx),
            TaskCommand::MovePrev => self.shift(false, cx),
            TaskCommand::NewTask => cx.emit(TaskListEvent::NewTask),
            TaskCommand::Filter => {
                // The Mine filter is the quickest one; the chips do the rest.
                self.filters.mine = !self.filters.mine;
                let under = self.cursor.task(&self.rows).map(|i| self.tasks[i].id.clone());
                self.recompute(under.as_ref(), cx);
            }
        }
    }

    pub(super) fn go(&mut self, to: Move, cx: &mut Context<Self>) {
        self.cursor.go(&self.rows, to);
        if let Some(row) = self.cursor.row {
            self.scroll.scroll_to_item(row, ScrollStrategy::Nearest);
        }
        cx.notify();
    }

    /// A press on a row of the open picker: the cursor goes there and Enter follows.
    pub(super) fn pick_row(&mut self, at: usize, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(picker) = self.picker.as_mut() {
            picker.set_cursor(at);
        }
        self.picker_key(&crate::task_picker::enter(), cx);
    }
    /// The keys while a picker is open.
    pub(super) fn picker_key(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        let Some(picker) = self.picker.as_mut() else { return };
        match handle_key(picker, event.keystroke.key.as_str(), event.keystroke.key_char.as_deref()) {
            Outcome::Open => {}
            Outcome::Close => {
                self.picker = None;
                self.picking_filter = false;
            }
            Outcome::Chosen { change, stays_open } => {
                if self.picking_filter {
                    self.picker = None;
                    self.picking_filter = false;
                    let next = self.filters.with(&change);
                    self.set_filters(next, cx);
                    return;
                }
                if !stays_open {
                    self.picker = None;
                }
                self.change(change, cx);
            }
        }
        cx.notify();
    }

    pub(super) fn key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.picker.is_some() {
            self.picker_key(event, cx);
            cx.stop_propagation();
            return;
        }
        let press = Press::from_keystroke(&event.keystroke);
        if let Some(command) = task_keys::read(&press, keys::typing(window)) {
            self.command(command, cx);
            cx.stop_propagation();
        }
    }

    /// The words on the chip of a filter: its name, and the value it filters by.
    pub(super) fn chip_label(&self, field: Field) -> String {
        let f = &self.filters;
        match field {
            Field::Assignee => f.assignee.as_ref().map_or("Assignee".to_string(), |a| format!("Assignee: {a}")),
            Field::Labels => f.label.as_ref().map_or("Label".to_string(), |l| format!("Label: {l}")),
            Field::Priority => f.priority.map_or("Priority".to_string(), |p| format!("Priority: {}", p.words())),
            Field::Status => "Status".to_string(),
        }
    }
    /// Whether a filter is on, which lights its chip.
    pub(super) fn chip_active(&self, field: Field) -> bool {
        let f = &self.filters;
        match field {
            Field::Assignee => f.assignee.is_some(),
            Field::Labels => f.label.is_some(),
            Field::Priority => f.priority.is_some(),
            Field::Status => false,
        }
    }
    pub(super) fn chips(&self, cx: &mut Context<Self>) -> AnyElement {
        let this = cx.entity();
        let f = self.filters.clone();
        let picking = self.picker.as_ref().filter(|_| self.picking_filter).map(|p| p.field());
        let chip = |id: &'static str, label: String, active: bool, on: ChipAction| {
            let this = this.clone();
            let owner = this.clone();
            let field = match id {
                "assignee" => Some(Field::Assignee),
                "label" => Some(Field::Labels),
                "priority" => Some(Field::Priority),
                _ => None,
            };
            let button = Button::new(id)
                .open(field.is_some() && field == picking)
                .label(label)
                .variant(if active { ButtonVariant::Secondary } else { ButtonVariant::Ghost })
                .size(ButtonSize::Sm)
                .on_click(move |_, _, cx| this.update(cx, |s, cx| on(s, cx)));
            match field {
                Some(field) => div()
                    .debug_selector(move || format!("filter-{id}"))
                    .relative()
                    .when(self.morph.hides(field), |d| d.opacity(0.))
                    .child(button)
                    .child({
                        crate::placement::measure(move |bounds, cx| {
                            owner.update(cx, |s, _| s.morph.set_anchor(field, bounds));
                        })
                    })
                    .into_any_element(),
                None => button.into_any_element(),
            }
        };
        div()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(4.))
            .px(px(8.))
            .h(px(36.))
            .child(chip("mine", "Mine".into(), f.mine, Box::new(|s, cx| {
                s.filters.mine = !s.filters.mine;
                s.recompute(None, cx);
            })))
            .child(chip(
                "assignee",
                f.assignee.as_ref().map_or("Assignee".to_string(), |a| format!("Assignee: {a}")),
                f.assignee.is_some(),
                Box::new(|s, cx| s.open_filter(Field::Assignee, cx)),
            ))
            .child(chip(
                "label",
                f.label.as_ref().map_or("Label".to_string(), |l| format!("Label: {l}")),
                f.label.is_some(),
                Box::new(|s, cx| s.open_filter(Field::Labels, cx)),
            ))
            .child(chip(
                "priority",
                f.priority.map_or("Priority".to_string(), |p| format!("Priority: {}", p.words())),
                f.priority.is_some(),
                Box::new(|s, cx| s.open_filter(Field::Priority, cx)),
            ))
            .child(div().flex_1())
            .child(chip(
                "sort",
                format!("Sort: {}{}", self.sort.key.words(), if self.sort.reversed { " ↑" } else { "" }),
                false,
                Box::new(|s, cx| {
                    let at = SortKey::ALL.iter().position(|k| *k == s.sort.key).unwrap_or(0);
                    s.sort = if s.sort.reversed || at + 1 == SortKey::ALL.len() {
                        Sort { key: SortKey::ALL[(at + 1) % SortKey::ALL.len()], reversed: false }
                    } else {
                        Sort { key: s.sort.key, reversed: true }
                    };
                    s.recompute(None, cx);
                }),
            ))
            .into_any_element()
    }

    pub(super) fn row_element(&mut self, at: usize, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let this = cx.entity();
        let cursor = self.cursor.row == Some(at);
        match self.rows[at] {
            Row::Header { status, count, open } => {
                let this = this.clone();
                div()
                    .id(("task-group", status as usize))
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap(px(8.))
                    .w_full()
                    .h(px(ROW_HEIGHT))
                    .px(px(10.))
                    .rounded(radius::md())
                    .cursor_pointer()
                    .bg(theme.card.opacity(0.6))
                    .when(cursor, |d| d.bg(theme.card_strong))
                    .text_size(TextSize::Sm.font_size())
                    .on_click(move |_, _, cx| {
                        this.update(cx, |s, cx| {
                            s.cursor.row = Some(at);
                            s.folds.toggle(status);
                            s.recompute(None, cx);
                        })
                    })
                    .child(Icon::new(if open { IconName::ChevronDown } else { IconName::ChevronRight }).size(px(14.)).color(theme.muted_foreground))
                    .child(TaskStatusMark::new(status))
                    .child(status.words())
                    .child(div().text_size(TextSize::Xs.font_size()).text_color(theme.muted_foreground).child(count.to_string()))
                    .into_any_element()
            }
            Row::Task { index } => {
                let task = self.tasks[index].clone();
                let selected = self.cursor.selected.contains(&task.id);
                let id = task.id.clone();
                TaskRow::new(("task", index), task, self.now)
                    .cursor(cursor)
                    .selected(selected)
                    .on_click(move |event, _, cx| {
                        let id = id.clone();
                        this.update(cx, |s, cx| {
                            s.cursor.row = Some(at);
                            if event.modifiers().secondary() || event.modifiers().shift {
                                s.cursor.toggle(&s.rows, &s.tasks);
                                cx.notify();
                            } else {
                                cx.emit(TaskListEvent::Open(id));
                            }
                        })
                    })
                    .into_any_element()
            }
        }
    }
}

impl Render for TaskList {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let reduce = cx.reduce_motion();
        // Only a filter's picker grows out of a chip; one opened on a row hangs from the list.
        self.morph.sync(self.picker.as_ref().filter(|_| self.picking_filter), reduce);
        if self.morph.is_moving() {
            window.request_animation_frame();
        }
        let count = self.rows.len();
        let list = uniform_list("task-rows", count, cx.processor(|this: &mut Self, range: Range<usize>, _, cx| {
            range.map(|at| this.row_element(at, cx)).collect::<Vec<_>>()
        }))
        .track_scroll(&self.scroll)
        .w_full()
        .flex_1()
        .min_h_0();
        let chips = self.chips(cx);
        let window_size = window.viewport_size();
        let picker = self
            .morph
            .field()
            .and_then(|field| {
                let (variant, label) = (
                    if self.chip_active(field) { ButtonVariant::Secondary } else { ButtonVariant::Ghost },
                    self.chip_label(field),
                );
                let (fill, ink) = crate::button::colors(variant, &theme, 1., false);
                let chip = crate::task_picker::Chip { fill, radius: f32::from(radius::lg()), inset: 10. };
                let face = div()
                    .font_family(crate::typography::FONT_FAMILY)
                    .text_size(px(11.))
                    .line_height(px(16.))
                    .whitespace_nowrap()
                    .text_color(ink)
                    .child(label)
                    .into_any_element();
                crate::task_picker::morph_popover(
                    "task-list-picker",
                    &self.morph,
                    &theme,
                    chip,
                    face,
                    window_size,
                    cx.entity().downgrade(),
                    |t: &mut Self| {
                        t.picker = None;
                        t.picking_filter = false;
                    },
                    Self::pick_row,
                )
            })
            .or_else(|| {
                self.picker.as_ref().filter(|_| !self.picking_filter).map(|p| {
                    picker_popover("task-list-picker", p, &theme, Hang::Left(120., 40.), cx.entity().downgrade(), |t: &mut Self| t.picker = None, Self::pick_row)
                })
            });
        div()
            .id("task-list")
            .key_context("TaskList")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| this.key(event, window, cx)))
            .on_mouse_down(
                gpui_kit::MouseButton::Left,
                cx.listener(|this, _, window, cx| {
                    if !this.focus.contains_focused(window, cx) {
                        window.focus(&this.focus, cx)
                    }
                }),
            )
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .child(chips)
            .child(if count == 0 {
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .flex_col()
                    .gap(px(4.))
                    .child(
                        div()
                            .debug_selector(|| "tasks-empty-title".into())
                            .text_size(TextSize::Sm.font_size())
                            .line_height(TextSize::Sm.line_height())
                            .child("No tasks match"),
                    )
                    .child(
                        div()
                            .debug_selector(|| "tasks-empty-line".into())
                            .text_size(TextSize::Xs.font_size())
                            .line_height(TextSize::Xs.line_height())
                            .text_color(theme.muted_foreground)
                            .child("Change a filter, or add one with New task."),
                    )
                    .into_any_element()
            } else {
                div().flex_1().min_h_0().flex().flex_col().px(px(8.)).child(list).into_any_element()
            })
            // Out of the flow: a surface over its chip adds nothing to the column.
            .children(picker.map(|p| div().absolute().child(p)))
    }
}
