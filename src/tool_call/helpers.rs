use gpui_kit::{App, Entity};

use super::structs::CallMotion;
use super::types::ToolStatus;

/// Whether the panel should pop open on this frame: a run just started, or output just showed up on a
/// call that started with none (also on a finished call, when it asked to start open). Pure, so both cases are testable without a window.
pub(super) fn should_open(status: ToolStatus, was_running: bool, had_body: bool, has_body: bool, default_open: bool) -> bool {
    let just_started = status == ToolStatus::Running && !was_running;
    let body_just_arrived = (status == ToolStatus::Running || default_open) && has_body && !had_body;
    just_started || body_just_arrived
}

/// Opens when a run starts, or when a running call's output arrives after starting empty; closes when
/// the run ends, unless the call keeps itself open.
pub(super) fn follow_status(motion: &Entity<CallMotion>, status: ToolStatus, has_body: bool, default_open: bool, collapse_on_complete: bool, reduce: bool, cx: &mut App) {
    motion.update(cx, |m, _| {
        let was_running = m.status == ToolStatus::Running;
        let status_changed = m.status != status;
        m.status = status;
        if should_open(status, was_running, m.had_body, has_body, default_open) {
            m.disclosure.set_open(true, reduce);
        } else if status_changed && was_running && collapse_on_complete {
            m.disclosure.set_open(false, reduce);
        }
        m.had_body = has_body;
    });
}
