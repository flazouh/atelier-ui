//! The head of a file's card in a review: a file icon, the path with its folder muted and its name in
//! ink, `+a −r`, and on the right Accept file (primary) and Reject file, each with its key cap. It sits
//! above the editor and does not scroll with the text, as the file heading does in GitQuiet's pull
//! request screen. Where a breadcrumb over the card already gives the path, [`ReviewFileHeader::path_shown`] takes
//! the path out of the head, leaving the counts and the buttons.
//!
//! In a narrow card the path gives way first: its folder truncates while the name stays whole. Only when
//! even the name would be cut do the two buttons drop the word "file". The header measures both off its
//! own layout, as [`crate::review_bar::ReviewBar`] does.
//!
//! A file the pull request did not change, opened to read beside it, says "Brought in" where the counts
//! go ([`ReviewFileHeader::brought_in`]). Where a language server answers, the header also holds its
//! four lookups (Uses, Names in this file, Go to name, Go to file), each an icon beside its key cap, the
//! word in its tooltip: they show for each handler set in [`ReviewHandlers`].

use std::sync::Arc;

use gpui_kit::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::FluentBuilder, 
};
use crate::scale::px;

use crate::{
    button::{Button, ButtonSize, ButtonVariant},
    changed_files::split_path,
    file_icon::FileIcon,
    icon::IconName,
    keys::{self, Command},
    placement::measure,
    review::{ReviewHandler, ReviewHandlers, caps},
    review_bar::{choose, worded},
    theme::{ActiveTheme, Theme},
    typography::{MONO_FONT_FAMILY, TextSize},
};

const GAP: f32 = 8.;
const PADDING: f32 = 10.;
/// The file icon and its gap.
const ICON: f32 = 14. + GAP;

#[derive(IntoElement)]
pub struct ReviewFileHeader {
    id: ElementId,
    path: SharedString,
    added: usize,
    removed: usize,
    handlers: ReviewHandlers,
    brought_in: bool,
    path_shown: bool,
    committed: Option<SharedString>,
    decided: Option<SharedString>,
}

impl ReviewFileHeader {
    /// Accept file and Reject file run the handlers' `on_accept_file` and `on_reject_file`.
    pub fn new(id: impl Into<ElementId>, path: impl Into<SharedString>, added: usize, removed: usize, handlers: ReviewHandlers) -> Self {
        Self { id: id.into(), path: path.into(), added, removed, handlers, brought_in: false, path_shown: true, committed: None, decided: None }
    }
    /// A file whose decisions are committed: these words, such as "Committed in b89cbbe", stand
    /// where Accept file and Reject file go.
    pub fn committed(mut self, words: Option<SharedString>) -> Self {
        self.committed = words;
        self
    }
    /// A file with nothing left to decide: these words, such as "Accepted", and Undo stand where Accept
    /// file and Reject file go.
    pub fn decided(mut self, words: Option<SharedString>) -> Self {
        self.decided = words;
        self
    }

    /// Whether the head shows the file's icon and path (the default). Off where a breadcrumb gives the path.
    pub fn path_shown(mut self, shown: bool) -> Self {
        self.path_shown = shown;
        self
    }

    /// A file the pull request did not change: "Brought in" stands where the counts go.
    pub fn brought_in(mut self, brought_in: bool) -> Self {
        self.brought_in = brought_in;
        self
    }
}

/// The lookups a language server answers, in the order GitQuiet's keyboard sheet lists them.
const LOOKUPS: [(Command, IconName); 5] = [
    (Command::Uses, IconName::Link),
    (Command::FileNames, IconName::FormatListBulleted),
    (Command::GoToName, IconName::DataObject),
    (Command::GoToFile, IconName::Description),
    // A comment on the line under the caret: the same as the `c` key, and a Tab stop for a reader without one.
    (Command::Comment, IconName::ChatBubble),
];

/// The widths the fit reads off the last frame, and what each of its two steps needs.
struct HeaderState {
    width: f32,
    name: f32,
    counts: f32,
    needed: [Option<f32>; 2],
    measured_for: SharedString,
    /// 0 with "file", 1 without.
    step: usize,
}

fn counts(id: impl Into<gpui_kit::ElementId>, added: usize, removed: usize, theme: &Theme) -> impl IntoElement {
    let id = id.into();
    let count = |text: String, added: bool| {
        div().font_family(MONO_FONT_FAMILY).text_size(TextSize::Xs.font_size()).text_color(theme.diff_color(added)).child(crate::Digits::new((id.clone(), if added { "added" } else { "removed" }), text, TextSize::Xs.font_size()))
    };
    div()
        .relative()
        .flex()
        .flex_none()
        .gap(px(6.))
        .when(added > 0, |d| d.child(count(format!("+{added}"), true)))
        .when(removed > 0, |d| d.child(count(format!("\u{2212}{removed}"), false)))
}

