use std::rc::Rc;

use gpui_kit::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
};

use super::helpers::link_actions;
use super::types::PrHandler;
use crate::scale::px;
use crate::{
    copy_feedback::CopyFeedback,
    focus::PressStop,
    icon::Icon,
    merge::{Choice, MergeFacts, PullState, Rights},
    merge_button::{ActionHandler, ChoiceHandler, MergeButton},
    pr::{Checks, ChecksSummary, PrChipData, ReviewState},
    spinner::Spinner,
    theme::{ActiveTheme, radius},
    typography::{MONO_FONT_FAMILY, TextSize},
};

#[derive(IntoElement)]
pub struct PrCard {
    pub(super) id: ElementId,
    pub(super) pr: PrChipData,
    pub(super) checks: Checks,
    pub(super) review: ReviewState,
    pub(super) on_open: Option<PrHandler>,
    pub(super) merge: Option<(MergeFacts, Choice)>,
    pub(super) on_merge: Option<ActionHandler>,
    on_merge_choice: Option<ChoiceHandler>,
}

impl PrCard {
    pub fn new(id: impl Into<ElementId>, pr: PrChipData) -> Self {
        Self {
            id: id.into(),
            pr,
            checks: Checks::default(),
            review: ReviewState::default(),
            on_open: None,
            merge: None,
            on_merge: None,
            on_merge_choice: None,
        }
    }

    /// The merge facts and the reader's choice. The button shows only on an open pull request the
    /// reader can merge.
    pub fn merge(mut self, facts: MergeFacts, choice: Choice) -> Self {
        self.merge = Some((facts, choice));
        self
    }

    pub fn on_merge(
        mut self,
        f: impl Fn(crate::merge::Action, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_merge = Some(Rc::new(f));
        self
    }

    pub fn on_merge_choice(mut self, f: impl Fn(Choice, &mut Window, &mut App) + 'static) -> Self {
        self.on_merge_choice = Some(Rc::new(f));
        self
    }

    pub fn checks(mut self, checks: Checks) -> Self {
        self.checks = checks;
        self
    }

    pub fn review(mut self, review: ReviewState) -> Self {
        self.review = review;
        self
    }

    pub fn on_open(
        mut self,
        handler: impl Fn(&PrChipData, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open = Some(Rc::new(handler));
        self
    }
}

#[derive(Default)]
pub(crate) struct LinkActions {
    pub copy: CopyFeedback,
}

impl RenderOnce for PrCard {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| LinkActions::default());
        let summary = self.checks.summary();
        let actions = link_actions(&self.id, &self.pr.url, &state, &theme, window, cx);
        let (pr, on_open) = (self.pr.clone(), self.on_open.clone());
        let group: SharedString = format!("pr-card-{}", self.id).into();
        let merge = self
            .merge
            .filter(|(facts, _)| facts.state == PullState::Open && facts.rights != Rights::Cannot)
            .map(|(facts, choice)| {
                let (on_action, on_choice) = (self.on_merge.clone(), self.on_merge_choice.clone());
                let button = MergeButton::new(
                    ElementId::NamedChild(std::sync::Arc::new(self.id.clone()), "merge".into()),
                    facts,
                    choice,
                )
                .on_action(move |action, window, cx| {
                    if let Some(f) = on_action.as_ref() {
                        f(action, window, cx);
                    }
                })
                .on_choice(move |choice, window, cx| {
                    if let Some(f) = on_choice.as_ref() {
                        f(choice, window, cx);
                    }
                });
                // A press on the button is the button's, not the row's.
                div()
                    .flex_none()
                    .ml(px(4.))
                    .on_mouse_down(gpui_kit::MouseButton::Left, |_, _, cx| {
                        cx.stop_propagation()
                    })
                    .child(button)
            });

        div()
            .id(self.id.clone())
            .group(group.clone())
            .flex()
            .items_center()
            .gap(px(10.))
            .h(px(36.))
            .pl(px(12.))
            .pr(px(6.))
            .rounded(radius::lg())
            .bg(theme.card_strong)
            .text_size(TextSize::Xs.font_size())
            .line_height(TextSize::Xs.line_height())
            .when_some(on_open, |d, open| {
                d.cursor_pointer()
                    .press_stop(
                        (self.id.clone(), "card-focus"),
                        crate::theme::radius::md(),
                        window,
                        cx,
                    )
                    .on_click(move |_, window, cx| open(&pr, window, cx))
            })
            .child(
                Icon::new(self.pr.state.icon())
                    .size(px(14.))
                    .color(self.pr.state.color(&theme)),
            )
            .child(
                div()
                    .flex_none()
                    .font_family(MONO_FONT_FAMILY)
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.foreground.opacity(0.9))
                    .child(self.pr.label()),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_color(theme.foreground.opacity(0.85))
                    .child(self.pr.title),
            )
            .when(summary != ChecksSummary::None, |d| {
                d.child(
                    div()
                        .flex()
                        .flex_none()
                        .items_center()
                        .gap(px(4.))
                        .text_color(summary.color(&theme))
                        .when(summary == ChecksSummary::Running, |d| {
                            d.child(
                                Spinner::new(ElementId::NamedChild(
                                    std::sync::Arc::new(self.id.clone()),
                                    "checks".into(),
                                ))
                                .size(px(12.))
                                .color(muted),
                            )
                        })
                        .child(summary.text()),
                )
            })
            .when(self.review != ReviewState::None, |d| {
                d.child(
                    div()
                        .flex_none()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(self.review.color(&theme))
                        .child(self.review.text()),
                )
            })
            .child(
                div()
                    .flex()
                    .flex_none()
                    .opacity(0.)
                    .group_hover(group, |s| s.opacity(1.))
                    .children(actions),
            )
            .children(merge)
    }
}
