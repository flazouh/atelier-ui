use gpui_kit::{
    App, ElementId, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, div,
};

use super::helpers::{colors, icon_of, pulse_at};
use super::types::{BadgeSize, BadgeStatus};
use crate::scale::px;
use crate::{
    icon::Icon,
    motion::now_millis,
    roll::{Kind, Roll},
    theme::ActiveTheme,
};

#[derive(IntoElement)]
pub struct AnimatedBadge {
    id: ElementId,
    pub(super) status: BadgeStatus,
    size: BadgeSize,
    label: Option<SharedString>,
    show_icon: bool,
    selector: Option<&'static str>,
}

impl AnimatedBadge {
    pub fn new(id: impl Into<ElementId>, status: BadgeStatus) -> Self {
        Self {
            id: id.into(),
            status,
            size: BadgeSize::default(),
            label: None,
            show_icon: true,
            selector: None,
        }
    }

    pub fn size(mut self, size: BadgeSize) -> Self {
        self.size = size;
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn show_icon(mut self, show: bool) -> Self {
        self.show_icon = show;
        self
    }

    pub fn debug_name(mut self, name: &'static str) -> Self {
        self.selector = Some(name);
        self
    }
}

impl RenderOnce for AnimatedBadge {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let reduce = cx.reduce_motion();
        let (height, pad, gap, text, icon) = self.size.metrics();
        let (ink, fill) = colors(self.status, &theme);
        let loading = self.status == BadgeStatus::Loading;
        let millis = now_millis();
        if loading && !reduce {
            window.request_animation_frame();
        }
        let wash = (loading && !reduce).then(|| {
            div()
                .absolute()
                .inset_0()
                .rounded_full()
                .bg(ink.opacity(pulse_at(millis)))
        });
        let turn = (loading && !reduce).then(|| (millis % 1000) as f32 / 1000.);
        let mark = self.show_icon.then(|| {
            Roll::new(
                (self.id.clone(), "icon"),
                self.status,
                Kind::Icon,
                px(icon),
                move |status: &BadgeStatus| {
                    let mut icon = Icon::new(icon_of(*status)).size(px(icon)).color(ink);
                    if let (BadgeStatus::Loading, Some(t)) = (status, turn) {
                        icon = icon.turn(t);
                    }
                    icon.into_any_element()
                },
            )
        });
        let words = self.label.map(|label| {
            Roll::new(
                (self.id.clone(), "words"),
                label,
                Kind::Words,
                px(16.),
                move |label: &SharedString| {
                    div()
                        .h(px(16.))
                        .line_height(px(16.))
                        .whitespace_nowrap()
                        .child(label.clone())
                        .into_any_element()
                },
            )
        });
        div()
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(gap))
            .h(px(height))
            .px(px(pad))
            .overflow_hidden()
            .rounded_full()
            .bg(fill)
            .text_color(ink)
            .text_size(px(text))
            .font_weight(FontWeight::MEDIUM)
            .whitespace_nowrap()
            .children(wash)
            .children(mark)
            .children(words)
    }
}
