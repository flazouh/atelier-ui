use std::{rc::Rc, time::Duration};

use gpui_kit::{
    App, ElementId, Entity, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, AppContext, component::input::InputState, div, prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    animated_badge::{AnimatedBadge, BadgeSize, BadgeStatus},
    button::{Button, ButtonSize, ButtonVariant},
    icon::{Icon, IconName},
    text_input::TextInput,
    theme::{ActiveTheme, Theme, radius},
    typography::TextSize,
};

use super::{
    helpers::{answer_text, is_answered, pressed},
    types::{QuestionStatus, QuestionView},
};

/// How long a single choice waits before the card moves on to the next question, as beui's does.
const ADVANCE: Duration = Duration::from_millis(240);
const CUSTOM_PLACEHOLDER: &str = "Add another response…";

/// What the reader answered, a question's text and the words that answer it, in the order the questions came.
pub type SubmitHandler = Rc<dyn Fn(Vec<(SharedString, SharedString)>, &mut Window, &mut App)>;

/// What pressing a choice does.
type PressHandler = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct QuestionCard {
    id: ElementId,
    questions: Vec<QuestionView>,
    status: QuestionStatus,
    answers: Vec<(SharedString, SharedString)>,
    on_submit: Option<SubmitHandler>,
}

impl QuestionCard {
    /// `id` must be unique among the cards on screen: the card keeps the reader's picks under it.
    pub fn new(id: impl Into<ElementId>, questions: Vec<QuestionView>) -> Self {
        Self { id: id.into(), questions, status: QuestionStatus::Pending, answers: Vec::new(), on_submit: None }
    }

    pub fn status(mut self, status: QuestionStatus) -> Self {
        self.status = status;
        self
    }

    /// What the reader answered, shown once the card is [`QuestionStatus::Answered`].
    pub fn answers(mut self, answers: Vec<(SharedString, SharedString)>) -> Self {
        self.answers = answers;
        self
    }

    /// Called with the answers when the reader presses Submit at the last question.
    pub fn on_submit(mut self, f: impl Fn(Vec<(SharedString, SharedString)>, &mut Window, &mut App) + 'static) -> Self {
        self.on_submit = Some(Rc::new(f));
        self
    }
}

/// The reader's picks, kept under the card's id while it waits.
struct CardState {
    step: usize,
    picks: Vec<Vec<usize>>,
    inputs: Vec<Entity<InputState>>,
}

fn child(id: &ElementId, name: impl Into<SharedString>) -> ElementId {
    ElementId::from((id.clone(), name.into()))
}

impl RenderOnce for QuestionCard {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let total = self.questions.len();
        let id = self.id.clone();
        let status = self.status;

        // The picks live only while the card waits; the questions are known whole by then.
        let state = (status == QuestionStatus::Pending && total > 0).then(|| {
            let state = window.use_keyed_state(child(&id, "state"), cx, |_, _| CardState { step: 0, picks: Vec::new(), inputs: Vec::new() });
            let missing = total.saturating_sub(state.read(cx).inputs.len());
            if missing > 0 {
                state.update(cx, |s, cx| {
                    for _ in 0..missing {
                        s.picks.push(Vec::new());
                        s.inputs.push(cx.new(|cx| InputState::new(window, cx).placeholder(CUSTOM_PLACEHOLDER)));
                    }
                });
            }
            state
        });
        let step = state.as_ref().map_or(0, |s| s.read(cx).step.min(total.saturating_sub(1)));

