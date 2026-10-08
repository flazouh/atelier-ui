use std::rc::Rc;
use gpui_kit::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, ObjectFit, ParentElement, RenderOnce,
    SharedString, Styled, Window, div,
};
use crate::scale::px;
use crate::{
    Button, ButtonVariant, Icon, IconName,
    theme::{ActiveTheme, Appearance, Theme},
};
use super::{
    consts::{CORNER, HERO_HEIGHT, HERO_PATH, PANEL_ALPHA, PANEL_CORNER, PANEL_GAP, SIDE},
    helpers::fade,
};
type Choice = Rc<dyn Fn(&mut Window, &mut App)>;

/// One line of what is new: a short lead and what it means.
#[derive(Clone)]
pub struct ReleaseNote {
    lead: SharedString,
    text: SharedString,
    icon: IconName,
}
impl ReleaseNote {
    pub fn new(lead: impl Into<SharedString>, text: impl Into<SharedString>) -> Self {
        Self { lead: lead.into(), text: text.into(), icon: IconName::Check }
    }
    /// The mark in the tile at the line's left (a tick by default).
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = icon;
        self
    }
}

#[derive(IntoElement)]
pub struct ReleaseSheet {
    id: ElementId,
    version: SharedString,
    kicker: SharedString,
    notes: Vec<ReleaseNote>,
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
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme: Theme = cx.theme().clone();
        // The picture is dark at the top left whatever the theme, so the words over it are the dark theme's.
        let light = Theme::of(Appearance::Dark).foreground;
        let picture = div()
            .absolute()
            .inset_0()
            .child(
                <gpui_kit::Img as gpui_kit::StyledImage>::object_fit(gpui_kit::img(HERO_PATH).size_full(), ObjectFit::Cover)
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

        let rows = self.notes.into_iter().map(|note| {
            div()
                    .flex()
                    .gap(px(14.))
                    .py(px(9.))
                    .child(
                        div()
                            .flex()
                            .flex_none()
                            .items_center()
                            .justify_center()
                            .size(px(34.))
                            .rounded(px(9.))
                            .bg(theme.card_strong)
                            .child(Icon::new(note.icon).size(px(16.)).color(theme.muted_foreground)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w_0()
                            .gap(px(2.))
                            .child(div().text_size(px(15.)).font_weight(FontWeight::MEDIUM).text_color(theme.foreground).child(note.lead))
                            .child(div().text_size(px(14.)).line_height(px(20.)).text_color(theme.muted_foreground).child(note.text)),
                    )
        });

        let (later, install, install_label) = (self.on_later, self.on_install, self.install);
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
            .child(div().px(px(18.)).py(px(10.)).flex().flex_col().children(rows))
            .child(foot);

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
    }
}
