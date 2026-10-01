use std::{ops::Range, rc::Rc};

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
    ScrollWheelEvent,
    SharedString,
    Styled,
    UniformListScrollHandle,
    Window,
    div,
    uniform_list,
};

use crate::scale::px;
use crate::{
    keys::{self, Press},
    motion::{Channel, Curve, Spring},
    panel_layout::Geometry,
    placement::measure,
    popover::Hang,
    task_board_model::{BoardMove, CARD_GAP, CARD_HEIGHT, COLUMN_WIDTH, Spot, columns, drop_on, geometry, move_cursor, spot_of, task_at},
    task_card::{DraggedTask, TaskCard},
    task_edit::{self, Change, Field, Picker},
    task_keys::{self, TaskCommand},
    task_list_model::{Filters, Group, Sort},
    task_marks::TaskStatusMark,
    task_model::{Assignee, Label, TaskData, TaskStatus},
    task_picker::{Outcome, handle_key, picker_popover},
    theme::{ActiveTheme, radius},
    typography::TextSize,
};
use super::types::{DROP_LIFT, MARGIN, TaskBoardEvent};

pub struct TaskBoard {
    pub(super) tasks: Vec<TaskData>,
    pub(super) me: SharedString,
    pub(super) now: u64,
    pub(super) filters: Filters,
    pub(super) sort: Sort,
    pub(super) columns: Vec<Group>,
    /// What the columns' card lists read: the tasks and each column's indexes, shared so a frame clones a
    /// pointer, not the tasks.
    pub(super) shown: Rc<Vec<TaskData>>,
    pub(super) shown_columns: Vec<Rc<Vec<usize>>>,
    pub(super) geometry: Geometry,
    pub(super) offset: f32,
    pub(super) viewport: f32,
    pub(super) scrolls: Vec<UniformListScrollHandle>,
    /// The card that just landed, and the spring that settles it.
    pub(super) landed: Option<(SharedString, Channel)>,
    /// The card the keyboard is on: its column, then its row.
    pub(super) cursor: Option<Spot>,
    pub(super) picker: Option<Picker>,
    pub(super) people: Vec<Assignee>,
    pub(super) focus: FocusHandle,
}

impl EventEmitter<TaskBoardEvent> for TaskBoard {}

impl Focusable for TaskBoard {
    fn focus_handle(&self, _: &gpui_kit::App) -> FocusHandle {
        self.focus.clone()
    }
}

impl TaskBoard {
    pub fn new(me: impl Into<SharedString>, cx: &mut Context<Self>) -> Self {
        let count = TaskStatus::BOARD_ORDER.len();
        Self {
            tasks: Vec::new(),
            me: me.into(),
            now: 0,
            filters: Filters::default(),
            sort: Sort::default(),
            columns: Vec::new(),
            shown: Rc::default(),
            shown_columns: Vec::new(),
            geometry: geometry(count),
            offset: 0.,
            viewport: 0.,
            scrolls: (0..count).map(|_| UniformListScrollHandle::new()).collect(),
            landed: None,
            cursor: None,
            picker: None,
            people: Vec::new(),
            focus: cx.focus_handle(),
        }
    }

    /// The people and agents a card can be assigned to.
    pub fn set_people(&mut self, people: Vec<Assignee>) {
        self.people = people;
    }

    pub fn cursor(&self) -> Option<Spot> {
        self.cursor
    }

    pub fn set_tasks(&mut self, tasks: Vec<TaskData>, now: u64, cx: &mut Context<Self>) {
        let under = self.cursor_task();
        self.tasks = tasks;
        self.now = now;
        self.recompute(under, cx);
    }

    pub fn set_filters(&mut self, filters: Filters, cx: &mut Context<Self>) {
        let under = self.cursor_task();
        self.filters = filters;
        self.recompute(under, cx);
    }

    pub fn tasks(&self) -> &[TaskData] {
        &self.tasks
    }

    pub fn columns(&self) -> &[Group] {
        &self.columns
    }

