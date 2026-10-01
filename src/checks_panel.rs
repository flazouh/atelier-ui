//! A pull request's checks, after GitQuiet's `Checks.tsx`: failing checks first and open, then the jobs
//! that were allowed to fail, then the rest behind one line. A failing check shows its Fault with no
//! click: the failing step, and the line of its log that names the cause. A Tolerated job (it failed in
//! a run that succeeded) says "Allowed to fail" and is never why the run is red. A running check spins.

use std::sync::Arc;

use gpui_kit::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, 
};
use crate::scale::px;

use crate::{
    focus::PressStop,
    icon::{Icon, IconName},
    rail_section::{RailSection, SectionTone},
    spinner::Spinner,
    theme::{ActiveTheme, Theme, radius},
    typography::{MONO_FONT_FAMILY, TextSize},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckState {
    Passed,
    Failed,
    /// Failed in a run that succeeded: `continue-on-error`.
    Tolerated,
    Running,
    Queued,
    Skipped,
}

/// One step of a check's job.
#[derive(Clone, Debug, PartialEq)]
pub struct JobStep {
    pub name: SharedString,
    pub state: CheckState,
    pub seconds: Option<u64>,
    /// The step's log, one line each.
    pub log: Vec<SharedString>,
}

/// One check.
#[derive(Clone, Debug, PartialEq)]
pub struct CheckRun {
    pub name: SharedString,
    /// GitHub's one line about it.
    pub summary: SharedString,
    pub state: CheckState,
    pub steps: Vec<JobStep>,
}

/// How the whole run stands, worst first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Standing {
    Empty,
    Red { failed: usize, total: usize },
    Running { waiting: usize, total: usize },
    Passed { total: usize },
    Stopped { green: usize, total: usize },
}

impl Standing {
    pub fn text(self) -> SharedString {
        match self {
            Self::Empty => "Nothing has run yet".into(),
            Self::Red { failed, total } => format!("CI is red \u{2014} {failed} of {total} failing").into(),
            Self::Running { waiting, total } => format!("{waiting} of {total} still running").into(),
            Self::Passed { total: 1 } => "All 1 check passed".into(),
            Self::Passed { total } => format!("All {total} checks passed").into(),
            Self::Stopped { green, total } => format!("{green} of {total} passed, none failing").into(),
        }
    }
}

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

/// The checks in the order the panel shows them.
pub struct Groups<'a> {
    pub failing: Vec<&'a CheckRun>,
    pub tolerated: Vec<&'a CheckRun>,
    pub rest: Vec<&'a CheckRun>,
}

