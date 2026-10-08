//! Color tokens: the theme in force. Every colour a component paints comes from here; none is written
//! in a component. The themes are data files in `assets/themes`, listed in [`crate::themes`]; atelier
//! (light and dark) is the default.
//!
//! The design is borderless in every theme: surfaces part by tone and space, never by lines.
//! `background` is the page, `card` sits one step up, and `card_strong` one more for a surface inside a
//! card; an agent panel is a card, so a box in it is `card_strong`. Quiet text is `muted_foreground`
//! whole and a quiet mark is [`Theme::faint`]; neither is faded with an opacity. Components never draw
//! borders; `divider` exists only for the split between panes. The primary button is the page
//! inverted, the accent marks highlights and the selection only, and no button carries a colour of its
//! own.

use std::sync::Arc;

use gpui_kit::{
    App, BoxShadow, Global, Hsla, Pixels, SharedString, component::highlighter::HighlightTheme,
    point, px, transparent_black,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Appearance {
    Light,
    Dark,
}

#[derive(Clone, Debug)]
pub struct Theme {
    /// As the picker lists it: "atelier Dark", "Catppuccin Mocha".
    pub name: SharedString,
    /// The group the picker puts it in: "atelier", "GitHub", "Catppuccin".
    pub family: SharedString,
    pub appearance: Appearance,
    pub background: Hsla,
    pub foreground: Hsla,
    /// Raised surfaces: user bubbles, tool output, the prompt input, secondary buttons.
    pub card: Hsla,
    /// A surface inside a card, such as the details box of an approval.
    pub card_strong: Hsla,
    pub muted_foreground: Hsla,
    /// The split between panes. Not for components.
    pub divider: Hsla,
    pub primary: Hsla,
    pub primary_foreground: Hsla,
    /// beui's cyan: focus rings and the one live thing on screen, such as a running tool.
    pub accent: Hsla,
    /// Status text and marks: Tailwind's blue, emerald, rose, and amber, 600 in light and 400 in dark,
    /// as beui's `text-blue-600 dark:text-blue-400`.
    pub info: Hsla,
    pub danger: Hsla,
    pub success: Hsla,
    pub warning: Hsla,
    /// The warning tone for a fill or a large mark, at full strength: a badge, a bar. `warning` is the text tone, darkened or
    /// lightened until it reads on the page and the cards, and can be a duller colour than the theme means by "warning".
    pub warning_fill: Hsla,
    /// The wash under selected text in a text input.
    pub selection: Hsla,
    /// A popover's fill, and the colour of its shadow.
    pub popover: Hsla,
    pub shadow: Hsla,
    /// The washes under an added and a removed diff line.
    pub diff_added: Hsla,
    pub diff_removed: Hsla,
    /// A button's arrow chip at rest and hovered, and its arrow at rest (mem0's chip).
    pub chip_rest: Hsla,
    pub chip_hover: Hsla,
    pub chip_arrow: Hsla,
    /// The status marks by presence: running, done, failed, pending, cancelled.
    pub status: [Hsla; 5],
    /// A pull request's state marks in GitHub's own colours, whatever the theme: open, draft, merged, closed.
    pub pull: [Hsla; 4],
    /// The code's colours, by syntax name.
    pub syntax: Arc<HighlightTheme>,
}

impl Theme {
    /// atelier Light, the default light theme.
    pub fn light() -> Self {
        crate::themes::atelier(Appearance::Light).clone()
    }

    /// atelier Dark, the default dark theme.
    pub fn dark() -> Self {
        crate::themes::atelier(Appearance::Dark).clone()
    }

    /// atelier in `appearance`.
    pub fn of(appearance: Appearance) -> Self {
        crate::themes::atelier(appearance).clone()
    }

    /// `bg-primary/90` in beui: the primary with 10% of the page showing through.
    pub fn primary_hover(&self) -> Hsla {
        mix(self.primary, self.background, 0.1)
    }

    /// The hover fill of ghost controls: a thin wash of the text color, so it shows on
    /// both the page and a card, in light and dark.
    pub fn muted_hover(&self) -> Hsla {
        self.foreground.opacity(0.06)
    }

    /// A surface inside a box that is itself inside a card: a thin wash of the ink, so it steps up from
    /// whatever it sits on in every theme, where the page can be as light as `card_strong`.
    pub fn wash(&self) -> Hsla {
        self.foreground.opacity(0.06)
    }

    /// A quiet mark that must still be seen: a chevron, a status check, a line number, a tool's icon. Muted,
    /// dimmed toward the card only as far as [`MARK_CONTRAST`] holds on the page, the card and `card_strong`.
    /// Quiet text takes `muted_foreground` whole.
    pub fn faint(&self) -> Hsla {
        let surfaces = [self.background, self.card, self.card_strong];
        (10..=20)
            .map(|step| mix(self.card, self.muted_foreground, step as f32 * 0.05))
            .find(|c| surfaces.iter().all(|bg| contrast(*c, *bg) >= MARK_CONTRAST))
            .unwrap_or(self.muted_foreground)
    }

    /// The tone of a status mark: the theme's own, by presence. atelier's are a muted ramp; an imported
    /// theme's are its blue, green, red, yellow and muted text.
    pub fn status_tone(&self, tone: StatusTone) -> Hsla {
        let rank = match tone {
            StatusTone::Running => 0,
            StatusTone::Done => 1,
            StatusTone::Failed => 2,
            StatusTone::Pending => 3,
            StatusTone::Cancelled => 4,
        };
        self.status[rank]
    }

    /// The background of an added or removed diff line.
    pub fn diff_line(&self, added: bool) -> Hsla {
        if added {
            self.diff_added
        } else {
            self.diff_removed
        }
    }

    /// The sign and the `+n` / `-n` counts of a diff.
    pub fn diff_color(&self, added: bool) -> Hsla {
        if added { self.success } else { self.danger }
    }
}

/// The 1px edge of a dropdown or menu panel floating over the page: a step from the popover fill toward the ink.
/// It is the one border atelier draws (`AGENTS.md`); a popover that is not a dropdown or a menu has none.
pub fn dropdown_edge(theme: &Theme) -> Hsla {
    mix(theme.popover, theme.foreground, 0.14)
}

/// Every popover's shadow, Select's menu and the PR hover card alike: Tailwind's `shadow-lg`,
/// `0 10px 15px -3px` and `0 4px 6px -4px`, in the theme's shadow colour. Popovers have no border.
pub fn popover_shadow(theme: &Theme) -> Vec<BoxShadow> {
    let color = theme.shadow;
    vec![
        BoxShadow {
            color,
            offset: point(px(0.), px(10.)),
            blur_radius: px(15.),
            spread_radius: px(-3.),
            inset: false,
        },
        BoxShadow {
            color,
            offset: point(px(0.), px(4.)),
            blur_radius: px(6.),
            spread_radius: px(-4.),
            inset: false,
        },
    ]
}

impl Global for Theme {}

pub trait ActiveTheme {
    fn theme(&self) -> &Theme;
}

impl ActiveTheme for App {
    fn theme(&self) -> &Theme {
        self.global::<Theme>()
    }
}

impl Appearance {
    /// Light or dark to match a macOS appearance; the vibrant ones count as their plain kind.
    pub fn of_system(system: gpui_kit::WindowAppearance) -> Self {
        match system {
            gpui_kit::WindowAppearance::Dark | gpui_kit::WindowAppearance::VibrantDark => {
                Self::Dark
            }
            gpui_kit::WindowAppearance::Light | gpui_kit::WindowAppearance::VibrantLight => {
                Self::Light
            }
        }
    }
}

/// Matches the system appearance: light or dark as macOS is set now.
pub fn follow_system(cx: &mut App) {
    set_appearance(Appearance::of_system(cx.window_appearance()), cx);
}

/// Switches to the theme of the current family in `appearance` (atelier when none is set, or when the
/// family has none in it), as light and dark follow the system.
pub fn set_appearance(appearance: Appearance, cx: &mut App) {
    let family = cx.try_global::<Theme>().map(|t| t.family.clone());
    let theme = crate::themes::all()
        .iter()
        .find(|t| Some(&t.family) == family.as_ref() && t.appearance == appearance)
        .unwrap_or_else(|| crate::themes::atelier(appearance))
        .clone();
    set_theme(theme, cx);
}

/// Puts `theme` in force: every window repaints in it, gpui-component's inputs, Markdown and the code
/// editor's colours included.
pub fn set_theme(theme: Theme, cx: &mut App) {
    let shown = with_pick(&theme, picked(cx));
    cx.set_global(Unpicked(theme));
    show(shown, cx);
}

fn show(theme: Theme, cx: &mut App) {
    sync_component_theme(&theme, cx);
    cx.set_global(theme);
    cx.refresh_windows();
}

/// The theme as it was chosen, before the reader's primary colour was laid on it.
struct Unpicked(Theme);

impl Global for Unpicked {}

/// The colour the reader picked for the primary, if any.
struct Picked(Option<Hsla>);

impl Global for Picked {}

/// The primary colour the reader picked in Settings, or `None` for the theme's own (its ink).
pub fn picked(cx: &App) -> Option<Hsla> {
    cx.try_global::<Picked>().and_then(|p| p.0)
}

/// Puts the reader's primary colour in force in the theme that is showing, at once, and keeps it through every
/// later change of theme. `None` returns to the theme's own.
pub fn set_pick(pick: Option<Hsla>, cx: &mut App) {
    cx.set_global(Picked(pick));
    if let Some(base) = cx.try_global::<Unpicked>().map(|u| u.0.clone()) {
        show(with_pick(&base, pick), cx);
    }
}

/// The text on a `fill`: the lighter of the page and the ink when it reaches [`FILL_TEXT_CONTRAST`] on the fill
/// (white on blue, as beui.dev has it), else the darker one when that does (dark on yellow), else `None`. Worked out
/// from the fill each time; nothing stores it.
pub fn text_on(theme: &Theme, fill: Hsla) -> Option<Hsla> {
    let (light, dark) = if luminance(theme.background) >= luminance(theme.foreground) {
        (theme.background, theme.foreground)
    } else {
        (theme.foreground, theme.background)
    };
    [light, dark]
        .into_iter()
        .find(|text| contrast(*text, fill) >= FILL_TEXT_CONTRAST)
}

/// A mark on a `fill` (a tick, a bar, an icon with no words): the same tone as the words ([`text_on`]), and the ink
/// when neither tone reaches 3:1.
pub fn mark_on(theme: &Theme, fill: Hsla) -> Hsla {
    text_on(theme, fill).unwrap_or(theme.foreground)
}

/// The patch under a key cap on a `fill`: a tone of the fill, moved toward `text` as far as it goes (at most
/// `CAP_WASH`) while `text` keeps [`FILL_TEXT_CONTRAST`] on it.
pub fn cap_patch(fill: Hsla, text: Hsla) -> Hsla {
    (0..=8)
        .rev()
        .map(|step| mix(fill, text, CAP_WASH * step as f32 / 8.))
        .find(|patch| contrast(text, *patch) >= FILL_TEXT_CONTRAST)
        .unwrap_or(fill)
}

/// Whether `color` can be the primary in `theme`: some text reaches [`FILL_TEXT_CONTRAST`] on it.
pub fn can_be_primary(theme: &Theme, color: Hsla) -> bool {
    text_on(theme, color).is_some()
}

/// `theme` with the reader's `pick` as the primary. The primary button's fill is the pick and its text is the
/// lighter of the page and the ink, or the darker on a light pick (see [`text_on`]). The pick is also the accent, and the wash under a selection takes its hue,
/// unless the pick fails [`MARK_CONTRAST`] against the page: then the theme's accent stays. A pick no text can
/// be read on, and no pick, leave the theme as it is.
pub fn with_pick(theme: &Theme, pick: Option<Hsla>) -> Theme {
    let mut out = theme.clone();
    let Some(pick) = pick else { return out };
    let Some(text) = text_on(theme, pick) else {
        return out;
    };
    out.primary = pick;
    out.primary_foreground = text;
    if contrast(pick, theme.background) >= MARK_CONTRAST {
        out.accent = pick;
        out.selection = pick.opacity(theme.selection.a);
    }
    out
}

/// gpui-component draws text inputs and Markdown from its own theme, so mirror our tokens into it.
fn sync_component_theme(theme: &Theme, cx: &mut App) {
    use gpui_kit::component::{Theme as ComponentTheme, ThemeMode};
    let mode = match theme.appearance {
        Appearance::Light => ThemeMode::Light,
        Appearance::Dark => ThemeMode::Dark,
    };
    // Picks gpui-component's own light or dark palette, then our tokens override it.
    ComponentTheme::change(mode, None, cx);
    let component = ComponentTheme::global_mut(cx);
    component.font_family = crate::typography::FONT_FAMILY.into();
    component.mono_font_family = crate::typography::MONO_FONT_FAMILY.into();
    component.radius = radius::md();
    component.radius_lg = radius::lg();
    let colors = &mut component.colors;
    colors.background = theme.background;
    colors.foreground = theme.foreground;
    colors.muted = theme.card;
    colors.muted_foreground = theme.muted_foreground;
    // Borderless: gpui-component draws its input and popover edges in the fill color, so they vanish.
    colors.border = theme.card;
    colors.input = theme.card;
    colors.ring = theme.accent;
    colors.caret = theme.foreground;
    colors.selection = theme.selection;
    colors.popover = theme.popover;
    colors.primary = theme.primary;
    colors.link = theme.foreground;
    // Markdown and the scrollbar read a copy of these tokens, so rebuild it.
    ComponentTheme::sync_base(cx);
    // The code editor's token colors follow the theme too.
    crate::code_editor::install_syntax_theme(theme, cx);
}

/// What a status mark says. Its tone comes from `ramp` through [`Theme::status_tone`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusTone {
    /// Work in flight: the most present mark.
    Running,
    Done,
    Failed,
    /// Waiting on the user.
    Pending,
    /// Never ran: the quietest mark.
    Cancelled,
}

