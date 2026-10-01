use gpui_kit::{ElementId, IntoElement, SharedString};

use crate::scale::px;
use crate::{
    icon::{Icon, IconName},
    spinner::Spinner,
    theme::{Theme},
    };
use super::structs::{CheckRun, Fault, Groups, JobStep};
use super::types::{CheckState, Standing};

pub fn standing(checks: &[CheckRun]) -> Standing {
    let total = checks.len();
    let count = |f: fn(CheckState) -> bool| checks.iter().filter(|c| f(c.state)).count();
    let failed = count(|s| s == CheckState::Failed);
    let waiting = count(|s| matches!(s, CheckState::Running | CheckState::Queued));
    let green = count(|s| s == CheckState::Passed);
    match () {
        _ if total == 0 => Standing::Empty,
        _ if failed > 0 => Standing::Red { failed, total },
        _ if waiting > 0 => Standing::Running { waiting, total },
        _ if green == total => Standing::Passed { total },
        _ => Standing::Stopped { green, total },
    }
}

pub fn groups(checks: &[CheckRun]) -> Groups<'_> {
    let of = |f: fn(CheckState) -> bool| checks.iter().filter(|c| f(c.state)).collect();
    Groups {
        failing: of(|s| s == CheckState::Failed),
        tolerated: of(|s| s == CheckState::Tolerated),
        rest: of(|s| !matches!(s, CheckState::Failed | CheckState::Tolerated)),
    }
}

pub fn tolerated_text(count: usize) -> SharedString {
    format!("{count} allowed to fail").into()
}

/// A step the runner does around the work, rather than the work itself.
pub(super) fn is_chore(step: &JobStep) -> bool {
    let name = step.name.as_ref();
    ["Set up job", "Complete job", "Post ", "Initialize containers", "Stop containers"].iter().any(|p| name.starts_with(p))
        || name.contains("actions/checkout")
        || name.contains("actions/cache")
}

/// How strongly a log line names a cause: a panic or a compiler error outranks an assertion, which
/// outranks a bare "error:", and a "FAILED" test name or a timeout; a line that only restates the exit code ranks
/// lowest, as GitQuiet ranks its Notes.
pub(super) fn rank(line: &str) -> u8 {
    let has = |p: &str| line.contains(p);
    match () {
        _ if has("panicked at") || has("error[E") => 6,
        _ if has("assertion") || has("Assertion") => 5,
        _ if line.trim_start().starts_with("error:") || has("Error:") => 4,
        _ if has("FAILED") || has("FAIL ") || timed_out(line) => 3,
        _ if has("Process completed with exit code") || has("exited with code") => 1,
        _ if line.to_lowercase().contains("error") => 2,
        _ => 0,
    }
}

/// A line that says the work ran out of time ("Timed out after 20ms", "timeout (60s)"): the cause, where
/// the exit code after it only restates that the step stopped.
pub(super) fn timed_out(line: &str) -> bool {
    let lower = line.to_lowercase();
    lower.contains("timed out") || lower.contains("timeout")
}

/// A log line as a reader should see it, as GitQuiet's `linesIn` reads one (`domain/logs.ts`): the
/// colour escapes a tool wrote come off, then the timestamp the runner stored it with, then the
/// runner's own marker (`##[error]`, `##[group]` and the rest), and a workflow command's head
/// (`::error file=…::`). The message after them stays. A marker inside the words is words.
pub fn clean_line(raw: &str) -> String {
    let plain = strip_escapes(raw);
    let rest = strip_timestamp(&plain);
    let rest = strip_marker(rest).unwrap_or(rest);
    strip_command(rest).unwrap_or(rest).to_string()
}

/// Colour escapes: ESC `[`, digits and `;`, then `m`.
pub(super) fn strip_escapes(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' && chars.peek() == Some(&'[') {
            let mut seq = chars.clone();
            seq.next();
            while seq.peek().is_some_and(|c| c.is_ascii_digit() || *c == ';') {
                seq.next();
            }
            if seq.peek() == Some(&'m') {
                seq.next();
                chars = seq;
                continue;
            }
        }
        out.push(c);
    }
    out
}

