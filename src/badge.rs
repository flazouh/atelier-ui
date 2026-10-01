//! A small pill label: a status, a count, or a model name.
use gpui_kit::{App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div, };
use crate::scale::px;
use crate::theme::{ActiveTheme, Theme};
/// What a badge says about its subject, which sets its color.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tone {
    #[default]
    Neutral,
    Primary,
    Info,
    Success,
    Warning,
    Danger,
}
impl Tone {
    /// Fill and text. Tinted tones use a 14% wash of their color, like beui's `bg-amber-500/10`, a little
    /// stronger because no border helps them.
    fn colors(self, theme: &Theme) -> (Hsla, Hsla) {
        let tinted = |c: Hsla| (c.opacity(0.14), c);
        match self {
            Self::Neutral => (theme.card_strong, theme.muted_foreground),
            Self::Primary => (theme.primary, theme.primary_foreground),
            Self::Info => tinted(theme.info),
            Self::Success => tinted(theme.success),
            Self::Warning => tinted(theme.warning),
            Self::Danger => tinted(theme.danger),
        }
    }
}
#[derive(IntoElement)]
pub struct Badge {
    label: SharedString,
    tone: Tone,
}
impl Badge {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self { label: label.into(), tone: Tone::default() }
    }
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }
}
impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (fill, text) = self.tone.colors(cx.theme());
        div()
            .flex()
            .flex_none()
            .items_center()
            .h(px(20.))
            .px(px(8.))
            .rounded_full()
            .bg(fill)
            .text_color(text)
            .text_size(px(11.))
            .line_height(px(18.))
            .font_weight(FontWeight::MEDIUM)
            .whitespace_nowrap()
            .child(self.label)
    }
}