impl Groups<'_> {
    /// The folded line over the rest: "12 passed", or "10 passed, 2 other".
    pub fn rest_text(&self) -> SharedString {
        let passed = self.rest.iter().filter(|c| c.state == CheckState::Passed).count();
        if passed == self.rest.len() {
            format!("{passed} passed").into()
        } else {
            format!("{passed} passed, {} other", self.rest.len() - passed).into()
        }
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

/// Why a check failed, as a reader would say it: the failing step and the line that names the cause.
#[derive(Clone, Debug, PartialEq)]
pub struct Fault {
    pub step: SharedString,
    pub line: SharedString,
}

/// A step the runner does around the work, rather than the work itself.
fn is_chore(step: &JobStep) -> bool {
    let name = step.name.as_ref();
    ["Set up job", "Complete job", "Post ", "Initialize containers", "Stop containers"].iter().any(|p| name.starts_with(p))
        || name.contains("actions/checkout")
        || name.contains("actions/cache")
}

/// How strongly a log line names a cause: a panic or a compiler error outranks an assertion, which
/// outranks a bare "error:", and a "FAILED" test name or a timeout; a line that only restates the exit code ranks
/// lowest, as GitQuiet ranks its Notes.
fn rank(line: &str) -> u8 {
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
fn timed_out(line: &str) -> bool {
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
fn strip_escapes(raw: &str) -> String {
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
fn strip_timestamp(line: &str) -> &str {
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
fn strip_marker(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("##[")?;
    let close = rest.find(']')?;
    rest[..close].chars().all(|c| c.is_ascii_lowercase()).then(|| {
        let after = &rest[close + 1..];
        after.strip_prefix(' ').unwrap_or(after)
    })
}

/// `::error ...::`, `::warning::` and the rest, at the start.
fn strip_command(line: &str) -> Option<&str> {
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

/// The checks, as a rail section.
#[derive(IntoElement)]
pub struct ChecksPanel {
    id: ElementId,
    checks: Vec<CheckRun>,
}

impl ChecksPanel {
    pub fn new(id: impl Into<ElementId>, checks: Vec<CheckRun>) -> Self {
        Self { id: id.into(), checks }
    }
}

/// A check's mark: a cross when it failed, a warning when it was allowed to, a turning ring while it
/// runs, a tick once green.
fn mark(state: CheckState, id: ElementId, theme: &Theme) -> gpui_kit::AnyElement {
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

impl RenderOnce for ChecksPanel {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let open = window.use_keyed_state(self.id.clone(), cx, |_, _| false);
        let rest_open = *open.read(cx);
        let child = |name: String| ElementId::NamedChild(Arc::new(self.id.clone()), name.into());
        let standing = standing(&self.checks);
        let g = groups(&self.checks);
        let tone = match standing {
            Standing::Red { .. } => SectionTone::Bad,
            Standing::Passed { .. } => SectionTone::Done,
            _ => SectionTone::Plain,
        };
        let running = matches!(standing, Standing::Running { .. });

        let row = |check: &CheckRun| {
            let fault = fault(check);
            div()
                .flex()
                .flex_col()
                .gap(px(4.))
                .px(px(12.))
                .py(px(6.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .text_size(TextSize::Xs.font_size())
                        .child(mark(check.state, child(format!("mark-{}", check.name)), &theme))
                        .child(div().flex_none().font_weight(FontWeight::SEMIBOLD).text_color(theme.foreground.opacity(0.9)).child(check.name.clone()))
                        .child(div().flex_1().min_w_0().truncate().text_color(muted).child(check.summary.clone()))
                        .when(check.state == CheckState::Tolerated, |d| d.child(div().flex_none().text_color(theme.warning).child("Allowed to fail"))),
                )
                .when_some(fault, |d, fault| {
                    // The Fault, with no click: which step, and the line that names why.
                    d.child(
                        div()
                            .ml(px(22.))
                            .flex()
                            .flex_col()
                            .gap(px(2.))
                            .p(px(8.))
                            .rounded(radius::md())
                            .bg(theme.card_strong)
                            .text_size(TextSize::Xs.font_size())
                            .child(div().text_color(muted).child(fault.step))
                            .when(!fault.line.is_empty(), |d| {
                                d.child(div().font_family(MONO_FONT_FAMILY).text_size(px(11.)).text_color(theme.foreground.opacity(0.9)).child(fault.line))
                            }),
                    )
                })
        };

        let rest_line = (!g.rest.is_empty()).then(|| {
            let toggle = open.clone();
            div()
                .id(child("rest".into()))
                .flex()
                .items_center()
                .gap(px(8.))
                .px(px(12.))
                .py(px(6.))
                .cursor_pointer()
                .text_size(TextSize::Xs.font_size())
                .text_color(muted)
                .hover(|s| s.bg(theme.muted_hover()))
                .press_stop((self.id.clone(), "rest-focus"), crate::theme::radius::md(), window, cx)
                .on_click(move |_, _, cx| toggle.update(cx, |o, cx| {
                    *o = !*o;
                    cx.notify();
                }))
                .child(Icon::new(if rest_open { IconName::ChevronDown } else { IconName::ChevronRight }).size(px(12.)))
                .child(g.rest_text())
        });

        let summary = div()
            .flex()
            .items_center()
            .gap(px(6.))
            .when(running, |d| d.child(Spinner::new(child("standing".into())).size(px(12.)).color(theme.warning)))
            .child(standing.text());
        RailSection::new("Checks")
            .icon(IconName::Checklist)
            .summary(summary)
            .tone(tone)
            .children(g.failing.iter().map(|c| row(c)))
            .when(!g.tolerated.is_empty(), |d| {
                d.child(div().px(px(12.)).pt(px(4.)).text_size(TextSize::Xs.font_size()).text_color(muted).child(tolerated_text(g.tolerated.len())))
            })
            .children(g.tolerated.iter().map(|c| row(c)))
            .children(rest_line)
            .when(rest_open, |d| d.children(g.rest.iter().map(|c| row(c))))
            .child(div().h(px(4.)))
    }
}

#[cfg(test)]
mod tests;
