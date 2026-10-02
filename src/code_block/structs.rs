use std::sync::Arc;

use gpui_kit::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, StyledText, Window, div,
    prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    copy_feedback::CopyFeedback,
    file_icon::FileIcon,
    focus::PressStop,
    icon::{Icon, IconName},
    motion::duration,
    theme::{ActiveTheme, radius},
    typography::{MONO_FONT_FAMILY, TextSize},
};
use super::types::{CodeBlockStatus, LINE_HEIGHT, MAX_HEIGHT};
use super::helpers::{gutter_numbers, whole_text_runs};

#[derive(IntoElement)]
pub struct CodeBlock {
    id: ElementId,
    pub(super) code: SharedString,
    pub(super) language: SharedString,
    pub(super) filename: SharedString,
    pub(super) status: CodeBlockStatus,
    show_line_numbers: bool,
    pub(super) highlight_lines: Vec<u32>,
    max_height: f32,
    wrap: bool,
    copyable: bool,
}

impl CodeBlock {
    pub fn new(id: impl Into<ElementId>, code: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            code: code.into(),
            language: SharedString::default(),
            filename: SharedString::default(),
            status: CodeBlockStatus::Complete,
            show_line_numbers: true,
            highlight_lines: Vec::new(),
            max_height: MAX_HEIGHT,
            wrap: false,
            copyable: true,
        }
    }

    /// A file name or short caption, shown in mono before the language tag.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.filename = title.into();
        self
    }

    pub fn language(mut self, language: impl Into<SharedString>) -> Self {
        self.language = language.into();
        self
    }

    pub fn status(mut self, status: CodeBlockStatus) -> Self {
        self.status = status;
        self
    }

    pub fn show_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    /// 1-based line numbers to tint, such as the lines a diff just touched.
    pub fn highlight_lines(mut self, lines: impl IntoIterator<Item = u32>) -> Self {
        self.highlight_lines = lines.into_iter().collect();
        self
    }

    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = height;
        self
    }

    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn copyable(mut self, copyable: bool) -> Self {
        self.copyable = copyable;
        self
    }
}

impl RenderOnce for CodeBlock {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let reduce = cx.reduce_motion();
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let streaming = self.status == CodeBlockStatus::Streaming;
        // lucide's `animate-spin`.
        let spin_ms = duration::SPIN.as_millis();
        let spin = if streaming && !reduce {
            (std::time::UNIX_EPOCH.elapsed().unwrap_or_default().as_millis() % spin_ms) as f32 / spin_ms as f32
        } else {
            0.
        };
        if streaming && !reduce {
            window.request_animation_frame();
        }
        let copy_state = window.use_keyed_state(self.id.clone(), cx, |_, _| CopyFeedback::default());
        let copied = copy_state.read(cx).copied();
        let child = |name: &'static str| ElementId::NamedChild(Arc::new(self.id.clone()), name.into());

        let header = div()
            .flex()
            .items_center()
            .gap(px(10.))
            .h(px(40.))
            .px(px(12.))
            // The file's own icon when it has a name; the code glyph when it is a bare snippet.
            .child(if self.filename.is_empty() {
                div()
                    .flex_none()
                    .text_color(theme.faint())
                    .child(Icon::new(IconName::Code).size(px(14.)))
                    .into_any_element()
            } else {
                FileIcon::file(&self.filename).into_any_element()
            })
            .when(!self.filename.is_empty(), |d| {
                d.child(
                    div()
                        .min_w_0()
                        .truncate()
                        .font_family(MONO_FONT_FAMILY)
                        .text_size(TextSize::Xs.font_size())
                        .text_color(theme.foreground.opacity(0.8))
                        .child(self.filename.clone()),
                )
            })
            .child(
                div()
                    .flex_none()
                    .text_size(px(10.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(muted)
                    .child(self.language.to_uppercase()),
            )
            .child(div().flex_1())
            .child(
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap(px(4.))
                    .text_size(px(10.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(if streaming { theme.info } else { theme.success })
                    .child(if streaming {
                        Icon::new(IconName::Progress).size(px(12.)).turn(spin).into_any_element()
                    } else {
                        Icon::new(IconName::Check).size(px(12.)).into_any_element()
                    })
                    .child(if streaming { "Writing" } else { "Ready" }),
            )
            .when(self.copyable, |d| {
                let text = self.code.clone();
                d.child(
                    div()
                        .id(child("copy"))
                        .flex()
                        .flex_none()
                        .size(px(28.))
                        .items_center()
                        .justify_center()
                        .rounded_full()
                        .cursor_pointer()
                        .text_color(muted)
                        .hover(|s| s.bg(theme.background.opacity(0.7)).text_color(theme.foreground))
                        .press_stop((self.id.clone(), "copy-focus"), crate::theme::radius::md(), window, cx)
                        .on_click(move |_, _, cx| {
                            CopyFeedback::click(&copy_state, |f| f, text.to_string(), cx);
                        })
                        .child(Icon::new(if copied { IconName::Check } else { IconName::Copy }).size(px(14.))),
                )
            });

        let language = crate::syntax::LANGUAGES
            .iter()
            .find(|l| self.language.eq_ignore_ascii_case(l))
            .copied()
            .or_else(|| crate::syntax::language_for(&self.filename));
        let runs = language.and_then(|language| crate::syntax::highlight(language, &self.code, child("syntax"), cx));
        let line_count = self.code.split('\n').count();
        // A wrapped line takes more than one row, so the gutter and the tint bands, which count rows, only
        // fit an unwrapped block.
        let gutter = self.show_line_numbers && !self.wrap;
        let text = match &runs {
            Some(lines) => StyledText::new(self.code.clone()).with_highlights(whole_text_runs(&self.code, lines)),
            None => StyledText::new(self.code.clone()),
        };
        let bands = self.highlight_lines.iter().filter(|n| (1..=line_count as u32).contains(n)).map(|n| {
            div()
                .absolute()
                .left_0()
                .right_0()
                .top(px((n - 1) as f32 * LINE_HEIGHT))
                .h(px(LINE_HEIGHT))
                .bg(theme.info.opacity(0.07))
        });
        let pre = div()
            .relative()
            .flex()
            .font_family(MONO_FONT_FAMILY)
            .text_size(TextSize::Xs.font_size())
            .line_height(px(LINE_HEIGHT))
            .text_color(theme.foreground.opacity(0.85))
            .children(bands)
            .when(gutter, |d| {
                d.child(
                    div()
                        .w(px(44.))
                        .flex_none()
                        .pr(px(12.))
                        .text_right()
                        .text_color(theme.faint())
                        .child(gutter_numbers(line_count)),
                )
            })
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .pl(px(if gutter { 4. } else { 16. }))
                    .pr(px(16.))
                    .when(!self.wrap, |d| d.whitespace_nowrap())
                    .child(text),
            );

        div()
            .flex()
            .flex_col()
            .w_full()
            .overflow_hidden()
            .rounded(radius::card())
            .bg(theme.card.opacity(0.8))
            .text_size(TextSize::Sm.font_size())
            .child(header)
            .child(
                div()
                    .id(child("scroll"))
                    .max_h(px(self.max_height))
                    .overflow_scroll()
                    .py(px(8.))
                    .bg(theme.card_strong)
                    .child(pre),
            )
    }
}