fn button(button: Button, handler: &Option<ReviewHandler>) -> Button {
    let button = button.size(ButtonSize::Sm);
    match handler.clone() {
        Some(f) => button.on_click(move |_, window, cx| f(window, cx)),
        None => button.disabled(true),
    }
}

impl RenderOnce for ReviewFileHeader {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| HeaderState {
            width: f32::MAX,
            name: 0.,
            counts: 0.,
            needed: [None; 2],
            measured_for: SharedString::default(),
            step: 0,
        });
        let path = self.path.clone();
        state.update(cx, |s, _| {
            if s.measured_for != path {
                s.measured_for = path.clone();
                s.needed = [None; 2];
            }
        });
        let with_file = state.read(cx).step == 0;
        let child = |name: &'static str| ElementId::NamedChild(Arc::new(self.id.clone()), name.into());
        let (folder, name) = split_path(&self.path);
        let (folder, name) = (folder.to_string(), name.to_string());

        let measure_header = {
            let state = state.clone();
            measure(move |b, cx| state.update(cx, |s, _| s.width = f32::from(b.size.width)))
        };
        let measure_name = {
            let state = state.clone();
            measure(move |b, cx| state.update(cx, |s, _| s.name = f32::from(b.size.width)))
        };
        let measure_counts = {
            let state = state.clone();
            measure(move |b, cx| state.update(cx, |s, _| s.counts = f32::from(b.size.width)))
        };
        // The buttons come last in layout, so their measure settles what this step needs: the whole
        // name must show, the folder may truncate to nothing.
        let shown = self.path_shown;
        let measure_buttons = {
            let state = state.clone();
            measure(move |b, cx| {
                state.update(cx, |s, cx| {
                    let need = 2. * PADDING + if shown { ICON } else { 0. } + s.name + GAP + s.counts + GAP + f32::from(b.size.width);
                    s.needed[s.step] = Some(need);
                    let next = choose(s.width, &s.needed);
                    if next != s.step {
                        s.step = next;
                        cx.notify();
                    }
                })
            })
        };

        let h = &self.handlers;
        let (accept, reject) = if with_file { ("Accept file", "Reject file") } else { ("Accept", "Reject") };
        div()
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(GAP))
            .h(px(40.))
            .px(px(PADDING))
            .text_size(TextSize::Sm.font_size())
            .line_height(TextSize::Sm.line_height())
            .whitespace_nowrap()
            .child(measure_header)
            .when(shown, |d| {
                d
                .child(FileIcon::file(&self.path))
            .child(
                div()
                    .debug_selector(|| "file-header-path".into())
                    .flex()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .child(div().min_w_0().truncate().text_color(theme.muted_foreground).child(folder))
                    .child(div().relative().flex_none().text_color(theme.foreground.opacity(0.9)).child(measure_name).child(name)),
            )
            })
            .when(!shown, |d| d.child(div().flex_1().min_w_0()))
            .child(div().relative().flex_none().child(measure_counts).child(if self.brought_in {
                div().text_size(TextSize::Xs.font_size()).text_color(theme.muted_foreground).child("Brought in").into_any_element()
            } else {
                counts((self.id.clone(), "counts"), self.added, self.removed, &theme).into_any_element()
            }))
            .child(
                div()
                    .relative()
                    .flex()
                    .flex_none()
                    .gap(px(GAP))
                    .child(measure_buttons)
                    .children(LOOKUPS.iter().filter_map(|&(command, icon)| {
                        let handler = h.for_command(command).cloned();
                        handler.map(|f| {
                            let button = Button::new(child(keys::word(command))).debug_name(keys::word(command)).variant(ButtonVariant::Ghost).command(command);
                            let handler = Some(f);
                            worded(button, keys::word(command), icon, false, &handler).into_any_element()
                        })
                    }))
                    // A file that is read rather than decided, as in a pull request, has no buttons.
                    .when_some(self.committed.clone(), |d, words| {
                        d.child(div().flex().items_center().text_size(TextSize::Xs.font_size()).text_color(theme.muted_foreground).child(words))
                    })
                    .when_some(self.decided.clone().filter(|_| self.committed.is_none()), |d, words| {
                        d.child(div().flex().items_center().text_size(TextSize::Xs.font_size()).text_color(theme.muted_foreground).child(words))
                            .child(button(Button::new(child("undo")).label("Undo decision").variant(ButtonVariant::Ghost).command(Command::UndoDecision), &h.on_undo_decision))
                    })
                    .when(self.committed.is_none() && self.decided.is_none() && (h.on_accept_file.is_some() || h.on_reject_file.is_some()), |d| {
                        d.child(button(Button::new(child("accept")).label(accept).variant(ButtonVariant::Primary).cap(caps::ACCEPT_FILE), &h.on_accept_file))
                            .child(button(Button::new(child("reject")).label(reject).variant(ButtonVariant::Secondary).cap(caps::REJECT_FILE), &h.on_reject_file))
                    }),
            )
    }
}

#[cfg(test)]
mod tests;