        let (badge, badge_label) = match status {
            QuestionStatus::Streaming => (BadgeStatus::Loading, "Asking"),
            QuestionStatus::Pending => (BadgeStatus::Warning, "Your answer"),
            QuestionStatus::Answered => (BadgeStatus::Success, "Answered"),
        };
        let title: SharedString = match status {
            QuestionStatus::Streaming => "Asking a question…".into(),
            QuestionStatus::Pending => self
                .questions
                .get(step)
                .map(|q| q.header.clone())
                .filter(|h| !h.is_empty())
                .unwrap_or_else(|| "Question".into()),
            QuestionStatus::Answered => "Question answered".into(),
        };
        let head = div()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(12.))
            .px(px(12.))
            .py(px(8.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .min_w_0()
                    .child(Icon::new(IconName::ChatBubble).size(px(16.)).color(muted))
                    .child(
                        div()
                            .truncate()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(if status == QuestionStatus::Streaming { muted } else { theme.foreground })
                            .child(title),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .flex_none()
                    .when(status == QuestionStatus::Pending && total > 1, |d| {
                        d.child(div().text_size(TextSize::Xs.font_size()).text_color(muted).child(format!("{}/{}", step + 1, total)))
                    })
                    .child(AnimatedBadge::new(child(&id, "status"), badge).size(BadgeSize::Small).label(badge_label)),
            );

        let body = match status {
            QuestionStatus::Streaming => div()
                .flex()
                .flex_col()
                .gap(px(10.))
                .px(px(12.))
                .pb(px(10.))
                .children(self.questions.iter().enumerate().map(|(q, question)| {
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.))
                        .child(div().line_height(px(20.)).font_weight(FontWeight::MEDIUM).text_color(theme.foreground).child(question.question.clone()))
                        .children(question.options.iter().enumerate().map(|(i, (label, description))| {
                            option_row(child(&id, format!("stream-{q}-{i}")), &theme, label, description, question.multiple, false, None)
                        }))
                }))
                .into_any_element(),
            QuestionStatus::Answered => div()
                .flex()
                .flex_col()
                .gap(px(8.))
                .px(px(12.))
                .pb(px(10.))
                .children(self.answers.iter().map(|(question, answer)| {
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(1.))
                        .child(div().text_size(TextSize::Xs.font_size()).text_color(muted).child(question.clone()))
                        .when(!answer.is_empty(), |d| {
                            d.child(div().line_height(px(20.)).font_weight(FontWeight::MEDIUM).text_color(theme.foreground).child(answer.clone()))
                        })
                }))
                .into_any_element(),
            QuestionStatus::Pending => match (&state, self.questions.get(step)) {
                (Some(state), Some(question)) => pending_body(&id, &theme, state, step, total, question, &self.questions, self.on_submit.clone(), cx),
                _ => div().into_any_element(),
            },
        };

        div()
            .debug_selector(|| "question-card".into())
            .flex()
            .flex_col()
            .w_full()
            .overflow_hidden()
            .rounded(radius::card())
            .bg(theme.card_strong)
            .text_size(TextSize::Sm.font_size())
            .child(head)
            .child(body)
    }
}

