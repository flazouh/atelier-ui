use std::sync::Arc;

use gpui_kit::{
    App,
    ElementId,
    InteractiveElement,
    IntoElement,
    ParentElement,
    RenderOnce,
    SharedString,
    StatefulInteractiveElement,
    Styled,
    Window,
    div,
    prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    focus::PressStop,
    icon::{Icon, IconName},
    rail_section::RailSection,
    theme::ActiveTheme,
    typography::{MONO_FONT_FAMILY, TextSize},
};
use super::helpers::how_many;

#[derive(Clone, Debug, PartialEq)]
pub struct CommitData {
    pub sha: SharedString,
    pub title: SharedString,
    pub author: SharedString,
    /// How long ago, as the app words it: "2h ago".
    pub age: SharedString,
    /// When, for finding the newest: larger is newer.
    pub at: u64,
}

/// The commits, as a rail section: one line that opens to the list.
#[derive(IntoElement)]
pub struct CommitsSummary {
    pub(super) id: ElementId,
    pub(super) commits: Vec<CommitData>,
}

impl CommitsSummary {
    pub fn new(id: impl Into<ElementId>, commits: Vec<CommitData>) -> Self {
        Self { id: id.into(), commits }
    }
}

impl RenderOnce for CommitsSummary {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let open = window.use_keyed_state(self.id.clone(), cx, |_, _| false);
        let shown = *open.read(cx);
        let toggle = open.clone();
        let summary = how_many(&self.commits);
        let mut commits = self.commits;
        commits.sort_by_key(|c| std::cmp::Reverse(c.at));
        RailSection::new("Commits")
            .icon(IconName::Commit)
            .summary(summary)
            .when(!commits.is_empty(), |d| {
                d.child(
                    div()
                        .id(ElementId::NamedChild(Arc::new(self.id.clone()), "toggle".into()))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .px(px(12.))
                        .py(px(6.))
                        .cursor_pointer()
                        .text_size(TextSize::Xs.font_size())
                        .text_color(muted)
                        .hover(|s| s.bg(theme.muted_hover()))
                        .press_stop((self.id.clone(), "more-focus"), crate::theme::radius::md(), window, cx)
                        .on_click(move |_, _, cx| toggle.update(cx, |o, cx| {
                            *o = !*o;
                            cx.notify();
                        }))
                        .child(Icon::new(if shown { IconName::ChevronDown } else { IconName::ChevronRight }).size(px(12.)))
                        .child(if shown { "Hide them" } else { "Show them" }),
                )
            })
            .when(shown, |d| {
                d.children(commits.into_iter().map(|c| {
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .px(px(12.))
                        .py(px(4.))
                        .text_size(TextSize::Xs.font_size())
                        .child(div().flex_none().font_family(MONO_FONT_FAMILY).text_size(px(11.)).text_color(muted).child(c.sha.chars().take(7).collect::<String>()))
                        .child(div().flex_1().min_w_0().truncate().text_color(theme.foreground.opacity(0.9)).child(c.title))
                        .child(div().flex_none().text_color(muted).child(c.age))
                }))
            })
            .child(div().h(px(4.)))
    }
}
