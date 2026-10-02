use gpui_kit::{App, Entity, Hsla, SharedString};

use crate::{motion::{Curve, Spring}, theme::Theme};
use super::structs::{ListMotion, RowMotion, Todo};
use super::types::{RowPlan, TodoStatus};

/// How many steps are done, out of all of them.
pub fn progress(todos: &[Todo]) -> (usize, usize) {
    (todos.iter().filter(|t| t.status == TodoStatus::Done).count(), todos.len())
}

pub(super) fn status_color(status: TodoStatus, theme: &Theme) -> Hsla {
    match status {
        TodoStatus::InProgress => theme.foreground,
        TodoStatus::Cancelled => theme.danger,
        _ => theme.muted_foreground,
    }
}

/// beui's title colors: pending 65%, done 60%, and cancelled 55% of muted; in progress full.
pub(super) fn title_color(status: TodoStatus, theme: &Theme) -> Hsla {
    let muted = theme.muted_foreground;
    match status {
        TodoStatus::Pending => muted.opacity(0.65),
        TodoStatus::InProgress => theme.foreground,
        TodoStatus::Done => muted.opacity(0.6),
        TodoStatus::Cancelled => muted.opacity(0.55),
    }
}

/// Matches each new todo to a previous row by id. Pure, so insertion, removal, and progress changes can
/// be tested without a window.
pub(super) fn plan_rows(prev: &[(SharedString, TodoStatus, Option<f32>)], todos: &[Todo]) -> Vec<RowPlan> {
    todos
        .iter()
        .map(|todo| match prev.iter().position(|(id, ..)| *id == todo.id) {
            Some(i) if prev[i].1 == todo.status && prev[i].2 == todo.progress => RowPlan::Reuse(i),
            Some(i) => RowPlan::Retarget(i),
            None => RowPlan::New,
        })
        .collect()
}

pub(super) fn update(list: &Entity<ListMotion>, todos: &[Todo], reduce: bool, cx: &mut App) {
    let (done, total) = progress(todos);
    let all_done = total > 0 && done == total;
    list.update(cx, |m, _| {
        let prev: Vec<_> = m.rows.iter().map(|r| (r.id.clone(), r.status, r.progress)).collect();
        let plan = plan_rows(&prev, todos);
        let mut old_rows: Vec<Option<RowMotion>> = m.rows.drain(..).map(Some).collect();
        m.rows = plan
            .into_iter()
            .zip(todos.iter())
            .map(|(action, todo)| match action {
                RowPlan::Reuse(i) => old_rows[i].take().expect("each previous row is claimed once"),
                RowPlan::Retarget(i) => {
                    let mut row = old_rows[i].take().expect("each previous row is claimed once");
                    row.retarget(todo.status, todo.progress, reduce);
                    row
                }
                RowPlan::New => RowMotion::new(todo.id.clone(), todo.status, todo.progress),
            })
            .collect();
        if all_done != m.all_done {
            m.all_done = all_done;
            m.header_done.animate(if all_done { 1. } else { 0. }, Curve::Spring(Spring::SWAP), 0., reduce);
            m.disclosure.set_open(!all_done, reduce);
        }
    });
}
