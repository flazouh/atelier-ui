use gpui_kit::{
    AppContext,
    Context,
    Entity,
    EventEmitter,
    Focusable,
    FontWeight,
    IntoElement,
    ParentElement,
    Render,
    SharedString,
    Styled,
    Window,
    component::input::{Input, InputState, Textarea, TextareaState},
    div,
    prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    button::{Button, ButtonSize, ButtonVariant},
    focus::Field,
    icon::{Icon, IconName},
    merge::{Action, Blocker, Choice, MergeFacts, MergeMethod, PullState, blockers, button, standing},
    merge_button::MergeButton,
    theme::{ActiveTheme, radius},
    typography::{MONO_FONT_FAMILY, TextSize},
};
use super::types::MergeBoxEvent;
use super::helpers::mark;

pub struct MergeBox {
    pub(super) facts: MergeFacts,
    pub(super) choice: Choice,
    pub(super) title: Entity<InputState>,
    pub(super) message: Entity<TextareaState>,
    /// After a merge: whether the branch went too.
    pub(super) branch_deleted: bool,
}

impl EventEmitter<MergeBoxEvent> for MergeBox {}

impl MergeBox {
    /// For a pull request titled `title` with `body`, which fill the squash commit.
    pub fn new(
        facts: MergeFacts,
        choice: Choice,
        title: impl Into<SharedString>,
        body: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let (title, body): (SharedString, SharedString) = (title.into(), body.into());
        let title = cx.new(|cx| {
            let mut state = InputState::new(window, cx).placeholder("Commit title");
            state.set_value(title, window, cx);
            state
        });
        let message = cx.new(|cx| {
            let mut state = TextareaState::new(window, cx).auto_grow(2, 8).placeholder("Commit message");
            state.set_value(body, window, cx);
            state
        });
        Self { facts, choice, title, message, branch_deleted: false }
    }

    /// New facts from the forge, after a press or a refresh.
    pub fn set_facts(&mut self, facts: MergeFacts, cx: &mut Context<Self>) {
        self.facts = facts;
        cx.notify();
    }

    /// The pull request merged; `branch_deleted` says whether its branch went with it.
    pub fn merged(&mut self, branch_deleted: bool, cx: &mut Context<Self>) {
        self.facts.state = PullState::Merged;
        self.branch_deleted = branch_deleted;
        cx.notify();
    }

    pub(super) fn act(&mut self, action: Action, cx: &mut Context<Self>) {
        let (title, message) = (self.title.read(cx).value(), self.message.read(cx).value());
        cx.emit(MergeBoxEvent::Act { action, choice: self.choice, title, message });
    }
}

impl Render for MergeBox {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let facts = &self.facts;
        let ready = button(facts, &self.choice).is_some_and(|b| b.ready);
        let header = div()
            .flex()
            .items_center()
            .gap(px(8.))
            .child(Icon::new(IconName::PrMerged).size(px(14.)).color(if ready { theme.success } else { muted }))
            .child(div().text_size(TextSize::Sm.font_size()).font_weight(FontWeight::MEDIUM).child("Merge"))
            .child(div().min_w_0().truncate().text_size(TextSize::Xs.font_size()).text_color(muted).child(standing(facts)));

        let open = facts.state == PullState::Open;
        let reasons = if open { blockers(facts) } else { Vec::new() };
        let rows = reasons.into_iter().map(|blocker| {
            let (icon, tone) = mark(&blocker, &theme);
            let files = match &blocker {
                Blocker::Conflicts(files) => files.clone(),
                _ => Vec::new(),
            };
            div()
                .flex()
                .items_start()
                .gap(px(8.))
                .child(div().pt(px(2.)).flex_none().child(Icon::new(icon).size(px(14.)).color(tone)))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .min_w_0()
                        .gap(px(2.))
                        .child(div().text_size(TextSize::Xs.font_size()).font_weight(FontWeight::MEDIUM).child(blocker.name()))
                        .child(div().text_size(TextSize::Xs.font_size()).text_color(muted).child(blocker.explanation()))
                        .children(files.into_iter().map(|file| {
                            div().truncate().font_family(MONO_FONT_FAMILY).text_size(TextSize::Xs.font_size()).text_color(muted).child(file)
                        })),
                )
        });

        let squash = open && self.choice.method == MergeMethod::Squash && !facts.draft;
        let commit = squash.then(|| {
            div()
                .flex()
                .flex_col()
                .gap(px(6.))
                .child(div().text_size(TextSize::Xs.font_size()).text_color(muted).child("The squash commit"))
                .child(Field::new(self.title.focus_handle(cx), 
                    Input::new(&self.title)
                        .appearance(false)
                        .px(px(10.))
                        .text_size(TextSize::Sm.font_size())
                        .font_weight(FontWeight::MEDIUM),
                ).radius(radius::md()).surface(theme.card_strong))
                .child(Field::new(self.message.focus_handle(cx), 
                    // The textarea keeps 10px of its own inside, so this lines its text up with the title's.
                    Textarea::new(&self.message)
                        .appearance(false)
                        .px(px(0.))
                        .py(px(6.))
                        .text_size(TextSize::Xs.font_size())
                        .line_height(TextSize::Xs.line_height()),
                ).radius(radius::md()).surface(theme.card_strong))
        });

        let this = cx.entity().downgrade();
        let press = this.clone();
        let merge_button = open.then(|| {
            MergeButton::new("merge-box-button", facts.clone(), self.choice)
                .on_action(move |action, _, cx| {
                    press.update(cx, |b, cx| b.act(action, cx)).ok();
                })
                .on_choice(move |choice, _, cx| {
                    this.update(cx, |b, cx| {
                        b.choice = choice;
                        cx.emit(MergeBoxEvent::Chose(choice));
                        cx.notify();
                    })
                    .ok();
                })
        });

        let merged = facts.state == PullState::Merged;
        let result = merged.then(|| {
            let this = cx.entity().downgrade();
            let (delete, revert) = (this.clone(), this);
            div()
                .flex()
                .items_center()
                .gap(px(8.))
                .flex_wrap()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .text_size(TextSize::Xs.font_size())
                        .child(Icon::new(IconName::CheckCircle).size(px(14.)).color(theme.success))
                        .child(if self.branch_deleted { "Merged, and the branch deleted" } else { "Merged" }),
                )
                .child(div().flex_1())
                .when(!self.branch_deleted, |d| {
                    d.child(
                        Button::new("merge-box-delete-branch")
                            .label(Action::DeleteBranch.word())
                            .variant(ButtonVariant::Secondary)
                            .size(ButtonSize::Sm)
                            .on_click(move |_, _, cx| {
                                delete.update(cx, |b, cx| b.act(Action::DeleteBranch, cx)).ok();
                            }),
                    )
                })
                .child(
                    Button::new("merge-box-revert")
                        .label(Action::Revert.word())
                        .variant(ButtonVariant::Secondary)
                        .size(ButtonSize::Sm)
                        .on_click(move |_, _, cx| {
                            revert.update(cx, |b, cx| b.act(Action::Revert, cx)).ok();
                        }),
                )
        });

        div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .p(px(12.))
            .rounded(radius::lg())
            .bg(theme.card)
            .child(header)
            .children(rows)
            .children(commit)
            .children(merge_button.map(|b| div().flex().justify_end().child(b)))
            .children(result)
    }
}
