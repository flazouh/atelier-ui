use std::sync::Arc;

use gpui_kit::{
    App,
    ElementId,
    FontWeight,
    IntoElement,
    ParentElement,
    RenderOnce,
    SharedString,
    Styled,
    StyledText,
    Window,
    div,
    prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    agent_look::AgentLook,
    icon::{Icon, IconName},
    morph::Morph,
    theme::{ActiveTheme, radius},
    typography::TextSize,
};
use super::types::ROW_HEIGHT;
use super::helpers::{done_text, tool_calls_text};

#[derive(Clone, IntoElement)]
pub struct SubagentRow {
    id: ElementId,
    look: AgentLook,
    pub(super) title: SharedString,
    pub(super) detail: SharedString,
    pub(super) tool: Option<SharedString>,
    tool_calls: Option<u64>,
    pub(super) elapsed: Option<SharedString>,
    /// Some when the subagent finished; the inner value is how long it ran, in seconds, if known.
    pub(super) finished: Option<Option<u64>>,
}

impl SubagentRow {
    /// `title` names the agent ("Explore", "Review"); `detail` says what it works on.
    pub fn new(
        id: impl Into<ElementId>,
        look: AgentLook,
        title: impl Into<SharedString>,
        detail: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            look,
            title: title.into(),
            detail: detail.into(),
            tool: None,
            tool_calls: None,
            elapsed: None,
            finished: None,
        }
    }

    /// The tool call running now, such as "Read crates/beui/src/theme.rs". It shows in place of the task.
    pub fn tool(mut self, tool: impl Into<SharedString>) -> Self {
        self.tool = Some(tool.into());
        self
    }

    pub fn tool_calls(mut self, count: u64) -> Self {
        self.tool_calls = Some(count);
        self
    }

    /// How long it has run, such as "12s".
    pub fn elapsed(mut self, elapsed: impl Into<SharedString>) -> Self {
        self.elapsed = Some(elapsed.into());
        self
    }

    /// Marks it done, with its run time in seconds if known.
    pub fn finished(mut self, seconds: Option<u64>) -> Self {
        self.finished = Some(seconds);
        self
    }

    pub fn id(&self) -> &ElementId {
        &self.id
    }

    pub fn is_finished(&self) -> bool {
        self.finished.is_some()
    }

    pub fn title(&self) -> &SharedString {
        &self.title
    }

    /// The muted middle text: the live tool call while one runs, else the task.
    pub fn detail(&self) -> &SharedString {
        match (&self.tool, self.finished) {
            (Some(tool), None) => tool,
            _ => &self.detail,
        }
    }

    /// What the right side says: the elapsed time while it runs, then "Done in 38s" or "Done".
    pub fn status_label(&self) -> SharedString {
        match self.finished {
            Some(seconds) => done_text(seconds),
            None => self.elapsed.clone().unwrap_or_default(),
        }
    }
}

impl RenderOnce for SubagentRow {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let done = self.finished.is_some();
        let status = self.status_label();
        let detail = self.detail().clone();
        let child = |name: &'static str| ElementId::NamedChild(Arc::new(self.id.clone()), name.into());
        let title = StyledText::new(self.title.clone());
        let mark = &self.look.mark;

        div()
            .flex()
            .items_center()
            .gap(px(10.))
            .h(px(ROW_HEIGHT))
            .px(px(10.))
            .rounded(radius::lg())
            .bg(theme.card)
            .text_size(TextSize::Xs.font_size())
            .line_height(TextSize::Xs.line_height())
            .child(if done {
                div()
                    .flex()
                    .flex_none()
                    .size(px(16.))
                    .items_center()
                    .justify_center()
                    .text_color(theme.success)
                    .child(Icon::new(IconName::Check).size(px(14.)))
                    .into_any_element()
            } else {
                mark.sprite(child("spark"), mark.orbiting).size(px(16.)).into_any_element()
            })
            .child(
                div()
                    .flex_none()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(if done { muted } else { self.look.message })
                    .child(title),
            )
            .child(div().flex_1().min_w_0().overflow_hidden().text_color(muted).child(Morph::new(
                child("detail"),
                detail.clone(),
                move |_, _| div().truncate().child(detail.clone()).into_any_element(),
            )))
            .when_some(self.tool_calls.filter(|_| !done), |d, count| {
                d.child(div().flex_none().text_color(muted.opacity(0.7)).child(tool_calls_text(count)))
            })
            .when(!status.is_empty(), |d| d.child(div().flex_none().text_color(muted.opacity(0.7)).child(status)))
    }
}