/// The least contrast for text (WCAG AA, normal size) and for a UI mark.
pub const TEXT_CONTRAST: f32 = 4.5;
pub const MARK_CONTRAST: f32 = 3.0;
/// The least contrast the words on a coloured fill (the primary button) reach. Alex's choice: 3:1, so white text
/// can stand on the web's blue.
pub const FILL_TEXT_CONTRAST: f32 = 3.0;
/// How far a key cap's patch moves from the fill toward the text, at most.
const CAP_WASH: f32 = 0.2;

/// `color` moved just far enough to reach `least` against every one of `against`; `None` when it already does. A coloured
/// tone (an amber, a red) keeps its hue and saturation and moves in lightness, toward the side of the ink; a grey one
/// moves toward the ink. Mixing a colour with an ink of another hue would turn an amber into a grey.
pub fn raise(color: Hsla, ink: Hsla, against: &[Hsla], least: f32) -> Option<Hsla> {
    let passes = |c: Hsla| against.iter().all(|bg| contrast(c, *bg) >= least);
    if passes(color) {
        return None;
    }
    if color.s > 0.2 {
        let toward = if ink.l < color.l { -1. } else { 1. };
        if let Some(held) = (1..=50)
            .map(|step| Hsla {
                l: (color.l + toward * step as f32 * 0.02).clamp(0., 1.),
                ..color
            })
            .find(|c| passes(*c))
        {
            return Some(held);
        }
    }
    (1..=20)
        .map(|step| mix(color, ink, step as f32 * 0.05))
        .find(|c| passes(*c))
        .or(Some(ink))
}