/// The question the card is on: its text, the choices, the line for the reader's own words, and the way on.
#[allow(clippy::too_many_arguments)]
fn pending_body(
    id: &ElementId,
    theme: &Theme,
    state: &Entity<CardState>,
    step: usize,
    total: usize,
    question: &QuestionView,
    all: &[QuestionView],
    on_submit: Option<SubmitHandler>,
    cx: &mut App,
) -> gpui_kit::AnyElement {
    let (picked, input) = {
        let s = state.read(cx);
        (s.picks.get(step).cloned().unwrap_or_default(), s.inputs.get(step).cloned())
    };
    let custom = input.as_ref().map(|i| i.read(cx).value().to_string()).unwrap_or_default();
    let multiple = question.multiple;
    let rows = question.options.iter().enumerate().map(|(i, (label, description))| {
        let (press_state, press_input) = (state.clone(), input.clone());
        let handler: PressHandler = Rc::new(move |window, cx| {
            press_state.update(cx, |s, cx| {
                let next = pressed(&s.picks[step], i, multiple);
                s.picks[step] = next;
                cx.notify();
            });
            if multiple {
                return;
            }
            // Picking a choice lets go of words written beside it, and a single choice moves on once it has shown.
            if let Some(input) = &press_input {
                input.update(cx, |input, cx| input.set_value("", window, cx));
            }
            let picked_now = !press_state.read(cx).picks[step].is_empty();
            if picked_now && step + 1 < total {
                let advance = press_state.clone();
                cx.spawn(async move |cx| {
                    cx.background_executor().timer(ADVANCE).await;
                    advance
                        .update(cx, |s, cx| {
                            if s.step == step {
                                s.step = step + 1;
                                cx.notify();
                            }
                        });
                })
                .detach();
            }
        });
        option_row(child(id, format!("opt-{step}-{i}")), theme, label, description, multiple, picked.contains(&i), Some(handler))
    });

    let last = step + 1 >= total;
    let can = is_answered(&picked, &custom);
    let back = (step > 0).then(|| {
        let back = state.clone();
        Button::new(child(id, "back")).label("Back").variant(ButtonVariant::Ghost).size(ButtonSize::Sm).on_click(move |_, _, cx| {
            back.update(cx, |s, cx| {
                s.step = step.saturating_sub(1);
                cx.notify();
            })
        })
    });
    let dots = (total > 1).then(|| {
        div().flex().items_center().justify_center().gap(px(6.)).children((0..total).map(|d| {
            div().size(px(6.)).rounded_full().bg(if d == step { theme.foreground } else { theme.muted_foreground.opacity(0.35) })
        }))
    });
    let forward = {
        let (go, questions) = (state.clone(), all.to_vec());
        Button::new(child(id, if last { "submit" } else { "next" }))
            .label(if last { "Submit" } else { "Next" })
            .variant(ButtonVariant::Invert)
            .size(ButtonSize::Sm)
            .disabled(!can)
            .on_click(move |_, window, cx| {
                if !last {
                    go.update(cx, |s, cx| {
                        s.step = step + 1;
                        cx.notify();
                    });
                    return;
                }
                let (picks, inputs) = {
                    let s = go.read(cx);
                    (s.picks.clone(), s.inputs.clone())
                };
                let answers = questions
                    .iter()
                    .enumerate()
                    .map(|(q, question)| {
                        let words = inputs.get(q).map(|i| i.read(cx).value().to_string()).unwrap_or_default();
                        (question.question.clone(), SharedString::from(answer_text(question, picks.get(q).map(Vec::as_slice).unwrap_or_default(), &words)))
                    })
                    .collect();
                if let Some(submit) = &on_submit {
                    submit(answers, window, cx);
                }
            })
    };

    div()
        .flex()
        .flex_col()
        .child(
            div()
                .px(px(12.))
                .pb(px(6.))
                .line_height(px(20.))
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.foreground)
                .child(question.question.clone()),
        )
        .child(div().flex().flex_col().gap(px(2.)).px(px(6.)).children(rows))
        .children(input.map(|input| {
            div().px(px(12.)).pt(px(6.)).child(TextInput::new(child(id, format!("custom-{step}")), &input).surface(theme.background))
        }))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap(px(8.))
                .px(px(12.))
                .pt(px(10.))
                .pb(px(10.))
                .child(div().min_w(px(56.)).children(back))
                .children(dots)
                .child(div().flex().justify_end().min_w(px(56.)).child(forward)),
        )
        .into_any_element()
}

/// One choice: a mark (a circle for one answer, a square for several), its label, and what it means. A row with no handler is
/// shown while the question still streams, and cannot be pressed.
fn option_row(
    id: ElementId,
    theme: &Theme,
    label: &SharedString,
    description: &SharedString,
    multiple: bool,
    selected: bool,
    on_press: Option<PressHandler>,
) -> impl IntoElement {
    let ink = theme.foreground;
    let line = theme.muted_foreground;
    let mark = div()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .mt(px(2.))
        .size(px(16.))
        .border_1()
        .border_color(if selected { ink } else { line.opacity(0.7) })
        .when(multiple, |d| d.rounded(radius::md()).when(selected, |d| d.bg(ink)))
        .when(!multiple, |d| d.rounded_full())
        .when(selected && multiple, |d| d.child(Icon::new(IconName::Check).size(px(12.)).color(theme.card_strong)))
        .when(selected && !multiple, |d| d.child(div().size(px(8.)).rounded_full().bg(ink)));
    let words = div()
        .flex()
        .flex_col()
        .min_w_0()
        .child(div().line_height(px(20.)).text_color(ink).child(label.clone()))
        .when(!description.is_empty(), |d| {
            d.child(div().text_size(TextSize::Xs.font_size()).line_height(px(16.)).text_color(line).child(description.clone()))
        });
    let interactive = on_press.is_some();
    div()
        .id(id)
        .flex()
        .items_start()
        .gap(px(10.))
        .px(px(6.))
        .py(px(6.))
        .rounded(radius::lg())
        .when(!interactive, |d| d.opacity(0.7))
        .when(interactive, |d| d.cursor_pointer().hover(|s| s.bg(theme.wash())))
        .when_some(on_press, |d, press| d.on_click(move |_, window, cx| press(window, cx)))
        .child(mark)
        .child(words)
}
