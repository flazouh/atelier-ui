use gpui_kit::SharedString;

use super::types::{BoardMove, CARD_GAP, CARD_HEIGHT, COLUMN_GAP, COLUMN_WIDTH, Spot};
use crate::{
    panel_layout::{Column, Geometry},
    task_edit::Change,
    task_list_model::{Filters, Group, Sort, group},
    task_model::{TaskData, TaskStatus},
};

/// The columns of a board: one per status, empty ones kept, tasks sorted inside each.
pub fn columns(tasks: &[TaskData], filters: &Filters, me: &str, sort: Sort) -> Vec<Group> {
    group(tasks, filters, me, sort, &TaskStatus::BOARD_ORDER, true)
}

/// Where the columns sit sideways.
pub fn geometry(count: usize) -> Geometry {
    Geometry::new(
        (0..count)
            .map(|i| Column {
                width: COLUMN_WIDTH,
                gap_before: if i == 0 { 0. } else { COLUMN_GAP },
            })
            .collect(),
    )
}

/// The change a drop makes: the dragged task takes the status of the column it was dropped on. `None` when
/// it was dropped on its own column or is not there.
pub fn drop_on(tasks: &[TaskData], id: &SharedString, column: TaskStatus) -> Option<Change> {
    let task = tasks.iter().find(|t| t.id == *id)?;
    (task.status != column).then_some(Change::Status(column))
}

/// The column under `x`, in the row's own coordinates (the pointer's x less the row's left edge plus the
/// scroll). A gap belongs to the column before it, so a drop that misses by a few pixels still lands.
pub fn column_at(geometry: &Geometry, x: f32) -> Option<usize> {
    if geometry.is_empty() || x < 0. || x > geometry.total() {
        return None;
    }
    (0..geometry.len()).rev().find(|&i| x >= geometry.left(i))
}

/// The card rows in a column that a scroll of `top` pixels shows in a viewport `height` tall, and one more
/// each side.
pub fn visible_cards(count: usize, top: f32, height: f32) -> std::ops::Range<usize> {
    let step = CARD_HEIGHT + CARD_GAP;
    let first = ((top / step).floor() as usize).saturating_sub(1).min(count);
    let last = (((top + height) / step).ceil() as usize + 1).min(count);
    first..last.max(first)
}

/// Where the cursor goes. Left and right skip empty columns and keep the row as far as the next column
/// allows; up and down stop at the ends. With no cursor yet, any move lands on the first card of the first
/// column that has one. `None` when there is no card at all.
pub fn move_cursor(columns: &[Group], at: Option<Spot>, step: BoardMove) -> Option<Spot> {
    let first_filled = || {
        columns
            .iter()
            .position(|c| !c.tasks.is_empty())
            .map(|c| (c, 0))
    };
    let Some((column, row)) =
        at.filter(|(c, r)| columns.get(*c).is_some_and(|g| *r < g.tasks.len()))
    else {
        return first_filled();
    };
    let len = columns[column].tasks.len();
    Some(match step {
        BoardMove::Up => (column, row.saturating_sub(1)),
        BoardMove::Down => (column, (row + 1).min(len - 1)),
        BoardMove::First => (column, 0),
        BoardMove::Last => (column, len - 1),
        BoardMove::Left | BoardMove::Right => {
            let next = if step == BoardMove::Left {
                (0..column).rev().find(|&c| !columns[c].tasks.is_empty())
            } else {
                (column + 1..columns.len()).find(|&c| !columns[c].tasks.is_empty())
            };
            match next {
                Some(c) => (c, row.min(columns[c].tasks.len() - 1)),
                None => (column, row),
            }
        }
    })
}

/// The index in `tasks` of the card at `spot`.
pub fn task_at(columns: &[Group], spot: Spot) -> Option<usize> {
    columns.get(spot.0)?.tasks.get(spot.1).copied()
}

/// Where the task `id` sits now, to keep the cursor on it after the columns change.
pub fn spot_of(columns: &[Group], tasks: &[TaskData], id: &SharedString) -> Option<Spot> {
    columns.iter().enumerate().find_map(|(c, g)| {
        g.tasks
            .iter()
            .position(|&i| tasks[i].id == *id)
            .map(|r| (c, r))
    })
}