    pub(super) fn recompute(&mut self, under: Option<SharedString>, cx: &mut Context<Self>) {
        self.columns = columns(&self.tasks, &self.filters, &self.me, self.sort);
        self.shown = Rc::new(self.tasks.clone());
        self.shown_columns = self.columns.iter().map(|c| Rc::new(c.tasks.clone())).collect();
        // The cursor stays on its card when the card moves; on its place when the card is gone.
        self.cursor = under.as_ref().and_then(|id| spot_of(&self.columns, &self.tasks, id)).or(self.cursor.filter(|s| task_at(&self.columns, *s).is_some()));
        cx.notify();
    }

    /// The id of the task under the keyboard cursor.
    pub(super) fn cursor_task(&self) -> Option<SharedString> {
        let index = task_at(&self.columns, self.cursor?)?;
        Some(self.tasks.get(index)?.id.clone())
    }

    /// Puts the cursor on the card of task `id`, or on the first card when there is none named. A key then acts
    /// on it.
    pub fn put_cursor_on(&mut self, id: Option<&SharedString>, cx: &mut Context<Self>) {
        let spot = match id {
            Some(id) => crate::task_board_model::spot_of(&self.columns, &self.tasks, id),
            None => move_cursor(&self.columns, None, BoardMove::First),
        };
        if let Some(spot) = spot {
            self.reveal(spot);
            cx.notify();
        }
    }

    /// Puts the cursor on `spot` and scrolls to it: sideways to its column, down or up to its card.
    pub(super) fn reveal(&mut self, spot: Spot) {
        self.cursor = Some(spot);
        self.offset = self.geometry.reveal(self.offset, self.viewport, spot.0);
        self.scrolls[spot.0].scroll_to_item(spot.1, ScrollStrategy::Nearest);
    }

    pub(super) fn apply_to(&mut self, id: &SharedString, change: Change, cx: &mut Context<Self>) {
        let ids = vec![id.clone()];
        let by = self.me.to_string();
        if task_edit::apply(&mut self.tasks, &ids, &change, &by, self.now) > 0 {
            if matches!(change, Change::Status(_)) {
                let mut settle = Channel::new(DROP_LIFT);
                settle.animate(0., Curve::Spring(Spring::LAYOUT), 0., cx.reduce_motion());
                self.landed = Some((id.clone(), settle));
            }
            self.recompute(self.cursor.is_some().then_some(id.clone()), cx);
            if let Some(spot) = self.cursor_task().filter(|c| c == id).and(self.cursor) {
                self.reveal(spot);
            }
            cx.emit(TaskBoardEvent::Changed { ids, change });
        }
    }

    pub(super) fn open_picker(&mut self, field: Field, cx: &mut Context<Self>) {
        let Some(index) = self.cursor.and_then(|s| task_at(&self.columns, s)) else { return };
        let task = &self.tasks[index];
        let one = [task];
        let mut labels: Vec<Label> = Vec::new();
        for t in &self.tasks {
            for l in &t.labels {
                if !labels.iter().any(|x| x.name == l.name) {
                    labels.push(l.clone());
                }
            }
        }
        self.picker = Some(match field {
            Field::Status => Picker::status(Some(task.status)),
            Field::Priority => Picker::priority(Some(task.priority)),
            Field::Assignee => Picker::assignee(&self.people, task.assignee.as_ref().map(Assignee::name)),
            Field::Labels => Picker::labels(&labels, &task_edit::shared_labels(&one)),
        });
        cx.notify();
    }

