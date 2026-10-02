//! The Git view's sidebar: the repository and the branch of the session in focus, and the files it changed, each in
//! the colour of its change as JetBrains shows them: blue for a change, green for a new file, red for a deleted one.
//! Each row has its letter, its icon, its name, its folder and its `+a −r`. Pressing a file reports it; the file in
//! the review has the accent's wash.
use std::rc::Rc;

use gpui_kit::{
    App, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
};

use crate::{
    changed_files::{ChangedFile, FileChange, split_path},
    file_icon::FileIcon,
    icon::{Icon, IconName},
    scale::px,
    theme::{ActiveTheme, Theme, radius},
    typography::{MONO_FONT_FAMILY, TextSize},
};

const ROW_HEIGHT: f32 = 26.;

/// The colour of a file's name and letter.
pub fn tone(change: &FileChange, theme: &Theme) -> Hsla {
    match change {
        FileChange::Modified | FileChange::Renamed { .. } => theme.info,
        FileChange::Added => theme.success,
        FileChange::Deleted => theme.danger,
    }
}

/// The letter before a file, as `git status --short` writes it.
pub fn letter(change: &FileChange) -> &'static str {
    match change {
        FileChange::Modified => "M",
        FileChange::Added => "A",
        FileChange::Deleted => "D",
        FileChange::Renamed { .. } => "R",
    }
}

type OnOpen = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct GitPanel {
    id: ElementId,
    repo: SharedString,
    branch: Option<SharedString>,
    session: Option<SharedString>,
    files: Vec<ChangedFile>,
    current: Option<SharedString>,
    on_open: Option<OnOpen>,
}

impl GitPanel {
    pub fn new(id: impl Into<ElementId>, repo: impl Into<SharedString>) -> Self {
        Self { id: id.into(), repo: repo.into(), branch: None, session: None, files: Vec::new(), current: None, on_open: None }
    }

    pub fn branch(mut self, branch: Option<SharedString>) -> Self {
        self.branch = branch;
        self
    }

    /// The title of the session in focus; with none, the panel says so in place of the files.
    pub fn session(mut self, title: Option<SharedString>) -> Self {
        self.session = title;
        self
    }

    pub fn files(mut self, files: Vec<ChangedFile>) -> Self {
        self.files = files;
        self
    }

    /// The file the review shows.
    pub fn current(mut self, path: Option<SharedString>) -> Self {
        self.current = path;
        self
    }

    pub fn on_open(mut self, handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_open = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for GitPanel {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let line = |icon: IconName, text: SharedString, selector: &'static str| {
            div()
                .debug_selector(move || selector.into())
                .flex()
                .items_center()
                .gap(px(8.))
                .h(px(24.))
                .px(px(12.))
                .text_size(TextSize::Sm.font_size())
                .child(Icon::new(icon).size(px(14.)).color(muted))
                .child(div().min_w_0().truncate().child(text))
        };
        let heading = |text: SharedString| {
            div().px(px(12.)).pt(px(12.)).pb(px(4.)).text_size(TextSize::Xs.font_size()).text_color(muted).child(text)
        };
        let body = match (&self.session, self.files.is_empty()) {
            (None, _) => Some("Focus a session to see what it changed."),
            (Some(_), true) => Some("This session has changed no file yet."),
            _ => None,
        };
        let rows = self.files.into_iter().map(|file| {
            let colour = tone(&file.change, &theme);
            let (folder, name) = split_path(&file.path);
            let (folder, name) = (SharedString::from(folder.to_string()), SharedString::from(name.to_string()));
            let current = self.current.as_ref() == Some(&file.path);
            let open = self.on_open.clone();
            let path = file.path.clone();
            let selector: SharedString = format!("git-file-{}", file.path).into();
            div()
                .id(ElementId::Name(selector.clone()))
                .debug_selector(move || selector.to_string())
                .flex()
                .flex_none()
                .items_center()
                .gap(px(6.))
                .h(px(ROW_HEIGHT))
                .mx(px(4.))
                .px(px(8.))
                .rounded(radius::md())
                .cursor_pointer()
                .text_size(TextSize::Sm.font_size())
                .when(current, |d| d.bg(theme.accent.opacity(0.12)))
                .when(!current, |d| d.hover(|s| s.bg(theme.muted_hover())))
                .when_some(open, |d, open| d.on_click(move |_, window, cx| open(&path, window, cx)))
                .child(div().flex_none().w(px(10.)).font_family(MONO_FONT_FAMILY).text_size(TextSize::Xs.font_size()).text_color(colour).child(letter(&file.change)))
                .child(FileIcon::file(&file.path).size(px(14.)))
                .child(div().flex_none().max_w(px(160.)).truncate().text_color(colour).child(name))
                .child(div().flex_1().min_w_0().truncate().text_size(TextSize::Xs.font_size()).text_color(muted).child(folder))
                .child(
                    div()
                        .flex_none()
                        .flex()
                        .gap(px(4.))
                        .font_family(MONO_FONT_FAMILY)
                        .text_size(TextSize::Xs.font_size())
                        .when(file.added > 0, |d| d.child(div().text_color(theme.diff_color(true)).child(format!("+{}", file.added))))
                        .when(file.removed > 0, |d| d.child(div().text_color(theme.diff_color(false)).child(format!("−{}", file.removed)))),
                )
        });
        let count = rows.len();
        div()
            .id(self.id)
            .debug_selector(|| "git-panel".into())
            .flex()
            .flex_col()
            .size_full()
            .pt(px(8.))
            .child(line(IconName::Folder, self.repo, "git-repo"))
            .when_some(self.branch, |d, branch| d.child(line(IconName::PrOpen, branch, "git-branch")))
            .when_some(self.session, |d, title| d.child(line(IconName::Forum, title, "git-session")))
            .child(heading(if count == 0 { "Changes".into() } else { format!("Changes · {count}").into() }))
            .when_some(body, |d, words| d.child(div().px(px(12.)).text_size(TextSize::Xs.font_size()).text_color(muted).child(words)))
            .child(div().id("git-files").flex().flex_col().flex_1().min_h_0().overflow_y_scroll().children(rows))
    }
}

#[cfg(test)]
mod tests;
