use super::{
    consts::{
        CLOSE_INSET, CLOSE_SIZE, CORNER, HERO_HEIGHT, HERO_PATH, HISTORY_MAX, PANEL_ALPHA,
        PANEL_CORNER, PANEL_GAP, SIDE, TILE_BOTTOM_ALPHA, TILE_CORNER, TILE_ICON, TILE_RING_ALPHA,
        TILE_SIZE, TILE_TOP_ALPHA,
    },
    helpers::fade,
    types::ReleaseKind,
};
use crate::scale::px;
use crate::{
    Button, ButtonVariant, Icon, IconName,
    theme::{ActiveTheme, Appearance, Theme},
};
use gpui_kit::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, ObjectFit, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div, linear_color_stop,
    linear_gradient,
};
use std::rc::Rc;
type Choice = Rc<dyn Fn(&mut Window, &mut App)>;

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
        Self {
            lead: lead.into(),
            text: text.into(),
            icon: IconName::Check,
            kind: None,
        }
    }
    /// What kind of change the note tells of: its icon and its colour follow.
    pub fn kind(mut self, kind: ReleaseKind) -> Self {
        self.kind = Some(kind);
        self.icon = kind.icon();
        self
    }
    /// The mark in the tile at the line's left (a tick by default).
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
    pub fn new(
        version: impl Into<SharedString>,
        notes: impl IntoIterator<Item = ReleaseNote>,
    ) -> Self {
        Self {
            version: version.into(),
            notes: notes.into_iter().collect(),
        }
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
        }
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
    pub fn labels(
        mut self,
        later: impl Into<SharedString>,
        install: impl Into<SharedString>,
    ) -> Self {
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
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme: Theme = cx.theme().clone();
        // The picture is dark at the top left whatever the theme, so the words over it are the dark theme's.
        let light = Theme::of(Appearance::Dark).foreground;
        let picture = div().absolute().inset_0().child(
            <gpui_kit::Img as gpui_kit::StyledImage>::object_fit(
                gpui_kit::img(HERO_PATH).size_full(),
                ObjectFit::Fill,
            )
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
            .child(
                div()
                    .text_size(px(15.))
                    .text_color(fade(light, 0.8))
                    .child(self.kicker.clone()),
            )
            .child(
                div()
                    .debug_selector(|| "release-version".into())
                    .text_size(px(84.))
                    .line_height(px(88.))
                    .text_color(light)
                    .child(self.version.clone()),
            );

        let rows = self.notes.into_iter().map(|note| note_row(note, &theme));
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
                .children(release.notes.into_iter().map(|note| note_row(note, &theme)))
        });
        let notes = div()
            .px(px(18.))
            .py(px(10.))
            .flex()
            .flex_col()
            .children(rows)
            .children(earlier);
        let notes = if scrolls {
            div()
                .id((self.id.clone(), "history"))
                .max_h(px(HISTORY_MAX))
                .overflow_y_scroll()
                .child(notes)
                .into_any_element()
        } else {
            notes.into_any_element()
        };
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
                    .variant(if close_only {
                        ButtonVariant::Primary
                    } else {
                        ButtonVariant::Secondary
                    })
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

/// The tile at a note's left: the icon in the colour of its kind, on a soft gradient of that colour, inside a thin ring of it. A
/// note with no kind has the neutral foreground in its place.
fn icon_tile(note: &ReleaseNote, theme: &Theme) -> gpui_kit::Div {
    let tone = note.kind.map_or(theme.foreground, |kind| kind.tone(theme));
    let (top, bottom, ring) = match note.kind {
        Some(_) => (TILE_TOP_ALPHA, TILE_BOTTOM_ALPHA, TILE_RING_ALPHA),
        None => (
            TILE_TOP_ALPHA / 2.,
            TILE_BOTTOM_ALPHA / 2.,
            TILE_RING_ALPHA / 2.,
        ),
    };
    div()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(px(TILE_SIZE))
        .rounded(px(TILE_CORNER))
        .border_1()
        .border_color(fade(tone, ring))
        .bg(linear_gradient(
            180.,
            linear_color_stop(fade(tone, top), 0.),
            linear_color_stop(fade(tone, bottom), 1.),
        ))
        .child(Icon::new(note.icon).size(px(TILE_ICON)).color(tone))
}

/// One note: the tile with its icon, the lead, and what it says.
fn note_row(note: ReleaseNote, theme: &Theme) -> gpui_kit::Div {
    div()
        .flex()
        .gap(px(14.))
        .py(px(9.))
        .child(icon_tile(&note, theme))
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .gap(px(2.))
                .child(
                    div()
                        .text_size(px(15.))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.foreground)
                        .child(note.lead),
                )
                .child(
                    div()
                        .text_size(px(14.))
                        .line_height(px(20.))
                        .text_color(theme.muted_foreground)
                        .child(note.text),
                ),
        )
}
