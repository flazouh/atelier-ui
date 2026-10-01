use gpui_kit::Hsla;

use crate::theme::Theme;

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
    pub(super) fn colors(self, theme: &Theme) -> (Hsla, Hsla) {
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
