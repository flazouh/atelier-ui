use std::{collections::HashSet, rc::Rc, sync::Arc};

use gpui_kit::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
};

use super::helpers::{checks_text, courts, initial};
use super::types::{Court, OpenHandler, TITLE_LEAST};
use crate::scale::px;
use crate::{
    focus::PressStop,
    icon::{Icon, IconName},
    pr::{Checks, ChecksSummary, PrChipData, ReviewState},
    spinner::Spinner,
    theme::{ActiveTheme, radius},
    typography::{MONO_FONT_FAMILY, TextSize},
};

/// One pull request in the working set.
#[derive(Clone, Debug, PartialEq)]
pub struct CourtItem {
    pub pr: PrChipData,
    pub author: SharedString,
    pub court: Court,
    /// Why it sits in its Court, in a few words: "Review asked of you", "Ready to merge".
    pub why: SharedString,
    pub checks: Checks,
    pub review: ReviewState,
    pub comments: usize,
    pub added: usize,
    pub removed: usize,
    /// How long since it changed, as the app words it: "2h ago".
    pub age: SharedString,
    /// When it changed, for ordering: larger is newer.
    pub changed_at: u64,
    /// The reader has not read it since it changed.
    pub unread: bool,
}

/// title (in weight while unread), its repository, why it sits there, its review and checks, how much was
/// said, its size, and its age. Each Court heads its rows with its name and count, and folds.
#[derive(IntoElement)]
pub struct CourtList {
    id: ElementId,
    pub(super) items: Vec<CourtItem>,
    on_open: Option<OpenHandler>,
}

impl CourtList {
    pub fn new(id: impl Into<ElementId>, items: Vec<CourtItem>) -> Self {
        Self {
            id: id.into(),
            items,
            on_open: None,
        }
    }

