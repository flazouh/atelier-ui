use std::rc::Rc;

use gpui_kit::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, ObjectFit, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    Icon, IconName,
    theme::{ActiveTheme, Appearance, Theme},
    typography::FONT_FAMILY,
};

use super::consts::{
    BAND_HEIGHT, BAND_WIDTH, CLOSE_HOVER_ALPHA, CLOSE_INSET, CLOSE_SIZE, COLUMN_GAP, CORNER,
    DATE_GAP, DATE_SIZE, HERO_PATH, LEAD_GAP, LEAD_SIZE, META_WIDTH, NOTE_GAP, ROW_PAD, SIDE,
    TEXT_LINE, TEXT_SIZE, TITLE_BOTTOM, TITLE_SIZE, VERSION_SIZE,
};

type Close = Rc<dyn Fn(&mut Window, &mut App)>;

/// One note of a release: a bold lead and a muted text under it.
#[derive(Clone)]
pub struct ReleaseNote {
    lead: SharedString,
    text: SharedString,
}

impl ReleaseNote {
    pub fn new(lead: impl Into<SharedString>, text: impl Into<SharedString>) -> Self {
        Self {
            lead: lead.into(),
            text: text.into(),
        }
    }
}

/// An earlier release and what it brought, listed under the current one.
#[derive(Clone)]
pub struct ReleaseVersion {
    version: SharedString,
    date: Option<SharedString>,
    notes: Vec<ReleaseNote>,
}

impl ReleaseVersion {
    pub fn new(
        version: impl Into<SharedString>,
        notes: impl IntoIterator<Item = ReleaseNote>,
    ) -> Self {
        Self {
            version: version.into(),
            date: None,
            notes: notes.into_iter().collect(),
        }
    }

    /// The date as a ready string ("Oct 9, 2026"). None draws no date line.
    pub fn date(mut self, date: Option<SharedString>) -> Self {
        self.date = date;
        self
    }
}

#[derive(IntoElement)]
pub struct ReleaseSheet {
    id: ElementId,
    title: SharedString,
    current: ReleaseVersion,
    earlier: Vec<ReleaseVersion>,
    on_close: Option<Close>,
}

impl ReleaseSheet {
    pub fn new(id: impl Into<ElementId>, version: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: "Changelog".into(),
            current: ReleaseVersion::new(version, []),
            earlier: Vec::new(),
            on_close: None,
        }
    }

    /// The title at the bottom left of the band ("Changelog" by default).
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }

    /// The current release's date, as a ready string. None (the default) draws no date line.
    pub fn date(mut self, date: Option<SharedString>) -> Self {
        self.current.date = date;
        self
    }

    pub fn note(mut self, note: ReleaseNote) -> Self {
        self.current.notes.push(note);
        self
    }

    pub fn notes(mut self, notes: impl IntoIterator<Item = ReleaseNote>) -> Self {
        self.current.notes.extend(notes);
        self
    }

    /// Earlier releases, newest first, listed under the current one.
    pub fn earlier(mut self, earlier: impl IntoIterator<Item = ReleaseVersion>) -> Self {
        self.earlier.extend(earlier);
        self
    }

    /// Fires when the close button is pressed.
    pub fn on_close(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for ReleaseSheet {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme: Theme = cx.theme().clone();
        // The picture is dark at the top left whatever the theme, so the words over it are the dark theme's.
        let light = Theme::of(Appearance::Dark).foreground;
        let mut hero = gpui_kit::img(HERO_PATH).w_full().h(px(BAND_HEIGHT));
        hero.style().aspect_ratio = Some(BAND_WIDTH / BAND_HEIGHT);
        let picture = div()
            .absolute()
            .top_0()
            .left_0()
            .w_full()
            .h(px(BAND_HEIGHT))
            .overflow_hidden()
            .child(
                <gpui_kit::Img as gpui_kit::StyledImage>::object_fit(hero, ObjectFit::Fill)
                    .rounded_tl(px(CORNER))
                    .rounded_tr(px(CORNER)),
            );
        let close = self.on_close;
        let band = div()
            .debug_selector(|| "release-band".into())
            .relative()
            .flex_none()
            .h(px(BAND_HEIGHT))
            .child(picture)
            .child(
                div()
                    .debug_selector(|| "release-title".into())
                    .absolute()
                    .left(px(SIDE))
                    .bottom(px(TITLE_BOTTOM))
                    .font_family(FONT_FAMILY)
                    .text_size(px(TITLE_SIZE))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(light)
                    .child(self.title),
            )
            .child(
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
                    .hover(move |style| style.bg(fade(light, CLOSE_HOVER_ALPHA)))
                    .on_click(move |_, window, cx| {
                        if let Some(f) = &close {
                            f(window, cx);
                        }
                    })
                    .child(Icon::new(IconName::Close).size(px(16.)).color(light)),
            );
        let releases = std::iter::once(self.current)
            .chain(self.earlier)
            .enumerate()
            .map(|(at, release)| release_row(release, at, &theme));
        div()
            .id(self.id)
            .debug_selector(|| "release-sheet".into())
            .relative()
            .w_full()
            .flex()
            .flex_col()
            .child(band)
            .child(div().px(px(SIDE)).flex().flex_col().children(releases))
    }
}

/// `color` at `t` of its strength.
fn fade(color: gpui_kit::Hsla, t: f32) -> gpui_kit::Hsla {
    gpui_kit::Hsla {
        a: color.a * t,
        ..color
    }
}

/// One release: its version and date at the left, its notes at the right, a hairline over it when it is not the first.
fn release_row(
    release: ReleaseVersion,
    at: usize,
    theme: &Theme,
) -> gpui_kit::Stateful<gpui_kit::Div> {
    let date = release.date.map(|date| {
        div()
            .debug_selector(move || format!("release-date-{at}"))
            .mt(px(DATE_GAP))
            .text_size(px(DATE_SIZE))
            .text_color(theme.muted_foreground)
            .child(date)
    });
    let meta = div()
        .flex_none()
        .w(px(META_WIDTH))
        .child(
            div()
                .text_size(px(VERSION_SIZE))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.foreground)
                .child(release.version),
        )
        .children(date);
    let notes = release
        .notes
        .into_iter()
        .enumerate()
        .map(|(n, note)| note_block(note, at, n, theme));
    div()
        .id(SharedString::from(format!("release-row-{at}")))
        .debug_selector(move || format!("release-{at}"))
        .font_family(FONT_FAMILY)
        .flex()
        .gap(px(COLUMN_GAP))
        .py(px(ROW_PAD))
        .when(at > 0, |row| row.border_t_1().border_color(theme.divider))
        .child(meta)
        .child(div().flex().flex_col().flex_1().min_w_0().children(notes))
}

/// One note: the lead in the foreground, the text in the muted foreground.
fn note_block(note: ReleaseNote, release: usize, at: usize, theme: &Theme) -> gpui_kit::Div {
    div()
        .mb(px(NOTE_GAP))
        .child(
            div()
                .debug_selector(move || format!("release-lead-{release}-{at}"))
                .mb(px(LEAD_GAP))
                .text_size(px(LEAD_SIZE))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.foreground)
                .child(note.lead),
        )
        .child(
            div()
                .debug_selector(move || format!("release-text-{release}-{at}"))
                .text_size(px(TEXT_SIZE))
                .line_height(px(TEXT_LINE))
                .text_color(theme.muted_foreground)
                .child(note.text),
        )
}