/// `2026-09-28T12:00:01.1234567Z ` at the start of a line.
pub(super) fn strip_timestamp(line: &str) -> &str {
    let b = line.as_bytes();
    let digits = |r: std::ops::Range<usize>| b.get(r).is_some_and(|s| s.iter().all(u8::is_ascii_digit));
    let date = b.len() > 11 && digits(0..4) && b[4] == b'-' && digits(5..7) && b[7] == b'-' && digits(8..10) && b[10] == b'T';
    if !date {
        return line;
    }
    let time = b[11..].iter().take_while(|c| c.is_ascii_digit() || **c == b':' || **c == b'.').count();
    match b.get(11 + time) {
        Some(b'Z') => line[12 + time..].strip_prefix(' ').unwrap_or(&line[12 + time..]),
        _ => line,
    }
}

/// `##[word]` and one space, at the start.
pub(super) fn strip_marker(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("##[")?;
    let close = rest.find(']')?;
    rest[..close].chars().all(|c| c.is_ascii_lowercase()).then(|| {
        let after = &rest[close + 1..];
        after.strip_prefix(' ').unwrap_or(after)
    })
}

/// `::error ...::`, `::warning::` and the rest, at the start.
pub(super) fn strip_command(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("::")?;
    let name_end = rest.find([' ', ':']).unwrap_or(rest.len());
    let name = &rest[..name_end];
    if !["error", "warning", "notice", "debug"].contains(&name) {
        return None;
    }
    let close = rest[name_end..].find("::")? + name_end;
    Some(&rest[close + 2..])
}

/// The Fault of a failing check: the first failing step that did the work (a chore only when no work
/// failed), and its most telling log line, the first of the strongest rank. With nothing telling, the
/// last line; with no log, nothing.
pub fn fault(check: &CheckRun) -> Option<Fault> {
    if !matches!(check.state, CheckState::Failed | CheckState::Tolerated) {
        return None;
    }
    let failed = |s: &&JobStep| matches!(s.state, CheckState::Failed | CheckState::Tolerated);
    let step = check.steps.iter().filter(|s| !is_chore(s)).find(failed).or_else(|| check.steps.iter().find(failed))?;
    // Ranked and shown clean, so a runner's markup never outranks or hides the words.
    let lines: Vec<String> = step.log.iter().map(|l| clean_line(l)).collect();
    let best = lines.iter().map(|l| (rank(l), l)).filter(|(r, _)| *r > 0).fold(None::<(u8, &String)>, |best, (r, l)| {
        match best {
            Some((b, _)) if b >= r => best,
            _ => Some((r, l)),
        }
    });
    let line: SharedString = best.map(|(_, l)| l.clone()).or_else(|| lines.last().cloned()).unwrap_or_default().into();
    Some(Fault { step: step.name.clone(), line })
}

/// A check's mark: a cross when it failed, a warning when it was allowed to, a turning ring while it
/// runs, a tick once green.
pub(super) fn mark(state: CheckState, id: ElementId, theme: &Theme) -> gpui_kit::AnyElement {
    let icon = |name, color| Icon::new(name).size(px(14.)).color(color).into_any_element();
    match state {
        CheckState::Passed => icon(IconName::CheckCircle, theme.success),
        CheckState::Failed => icon(IconName::Cancel, theme.danger),
        CheckState::Tolerated => icon(IconName::Error, theme.warning),
        CheckState::Running => Spinner::new(id).size(px(14.)).color(theme.warning).into_any_element(),
        CheckState::Queued => icon(IconName::Schedule, theme.muted_foreground),
        CheckState::Skipped => icon(IconName::Block, theme.muted_foreground),
    }
}