/// Every status mark reaches [`MARK_CONTRAST`] on the page: one that does not moves toward the ink,
/// the least that passes. atelier's quiet pending and cancelled marks moved this way on 2026-09-29.
pub fn raise_marks(theme: &mut Theme) {
    for tone in &mut theme.status {
        if let Some(raised) = raise(*tone, theme.foreground, &[theme.background], MARK_CONTRAST) {
            *tone = raised;
        }
    }
}

/// The relative luminance of an opaque color, as WCAG defines it.
fn luminance(c: Hsla) -> f32 {
    let c = c.to_rgb();
    let channel = |x: f32| {
        if x <= 0.03928 {
            x / 12.92
        } else {
            ((x + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(c.r) + 0.7152 * channel(c.g) + 0.0722 * channel(c.b)
}

/// The contrast ratio between two opaque colors, as WCAG defines it.
pub fn contrast(a: Hsla, b: Hsla) -> f32 {
    let (x, y) = (luminance(a), luminance(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

/// Blends `a` toward `b`, weighting each channel by its alpha, so a clear color adds no black.
pub fn mix(a: Hsla, b: Hsla, amount: f32) -> Hsla {
    let (a, b) = (a.to_rgb(), b.to_rgb());
    let lerp = |x: f32, y: f32| x + (y - x) * amount;
    let alpha = lerp(a.a, b.a);
    if alpha == 0. {
        return transparent_black();
    }
    let channel = |x: f32, y: f32| lerp(x * a.a, y * b.a) / alpha;
    gpui_kit::Rgba {
        r: channel(a.r, b.r),
        g: channel(a.g, b.g),
        b: channel(a.b, b.b),
        a: alpha,
    }
    .into()
}

/// Tailwind radii as beui uses them. Keep one radius system per surface group.
pub mod radius {
    use super::*;
    use crate::scale::px;
    /// `rounded-md`: tool-call rows, small icon buttons. The radii scale with the interface.
    pub fn md() -> Pixels {
        px(6.)
    }
    /// `rounded-lg`: buttons and inputs.
    pub fn lg() -> Pixels {
        px(8.)
    }
    /// `rounded-xl`: tool output, diffs, code blocks, the prompt input.
    pub fn xl() -> Pixels {
        px(12.)
    }
    /// The cards of the agent panel and the PR view: tool calls, diffs, approvals, subagents, to-dos, changed files.
    /// The PR view sets the measure: 8px corners, 12px side padding, headers 32 to 36px tall.
    pub fn card() -> Pixels {
        lg()
    }
    /// `rounded-2xl`: message bubbles.
    pub fn xxl() -> Pixels {
        px(16.)
    }
}

#[cfg(test)]
mod tests;
