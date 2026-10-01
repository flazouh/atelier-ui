use gpui_kit::{
    AnyElement,
    App,
    IntoElement,
    ParentElement,
    RenderOnce,
    SharedString,
    Styled,
    Window,
    div,
    prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    icon::{Icon, IconName},
    theme::{ActiveTheme, radius},
    typography::TextSize,
};
use super::types::SectionTone;

#[derive(IntoElement)]
pub struct RailSection {
    pub(super) name: SharedString,
    pub(super) icon: Option<IconName>,
    pub(super) summary: Option<AnyElement>,
    pub(super) tone: SectionTone,
    pub(super) body: Vec<AnyElement>,
}

impl RailSection {
    pub fn new(name: impl Into<SharedString>) -> Self {
        Self { name: name.into(), icon: None, summary: None, tone: SectionTone::Plain, body: Vec::new() }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn summary(mut self, summary: impl IntoElement) -> Self {
        self.summary = Some(summary.into_any_element());
        self
    }

    pub fn tone(mut self, tone: SectionTone) -> Self {
        self.tone = tone;
        self
    }
}

impl ParentElement for RailSection {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.body.extend(elements);
    }
}

impl RenderOnce for RailSection {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let tone = match self.tone {
            SectionTone::Plain => theme.muted_foreground,
            SectionTone::Bad => theme.danger,
            SectionTone::Done => theme.success,
            SectionTone::Attention => theme.warning,
        };
        div()
            .flex()
            .flex_col()
            // A card keeps its own height in a scrolling rail.
            .flex_none()
            .rounded(radius::lg())
            .bg(theme.card)
            .overflow_hidden()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .h(px(36.))
                    .px(px(12.))
                    .text_size(TextSize::Xs.font_size())
                    .whitespace_nowrap()
                    .when_some(self.icon, |d, icon| d.child(Icon::new(icon).size(px(14.)).color(theme.muted_foreground)))
                    .child(div().flex_none().font_weight(gpui_kit::FontWeight::SEMIBOLD).text_color(theme.foreground.opacity(0.9)).child(self.name))
                    .when_some(self.summary, |d, summary| d.child(div().flex().items_center().min_w_0().truncate().text_color(tone).child(summary))),
            )
            .children(self.body)
    }
}
