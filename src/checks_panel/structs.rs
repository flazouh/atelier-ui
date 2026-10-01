use std::sync::Arc;

use gpui_kit::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    focus::PressStop,
    icon::{Icon, IconName},
    rail_section::{RailSection, SectionTone},
    spinner::Spinner,
    theme::{ActiveTheme, radius},
    typography::{MONO_FONT_FAMILY, TextSize},
};
use super::types::{CheckState, Standing};
use super::helpers::{fault, groups, mark, standing, tolerated_text};

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

/// Why a check failed, as a reader would say it: the failing step and the line that names the cause.
#[derive(Clone, Debug, PartialEq)]
pub struct Fault {
    pub step: SharedString,
    pub line: SharedString,
}

/// The checks, as a rail section.
#[derive(IntoElement)]
pub struct ChecksPanel {
    pub(super) id: ElementId,
    pub(super) checks: Vec<CheckRun>,
}

impl ChecksPanel {
    pub fn new(id: impl Into<ElementId>, checks: Vec<CheckRun>) -> Self {
        Self { id: id.into(), checks }
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
