use std::{collections::HashSet, rc::Rc, sync::Arc};

use gpui_kit::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    focus::PressStop,
    icon::{Icon, IconName},
    line_comment::{Comment, LineComment},
    rail_section::RailSection,
    theme::ActiveTheme,
    typography::TextSize,
};
use super::types::{MoreHandler, ThreadHandler};
use super::helpers::{faces_el, open_first, said_by_count, said_so_far};

/// One review thread, as the list shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct ThreadSummary {
    /// Everyone who spoke in it, first speaker first.
    pub people: Vec<SharedString>,
    pub comments: Vec<Comment>,
    /// Its first words, on one line.
    pub first: SharedString,
    pub resolved: bool,
}

/// Something said about the pull request as a whole.
#[derive(Clone, Debug, PartialEq)]
pub struct RemarkSummary {
    pub author: SharedString,
    pub comment: Comment,
    pub first: SharedString,
}

/// The whole conversation, as a rail section.
#[derive(IntoElement)]
pub struct ConversationList {
    id: ElementId,
    pub(super) threads: Vec<ThreadSummary>,
    pub(super) remarks: Vec<RemarkSummary>,
    on_open: Option<ThreadHandler>,
    on_reply: Option<ThreadHandler>,
    on_resolve: Option<ThreadHandler>,
    /// Open, resolved and remark counts of the whole conversation, when the list holds only a page of it.
    totals: Option<(usize, usize, usize)>,
    more_threads: Option<(usize, MoreHandler)>,
    more_remarks: Option<(usize, MoreHandler)>,
}

impl ConversationList {
    pub fn new(id: impl Into<ElementId>, threads: Vec<ThreadSummary>, remarks: Vec<RemarkSummary>) -> Self {
        Self { id: id.into(), threads, remarks, on_open: None, on_reply: None, on_resolve: None, totals: None, more_threads: None, more_remarks: None }
    }

    /// A press on a thread's line, to take the reader to where it hangs in the code.
    pub fn on_open(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_open = Some(Rc::new(f));
        self
    }

    /// Reply and Resolve in an opened thread. Without them the two buttons are off.
    pub fn on_reply(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_reply = Some(Rc::new(f));
        self
    }

    pub fn on_resolve(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_resolve = Some(Rc::new(f));
        self
    }

    /// For a page of a long conversation: the counts of all of it, for the header. The threads given must
    /// already be in list order (open first), and the first ones.
    pub fn totals(mut self, open: usize, resolved: usize, remarks: usize) -> Self {
        self.totals = Some((open, resolved, remarks));
        self
    }

    /// `hidden` threads are not in the list; a press on the last row asks for more.
    pub fn more_threads(mut self, hidden: usize, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.more_threads = (hidden > 0).then(|| (hidden, Rc::new(f) as MoreHandler));
        self
    }

    pub fn more_remarks(mut self, hidden: usize, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.more_remarks = (hidden > 0).then(|| (hidden, Rc::new(f) as MoreHandler));
        self
    }
}

impl RenderOnce for ConversationList {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let opened = window.use_keyed_state(self.id.clone(), cx, |_, _| HashSet::<usize>::new());
        let open_set = opened.read(cx).clone();
        let header = match self.totals {
            Some((open, resolved, remarks)) => said_by_count(open, resolved, remarks),
            None => said_so_far(&self.threads, self.remarks.len()),
        };
        let child = |name: String| ElementId::NamedChild(Arc::new(self.id.clone()), name.into());
        let threads = open_first(self.threads);
        let remarks = self.remarks;

        // One line: the fold mark (a tick once resolved), faces, first words, and a count.
        let open_thread = self.on_open.clone();
        let line = |key: usize, mark: IconName, mark_color, people: &[SharedString], first: SharedString, count: Option<usize>, receded: bool, window: &mut Window, cx: &mut App| {
            let toggle = opened.clone();
            let open_thread = open_thread.clone().filter(|_| key < 10_000);
            div()
                .id(child(format!("row-{key}")))
                .press_stop(child(format!("row-focus-{key}")), crate::theme::radius::md(), window, cx)
                .flex()
                .items_center()
                .gap(px(8.))
                .px(px(12.))
                .py(px(6.))
                .cursor_pointer()
                .hover(|s| s.bg(theme.muted_hover()))
                .on_click(move |_, window, cx| {
                    toggle.update(cx, |set, cx| {
                        if !set.remove(&key) {
                            set.insert(key);
                        }
                        cx.notify();
                    });
                    if let Some(open) = &open_thread {
                        open(key, window, cx);
                    }
                })
                .child(Icon::new(mark).size(px(12.)).color(mark_color))
                .child(
                    div()
                        .flex()
                        .flex_1()
                        .min_w_0()
                        .items_center()
                        .gap(px(8.))
                        .when(receded, |d| d.opacity(0.6))
                        .child(faces_el(people, &theme))
                        .child(div().flex_1().min_w_0().truncate().text_size(TextSize::Xs.font_size()).text_color(muted).child(first))
                        .when_some(count, |d, n| d.child(div().flex_none().text_size(TextSize::Xs.font_size()).text_color(muted).child(n.to_string()))),
                )
        };
        let mut rows: Vec<gpui_kit::AnyElement> = Vec::new();
        for (i, t) in threads.into_iter().enumerate() {
            let open = open_set.contains(&i);
            let mark = if t.resolved { IconName::Check } else if open { IconName::ChevronDown } else { IconName::ChevronRight };
            let color = if t.resolved { theme.success } else { muted };
            rows.push(line(i, mark, color, &t.people, t.first.clone(), Some(t.comments.len()), t.resolved, window, cx).into_any_element());
            if open {
                let mut comment = LineComment::new(child(format!("thread-{i}")), t.comments).resolved(t.resolved);
                if let Some(reply) = self.on_reply.clone() {
                    comment = comment.on_reply(move |_, window, cx| reply(i, window, cx));
                }
                if let Some(resolve) = self.on_resolve.clone() {
                    comment = comment.on_resolve(move |_, window, cx| resolve(i, window, cx));
                }
                rows.push(comment.into_any_element());
            }
        }
        let more_row = |name: &str, hidden: usize, what: &str, press: MoreHandler, window: &mut Window, cx: &mut App| {
            div().id(child(name.to_string())).flex().px(px(12.)).py(px(6.)).cursor_pointer().hover(|s| s.bg(theme.muted_hover())).press_stop(child(format!("{name}-focus")), crate::theme::radius::md(), window, cx).on_click(move |_, window, cx| press(window, cx)).child(
                div().text_size(TextSize::Xs.font_size()).text_color(muted).child(format!("Show more {what} ({hidden} not shown)")),
            )
        };
        if let Some((hidden, press)) = self.more_threads.clone() {
            rows.push(more_row("more-threads", hidden, "threads", press, window, cx).into_any_element());
        }
        for (j, r) in remarks.into_iter().enumerate() {
            let key = 10_000 + j;
            let open = open_set.contains(&key);
            let mark = if open { IconName::ChevronDown } else { IconName::ChevronRight };
            rows.push(line(key, mark, muted, std::slice::from_ref(&r.author), r.first.clone(), None, false, window, cx).into_any_element());
            if open {
                rows.push(LineComment::new(child(format!("remark-{j}")), vec![r.comment]).into_any_element());
            }
        }
        if let Some((hidden, press)) = self.more_remarks.clone() {
            rows.push(more_row("more-remarks", hidden, "remarks", press, window, cx).into_any_element());
        }
        RailSection::new("Conversation").icon(IconName::Forum).summary(header).children(rows).child(div().h(px(4.)))
    }
}
