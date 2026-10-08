use std::rc::Rc;
use gpui_kit::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, ObjectFit, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
};
use crate::scale::px;
use crate::{
    Button, ButtonVariant, Icon, IconName,
    theme::{ActiveTheme, Appearance, Theme},
};
use super::{
    consts::{CLOSE_INSET, CLOSE_SIZE, CORNER, HERO_HEIGHT, HERO_PATH, HISTORY_MAX, PANEL_ALPHA, FOOT_HEIGHT, GLYPH_SIZE, GLYPH_SLOT, HAIRLINE_ALPHA, MIN_NOTES_HEIGHT, PANEL_CORNER, PANEL_GAP, SHEET_CHROME, SIDE},
    helpers::fade,
    types::ReleaseKind,
};
type Choice = Rc<dyn Fn(&mut Window, &mut App)>;
/// The colour of a kind in the theme in force, as the owner gives it.
type KindColors = Rc<dyn Fn(ReleaseKind, &Theme) -> gpui_kit::Hsla>;

/// One line of what is new: a short lead and what it means.
#[derive(Clone)]
pub struct ReleaseNote {
    lead: SharedString,
    text: SharedString,
    icon: IconName,
    kind: Option<ReleaseKind>,
}
impl ReleaseNote {
    pub fn new(lead: impl Into<SharedString>, text: impl Into<SharedString>) -> Self {
        Self { lead: lead.into(), text: text.into(), icon: IconName::Check, kind: None }
    }
    /// What kind of change the note tells of: its icon, its label and its colour follow.
    pub fn kind(mut self, kind: ReleaseKind) -> Self {
        self.kind = Some(kind);
        self.icon = kind.icon();
        self
    }
    /// The mark at the line's left (a tick by default).
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = icon;
        self
    }
}

/// An earlier version and what it brought, listed under the notes of the one the sheet is about.
#[derive(Clone)]
pub struct ReleaseVersion {
    version: SharedString,
    notes: Vec<ReleaseNote>,
}
impl ReleaseVersion {
    pub fn new(version: impl Into<SharedString>, notes: impl IntoIterator<Item = ReleaseNote>) -> Self {
        Self { version: version.into(), notes: notes.into_iter().collect() }
    }
}