    /// A press on a row of the open picker: the cursor goes there and Enter follows.
    pub(super) fn pick_row(&mut self, at: usize, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(picker) = self.picker.as_mut() {
            picker.set_cursor(at);
        }
        self.key(&crate::task_picker::enter(), window, cx);
    }
    pub(super) fn key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(picker) = self.picker.as_mut() {
            match handle_key(picker, event.keystroke.key.as_str(), event.keystroke.key_char.as_deref()) {
                Outcome::Open => {}
                Outcome::Close => self.picker = None,
                Outcome::Chosen { change, stays_open } => {
                    if !stays_open {
                        self.picker = None;
                    }
                    if let Some(id) = self.cursor_task() {
                        self.apply_to(&id, change, cx);
                    }
                }
            }
            cx.notify();
            cx.stop_propagation();
            return;
        }
        let press = Press::from_keystroke(&event.keystroke);
        let Some(command) = task_keys::read(&press, keys::typing(window)) else { return };
        let go = |board: &mut Self, step: BoardMove, cx: &mut Context<Self>| {
            if let Some(spot) = move_cursor(&board.columns, board.cursor, step) {
                board.reveal(spot);
                cx.notify();
            }
        };
        match command {
            TaskCommand::Down => go(self, BoardMove::Down, cx),
            TaskCommand::Up => go(self, BoardMove::Up, cx),
            TaskCommand::First => go(self, BoardMove::First, cx),
            TaskCommand::Last => go(self, BoardMove::Last, cx),
            TaskCommand::Fold => go(self, BoardMove::Left, cx),
            TaskCommand::Unfold => go(self, BoardMove::Right, cx),
            TaskCommand::Open => {
                if let Some(id) = self.cursor_task() {
                    cx.emit(TaskBoardEvent::Open(id));
                }
            }
            TaskCommand::SetStatus => self.open_picker(Field::Status, cx),
            TaskCommand::SetPriority => self.open_picker(Field::Priority, cx),
            TaskCommand::SetAssignee => self.open_picker(Field::Assignee, cx),
            TaskCommand::SetLabels => self.open_picker(Field::Labels, cx),
            TaskCommand::AssignToMe => {
                let me = self.people.iter().find(|p| *p.name() == self.me).cloned();
                if let (Some(id), Some(me)) = (self.cursor_task(), me) {
                    self.apply_to(&id, Change::Assignee(Some(me)), cx);
                }
            }
            TaskCommand::MoveNext | TaskCommand::MovePrev => {
                if let Some(id) = self.cursor_task() {
                    let shifts = task_edit::shift_status(&self.tasks, std::slice::from_ref(&id), command == TaskCommand::MoveNext);
                    for (id, change) in shifts {
                        self.apply_to(&id, change, cx);
                    }
                }
            }
            TaskCommand::NewTask => cx.emit(TaskBoardEvent::NewTask),
            TaskCommand::Select | TaskCommand::SelectAll | TaskCommand::Clear | TaskCommand::Filter => return,
        }
        cx.stop_propagation();
    }

    /// A card dropped on the column `to`: it takes that status.
    pub fn drop_card(&mut self, id: &SharedString, to: TaskStatus, cx: &mut Context<Self>) {
        let Some(change) = drop_on(&self.tasks, id, to) else { return };
        let ids = vec![id.clone()];
        let by = self.me.to_string();
        if task_edit::apply(&mut self.tasks, &ids, &change, &by, self.now) > 0 {
            let mut settle = Channel::new(DROP_LIFT);
            settle.animate(0., Curve::Spring(Spring::LAYOUT), 0., cx.reduce_motion());
            self.landed = Some((id.clone(), settle));
            let under = self.cursor_task();
            self.recompute(under, cx);
            cx.emit(TaskBoardEvent::Changed { ids, change });
        }
    }

    /// Scrolls every column so `top` pixels of cards are above the view. For measuring runs.
    pub fn set_scroll_top(&mut self, top: f32) {
        for scroll in &self.scrolls {
            scroll.0.borrow().base_handle.set_offset(gpui_kit::point(px(0.), px(-top)));
        }
    }

    pub fn filters(&self) -> &Filters {
        &self.filters
    }

    /// Scrolls the row of columns to `offset`. For measuring runs.
    pub fn scroll_to(&mut self, offset: f32, cx: &mut Context<Self>) {
        self.offset = self.geometry.clamp(offset, self.viewport);
        cx.notify();
    }

    pub(super) fn column_element(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let status = self.columns[index].status;
        let this = cx.entity();
        let open = cx.entity();
        let count = self.columns[index].tasks.len();
        let step = CARD_HEIGHT + CARD_GAP;
        let cursor_row = self.cursor.filter(|(c, _)| *c == index).map(|(_, r)| r);
        let landed = self.landed.as_ref().filter(|(_, c)| c.is_running()).map(|(id, c)| (id.clone(), c.value()));
        if landed.is_some() {
            window.request_animation_frame();
        }
        let tasks = self.shown.clone();
        let rows = self.shown_columns[index].clone();
        let cards = uniform_list(("board-cards", index), count, move |range: Range<usize>, _, _| {
            range
                .map(|at| {
                    let task = tasks[rows[at]].clone();
                    let lift = landed.as_ref().filter(|(id, _)| *id == task.id).map_or(0., |(_, v)| *v);
                    let id = task.id.clone();
                    let open = open.clone();
                    div()
                        .h(px(step))
                        .pb(px(CARD_GAP))
                        .child(TaskCard::new(gpui_kit::ElementId::Name(format!("card-{id}").into()), task).lifted(lift).cursor(cursor_row == Some(at)))
                        .on_mouse_up(gpui_kit::MouseButton::Left, move |_, _, cx| {
                            let id = id.clone();
                            open.update(cx, |_, cx| cx.emit(TaskBoardEvent::Open(id)))
                        })
                        .into_any_element()
                })
                .collect::<Vec<_>>()
        })
        .track_scroll(&self.scrolls[index])
        .w_full()
        .flex_1()
        .min_h_0();
        div()
            .id(("board-column", index))
            .absolute()
            .left(px(self.geometry.left(index)))
            .top_0()
            .bottom_0()
            .w(px(COLUMN_WIDTH))
            .flex()
            .flex_col()
            .rounded(radius::xl())
            .bg(theme.card.opacity(0.5))
            .drag_over::<DraggedTask>(|s, _, _, cx| s.bg(cx.theme().card_strong.opacity(0.9)))
            .on_drop::<DraggedTask>(move |dragged, _, cx| {
                let id = dragged.id.clone();
                this.update(cx, |board, cx| board.drop_card(&id, status, cx))
            })
            .child(
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap(px(8.))
                    .h(px(36.))
                    .px(px(12.))
                    .text_size(TextSize::Sm.font_size())
                    .child(TaskStatusMark::new(status))
                    .child(status.words())
                    .child(div().text_size(TextSize::Xs.font_size()).text_color(theme.muted_foreground).child(count.to_string())),
            )
            .child(div().flex_1().min_h_0().flex().flex_col().px(px(8.)).child(cards))
            .into_any_element()
    }
}