    pub fn on_open(mut self, f: impl Fn(&PrChipData, &mut Window, &mut App) + 'static) -> Self {
        self.on_open = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for CourtList {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let folded = window.use_keyed_state(self.id.clone(), cx, |_, _| HashSet::<u8>::new());
        let folded_now = folded.read(cx).clone();
        let child = |name: String| ElementId::NamedChild(Arc::new(self.id.clone()), name.into());
        let on_open = self.on_open.clone();

        let row = |item: CourtItem, window: &mut Window, cx: &mut App| {
            let open = on_open.clone();
            let pr = item.pr.clone();
            let checks = item.checks.summary();
            let checks_el = match checks {
                ChecksSummary::None => div().into_any_element(),
                ChecksSummary::Running => div()
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .text_color(theme.warning)
                    .child(
                        Spinner::new(child(format!("spin-{}-{}", item.pr.repo, item.pr.number)))
                            .size(px(12.))
                            .color(theme.warning),
                    )
                    .children(checks_text(item.checks))
                    .into_any_element(),
                ChecksSummary::Failing(_) => div()
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .text_color(theme.danger)
                    .child(Icon::new(IconName::Error).size(px(12.)))
                    .children(checks_text(item.checks))
                    .into_any_element(),
                ChecksSummary::Passed(_) => Icon::new(IconName::CheckCircle)
                    .size(px(12.))
                    .color(theme.success)
                    .into_any_element(),
            };
            div()
                .id(child(format!("row-{}-{}", item.pr.repo, item.pr.number)))
                .flex()
                .items_center()
                .gap(px(10.))
                .h(px(30.))
                .px(px(12.))
                .rounded(radius::md())
                .cursor_pointer()
                .text_size(TextSize::Xs.font_size())
                .hover(|s| s.bg(theme.muted_hover()))
                .when_some(open, |d, open| {
                    d.press_stop(
                        gpui_kit::ElementId::Name(
                            format!("row-focus-{}-{}", item.pr.repo, item.pr.number).into(),
                        ),
                        crate::theme::radius::md(),
                        window,
                        cx,
                    )
                    .on_click(move |_, window, cx| open(&pr, window, cx))
                })
                .child(initial(&item.author, &theme))
                .child(
                    Icon::new(item.pr.state.icon())
                        .size(px(14.))
                        .color(item.pr.state.color(&theme)),
                )
                .child(
                    div()
                        .flex_none()
                        .w(px(58.))
                        .font_family(MONO_FONT_FAMILY)
                        .text_size(px(11.))
                        .text_color(muted)
                        .child(item.pr.label()),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w(px(TITLE_LEAST))
                        .debug_selector(|| "court-title".into())
                        .truncate()
                        .text_color(theme.foreground.opacity(0.9))
                        .when(item.unread, |d| d.font_weight(FontWeight::SEMIBOLD))
                        .child(item.pr.title.clone()),
                )
                .child(
                    div()
                        .min_w_0()
                        .w(px(120.))
                        .truncate()
                        .text_color(muted)
                        .child(item.pr.repo.clone()),
                )
                .child(
                    div()
                        .min_w_0()
                        .w(px(132.))
                        .truncate()
                        .text_color(muted)
                        .child(item.why.clone()),
                )
                .child(
                    div()
                        .min_w_0()
                        .w(px(104.))
                        .truncate()
                        .text_color(item.review.color(&theme))
                        .child(item.review.text()),
                )
                .child(div().min_w_0().w(px(72.)).child(checks_el))
                .child(
                    div()
                        .flex()
                        .min_w_0()
                        .w(px(40.))
                        .items_center()
                        .gap(px(4.))
                        .text_color(muted)
                        .when(item.comments > 0, |d| {
                            d.child(Icon::new(IconName::ChatBubble).size(px(12.)))
                                .child(item.comments.to_string())
                        }),
                )
                .child(
                    div()
                        .flex()
                        .min_w_0()
                        .w(px(88.))
                        .justify_end()
                        .gap(px(4.))
                        .font_family(MONO_FONT_FAMILY)
                        .text_size(px(11.))
                        .child(
                            div()
                                .text_color(theme.diff_color(true))
                                .child(format!("+{}", item.added)),
                        )
                        .child(
                            div()
                                .text_color(theme.diff_color(false))
                                .child(format!("\u{2212}{}", item.removed)),
                        ),
                )
                .child(
                    div()
                        .flex_none()
                        .w(px(56.))
                        .text_color(muted)
                        .text_right()
                        .child(item.age.clone()),
                )
        };

        div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .children(courts(self.items).into_iter().map(|(court, rows)| {
                let key = court as u8;
                let is_folded = folded_now.contains(&key);
                let toggle = folded.clone();
                let color = if court == Court::NeedsYou {
                    theme.warning
                } else {
                    theme.foreground.opacity(0.9)
                };
                let count = rows.len();
                div()
                    .flex()
                    .flex_col()
                    .p(px(4.))
                    .rounded(radius::lg())
                    .bg(theme.card)
                    .child(
                        div()
                            .id(child(format!("court-{key}")))
                            .press_stop(
                                child(format!("court-focus-{key}")),
                                crate::theme::radius::md(),
                                window,
                                cx,
                            )
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .h(px(30.))
                            .px(px(8.))
                            .rounded(radius::md())
                            .cursor_pointer()
                            .text_size(TextSize::Xs.font_size())
                            .hover(|s| s.bg(theme.muted_hover()))
                            .on_click(move |_, _, cx| {
                                toggle.update(cx, |set, cx| {
                                    if !set.remove(&key) {
                                        set.insert(key);
                                    }
                                    cx.notify();
                                })
                            })
                            .child(
                                Icon::new(if is_folded {
                                    IconName::ChevronRight
                                } else {
                                    court.icon()
                                })
                                .size(px(14.))
                                .color(color),
                            )
                            .child(
                                div()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(color)
                                    .child(court.name()),
                            )
                            .child(div().text_color(muted).child(count.to_string())),
                    )
                    .when(!is_folded, |d| {
                        d.children(rows.into_iter().map(|item| row(item, window, cx)))
                    })
            }))
    }
}
