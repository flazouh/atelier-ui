use std::rc::Rc;

use gpui_kit::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, ObjectFit, ParentElement, RenderOnce, SharedString,
    Styled, Window, div,
};

use super::consts::{
    BUTTON_GAP, DATE_GAP, DATE_SIZE, FOOT_TOP, HERO_CORNER, HERO_HEIGHT, HERO_PATH, LEAD_GAP, LEAD_SIZE, NAME_SIZE,
    NOTES_TOP, NOTE_GAP, PAD, PILL_BORDER, PILL_SIZE, SIDE, TEXT_LINE, TEXT_SIZE, TITLE_SIZE, TITLE_TOP,
};
use crate::{
    IconName,
    button::{Button, ButtonSize, ButtonVariant},
    release_sheet::ReleaseNote,
    scale::px,
    theme::{ActiveTheme, Appearance, Theme},
    typography::FONT_FAMILY,
};

type Press = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct ReleaseCard {
    id: ElementId,
    name: SharedString,
    version: SharedString,
    title: SharedString,
    date: Option<SharedString>,
    notes: Vec<ReleaseNote>,
    secondary: SharedString,
    primary: SharedString,
    on_secondary: Option<Press>,
    on_primary: Option<Press>,
}
impl ReleaseCard {
    pub fn new(id: impl Into<ElementId>, name: impl Into<SharedString>, version: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: version.into(),
            title: "What\u{2019}s new".into(),
            date: None,
            notes: Vec::new(),
            secondary: "All releases".into(),
            primary: "Got it".into(),
            on_secondary: None,
            on_primary: None,
        }
    }
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }
    /// The line under the title, as a ready string ("Version 0.1.16 · 10 October 2026").
    pub fn date(mut self, date: Option<SharedString>) -> Self {
        self.date = date;
        self
    }
    /// The notes, in the shape of the changelog sheet's: a bold lead and a muted text, no icon.
    pub fn notes(mut self, notes: impl IntoIterator<Item = ReleaseNote>) -> Self {
        self.notes.extend(notes);
        self
    }
    /// The words of the two buttons at the foot: the quiet one at the left, the main one at the right.
    pub fn buttons(mut self, secondary: impl Into<SharedString>, primary: impl Into<SharedString>) -> Self {
        self.secondary = secondary.into();
        self.primary = primary.into();
        self
    }
    pub fn on_secondary(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_secondary = Some(Rc::new(f));
        self
    }
    pub fn on_primary(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_primary = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for ReleaseCard {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme: Theme = cx.theme().clone();
        // The picture is dark at the top left whatever the theme, so the words over it are the dark theme's.
        let light = Theme::of(Appearance::Dark).foreground;
        let picture = gpui_kit::img(HERO_PATH).w_full().h(px(HERO_HEIGHT)).rounded(px(HERO_CORNER));
        let picture = <gpui_kit::Img as gpui_kit::StyledImage>::object_fit(picture, ObjectFit::Fill);
        let mark = div()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .gap(px(8.))
            .font_family(FONT_FAMILY)
            .text_color(light)
            .child(div().text_size(px(NAME_SIZE)).font_weight(FontWeight::MEDIUM).child(self.name))
            .child(
                div()
                    .px(px(8.))
                    .py(px(1.))
                    .border(px(PILL_BORDER))
                    .border_color(light)
                    .rounded_full()
                    .text_size(px(PILL_SIZE))
                    .font_weight(FontWeight::MEDIUM)
                    .child(self.version),
            );
        let rows = self.notes.into_iter().enumerate().map(|(at, note)| {
            let (lead, text) = note.parts();
            div()
                .pb(px(NOTE_GAP))
                .child(
                    div()
                        .debug_selector(move || format!("release-card-lead-{at}"))
                        .mb(px(LEAD_GAP))
                        .text_size(px(LEAD_SIZE))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.foreground)
                        .child(lead),
                )
                .child(
                    div()
                        .text_size(px(TEXT_SIZE))
                        .line_height(px(TEXT_LINE))
                        .text_color(theme.muted_foreground)
                        .child(text),
                )
        });
        let secondary = self.on_secondary;
        let primary = self.on_primary;
        div()
            .id(self.id)
            .debug_selector(|| "release-card".into())
            .w_full()
            .flex()
            .flex_col()
            .font_family(FONT_FAMILY)
            .child(
                div()
                    .p(px(PAD))
                    .pb_0()
                    .child(div().relative().debug_selector(|| "release-card-hero".into()).child(picture).child(mark)),
            )
            .child(
                div()
                    .px(px(PAD + SIDE))
                    .pt(px(TITLE_TOP))
                    .child(
                        div()
                            .text_size(px(TITLE_SIZE))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.foreground)
                            .child(self.title),
                    )
                    .children(self.date.map(|date| {
                        div().mt(px(DATE_GAP)).text_size(px(DATE_SIZE)).text_color(theme.muted_foreground).child(date)
                    }))
                    .child(div().pt(px(NOTES_TOP)).flex().flex_col().children(rows)),
            )
            .child(
                div()
                    .mt(px(FOOT_TOP))
                    .border_t_1()
                    .border_color(theme.divider)
                    .p(px(PAD))
                    .flex()
                    .gap(px(BUTTON_GAP))
                    .child(
                        div().flex_1().child(
                            Button::new("release-card-secondary")
                                .label(self.secondary)
                                .variant(ButtonVariant::Secondary)
                                .size(ButtonSize::Lg)
                                .debug_name("release-card-secondary")
                                .on_click(move |_, window, cx| {
                                    if let Some(f) = &secondary {
                                        f(window, cx);
                                    }
                                }),
                        ),
                    )
                    .child(
                        div().flex_1().child(
                            Button::new("release-card-primary")
                                .label(self.primary)
                                .trailing_icon(IconName::ArrowForward)
                                .variant(ButtonVariant::Invert)
                                .size(ButtonSize::Lg)
                                .debug_name("release-card-primary")
                                .on_click(move |_, window, cx| {
                                    if let Some(f) = &primary {
                                        f(window, cx);
                                    }
                                }),
                        ),
                    ),
            )
    }
}