#[derive(IntoElement)]
pub struct ReleaseSheet {
    id: ElementId,
    version: SharedString,
    kicker: SharedString,
    notes: Vec<ReleaseNote>,
    earlier: Vec<ReleaseVersion>,
    later: SharedString,
    install: SharedString,
    on_later: Option<Choice>,
    on_install: Option<Choice>,
    colors: Option<KindColors>,
}
impl ReleaseSheet {
    pub fn new(id: impl Into<ElementId>, version: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            version: version.into(),
            kicker: "What is new in".into(),
            notes: Vec::new(),
            earlier: Vec::new(),
            later: "Later".into(),
            install: "Restart and update".into(),
            on_later: None,
            on_install: None,
            colors: None,
        }
    }
    /// The colour of each kind of note, in the theme in force: its icon and its label. This crate names none, so the owner gives
    /// them, and sees the theme so that a colour can stay readable on a light page. A note of a kind the owner gave none to has
    /// the neutral foreground.
    pub fn kind_colors(mut self, colors: impl Fn(ReleaseKind, &Theme) -> gpui_kit::Hsla + 'static) -> Self {
        self.colors = Some(Rc::new(colors));
        self
    }
    /// The small line over the version ("What is new in" by default).
    pub fn kicker(mut self, kicker: impl Into<SharedString>) -> Self {
        self.kicker = kicker.into();
        self
    }
    pub fn note(mut self, note: ReleaseNote) -> Self {
        self.notes.push(note);
        self
    }
    pub fn notes(mut self, notes: impl IntoIterator<Item = ReleaseNote>) -> Self {
        self.notes.extend(notes);
        self
    }
    /// Earlier versions, newest first, listed under the notes. The list scrolls when it is long.
    pub fn earlier(mut self, earlier: impl IntoIterator<Item = ReleaseVersion>) -> Self {
        self.earlier.extend(earlier);
        self
    }
    /// The words on the two buttons ("Later", "Restart and update"). With no [`Self::on_install`] only the first shows.
    pub fn labels(mut self, later: impl Into<SharedString>, install: impl Into<SharedString>) -> Self {
        (self.later, self.install) = (later.into(), install.into());
        self
    }
    pub fn on_later(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_later = Some(Rc::new(f));
        self
    }
    pub fn on_install(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_install = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for ReleaseSheet {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme: Theme = cx.theme().clone();
        // The picture is dark at the top left whatever the theme, so the words over it are the dark theme's.
        let light = Theme::of(Appearance::Dark).foreground;
        let picture = div()
            .absolute()
            .inset_0()
            .child(
                <gpui_kit::Img as gpui_kit::StyledImage>::object_fit(gpui_kit::img(HERO_PATH).size_full(), ObjectFit::Fill)
                    .rounded(px(CORNER)),
            );

        let head = div()
            .relative()
            .flex_none()
            .h(px(HERO_HEIGHT))
            .px(px(SIDE))
            .pt(px(SIDE))
            .flex()
            .flex_col()
            .gap(px(6.))
              .child(div().text_size(px(15.)).text_color(fade(light, 0.8)).child(self.kicker.clone()))
            .child(                div()
                    .debug_selector(|| "release-version".into())
                    .text_size(px(84.))
                    .line_height(px(88.))
                    .text_color(light)
                    .child(self.version.clone()));

        let colors = self.colors.clone();
        let rows = self.notes.into_iter().enumerate().map(|(i, note)| note_row(note, i, colors.as_ref(), &theme));
        let scrolls = !self.earlier.is_empty();
        let earlier = self.earlier.into_iter().enumerate().map(|(i, release)| {
            div()
                .flex()
                .flex_col()
                .child(
                    div()
                        .debug_selector(move || format!("release-earlier-{i}"))
                        .mt(px(12.))
                        .pt(px(12.))
                        .border_t_1()
                        .border_color(theme.divider)
                        .text_size(px(13.))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.muted_foreground)
                        .child(release.version),
                )
                .children(release.notes.into_iter().enumerate().map(|(i, note)| note_row(note, i, colors.as_ref(), &theme)))
        });
        let notes = div().px(px(18.)).py(px(10.)).flex().flex_col().children(rows).children(earlier);
        // The notes scroll when they do not fit: with earlier versions under them, past a height of their own; always, past what the
        // window leaves once the picture, the panel's gaps and its foot have taken theirs. At a zoom the window holds fewer design
        // pixels, so a sheet that fits at 1 would stand taller than the window.
        let foot = if self.on_install.is_some() { FOOT_HEIGHT } else { 0. };
        let room = (crate::scale::design(window.viewport_size().height) - SHEET_CHROME - foot).max(MIN_NOTES_HEIGHT);
        let most = if scrolls { HISTORY_MAX.min(room) } else { room };
        let notes = div().id((self.id.clone(), "history")).max_h(px(most)).overflow_y_scroll().child(notes).into_any_element();
        let (later, install, install_label) = (self.on_later, self.on_install, self.install);
        let close = later.clone();
        // With nothing to restart, the one button closes the sheet, and it is the main one.
        let close_only = install.is_none();
        let foot = div()
                .flex()
                .justify_end()
                .gap(px(10.))
                .px(px(14.))
                .py(px(14.))
                .border_t_1()
                .border_color(theme.divider)
                .child(
                    Button::new((self.id.clone(), "later"))
                        .debug_name("release-later")
                        .label(self.later)
                        .variant(if close_only { ButtonVariant::Primary } else { ButtonVariant::Secondary })
                        .on_click(move |_, window, cx| {
                            if let Some(f) = &later {
                                f(window, cx);
                            }
                        }),
                )
                .children(install.map(|on_install| {
                    Button::new((self.id.clone(), "install"))
                        .debug_name("release-install")
                        .label(install_label)
                        .variant(ButtonVariant::Primary)
                        .on_click(move |_, window, cx| on_install(window, cx))
                }));

        let panel = div()
            .relative()
            .flex_none()
            .mx(px(PANEL_GAP))
            .mb(px(PANEL_GAP))
            .rounded(px(PANEL_CORNER))
            .overflow_hidden()
            .bg(fade(theme.popover, PANEL_ALPHA))
            .child(notes)
            .children((!close_only).then_some(foot));

        // With nothing to restart there is no foot: Close is a button at the top right, over the picture.
        let close_button = close_only.then(|| {
            div()
                .id((self.id.clone(), "close"))
                .debug_selector(|| "release-close".into())
                .absolute()
                .top(px(CLOSE_INSET))
                .right(px(CLOSE_INSET))
                .size(px(CLOSE_SIZE))
                .rounded(px(CLOSE_SIZE / 2.))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .hover(move |style| style.bg(fade(light, 0.18)))
                .on_click(move |_, window, cx| {
                    if let Some(f) = &close {
                        f(window, cx);
                    }
                })
                .child(Icon::new(IconName::Close).size(px(16.)).color(light))
        });
        div()
            .id(self.id)
            .debug_selector(|| "release-sheet".into())
            .relative()
            .w_full()
            .overflow_hidden()
            .flex()
            .flex_col()
            .child(picture)
            .child(head)
            .child(panel)
            .children(close_button)
    }
}

/// One note: its icon in the colour of its kind, with no tile, then its kind in small capitals in that colour, the lead and what it
/// says. A hairline parts it from the note above. A note with no kind has a plain mark in the neutral foreground and no label.
fn note_row(note: ReleaseNote, at: usize, colors: Option<&KindColors>, theme: &Theme) -> gpui_kit::Div {
    let tone = note.kind.and_then(|kind| colors.map(|colors| colors(kind, theme))).unwrap_or(theme.foreground);
    let label = note.kind.map(|kind| {
        div()
            .debug_selector(move || format!("release-kind-{at}"))
            .text_size(px(10.5))
            .line_height(px(14.))
            .font_weight(FontWeight::BOLD)
            .text_color(tone)
            .child(SharedString::from(kind.label().to_uppercase()))
    });
    div()
        .flex()
        .gap(px(14.))
        .py(px(11.))
        .when(at > 0, |row| row.border_t_1().border_color(fade(theme.foreground, HAIRLINE_ALPHA)))
        .child(
            div()
                .flex()
                .flex_none()
                .justify_center()
                .w(px(GLYPH_SLOT))
                .pt(px(1.))
                .child(Icon::new(note.icon).size(px(GLYPH_SIZE)).color(tone)),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .gap(px(2.))
                .children(label)
                .child(div().text_size(px(15.)).font_weight(FontWeight::MEDIUM).text_color(theme.foreground).child(note.lead))
                .child(div().text_size(px(14.)).line_height(px(20.)).text_color(theme.muted_foreground).child(note.text)),
        )
}