impl Render for TaskBoard {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.landed.as_ref().is_some_and(|(_, c)| !c.is_running()) {
            self.landed = None;
        }
        let visible = self.geometry.visible(self.offset, self.viewport, MARGIN);
        let elements: Vec<AnyElement> = visible.map(|i| self.column_element(i, window, cx)).collect();
        let this = cx.entity();
        let picker = self.picker.as_ref().zip(self.cursor).map(|(p, (column, _))| {
            let left = (self.geometry.left(column) - self.offset + 96.).max(8.);
            picker_popover("task-board-picker", p, &cx.theme().clone(), Hang::Left(left, 64.), cx.entity().downgrade(), |t: &mut Self| t.picker = None, Self::pick_row)
        });
        div()
            .id("task-board")
            .key_context("TaskBoard")
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
            .overflow_hidden()
            .px(px(8.))
            .pb(px(8.))
            .on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, _, cx| {
                let delta = event.delta.pixel_delta(px(16.));
                let (x, y) = (f32::from(delta.x), f32::from(delta.y));
                let dx = if event.modifiers.shift { x + y } else if x.abs() > y.abs() { x } else { 0. };
                if dx != 0. {
                    this.offset = this.geometry.clamp(this.offset - dx, this.viewport);
                    cx.stop_propagation();
                    cx.notify();
                }
            }))
            .child(measure(move |bounds, cx| {
                this.update(cx, |board, cx| {
                    let width = f32::from(bounds.size.width) - 16.;
                    if (board.viewport - width).abs() > 0.5 {
                        board.viewport = width;
                        cx.notify();
                    }
                })
            }))
            .child(div().relative().size_full().left(px(-self.offset)).children(elements))
            .children(picker)
    }
}
