use gpui_kit::{
    AnyElement, Context, ElementId, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, WeakEntity, div,
};

use crate::{
    icon::{Icon, IconName},
    scale::px,
    theme::{ActiveTheme, Theme, radius},
    tooltip::Tooltip,
    typography::TextSize,
};
use super::structs::PromptInput;
use super::types::{PromptInputEvent, QUEUED_GAP, QUEUED_ROW};

impl PromptInput {
    /// The messages waiting for the turn to end, oldest first, each with a button that sends it now and
    /// one that takes it out.
    pub(super) fn queued_rows(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if self.queued.is_empty() {
            return None;
        }
        let theme = cx.theme().clone();
        let (ink, chip_bg) = (super::helpers::chip_ink(&theme), theme.chip_rest);
        let quiet = crate::theme::mix(ink, chip_bg, 0.4);
        let this = cx.entity().downgrade();
        let rows = self.queued.iter().enumerate().map(|(place, text)| {
            let first_line: SharedString = text.lines().next().unwrap_or_default().to_string().into();
            div()
                .id(ElementId::NamedInteger("queued-row".into(), place as u64))
                .debug_selector(move || format!("queued-row-{place}"))
                .flex()
                .items_center()
                .gap(px(6.))
                .h(px(QUEUED_ROW))
                .pl(px(8.))
                .pr(px(2.))
                .rounded(radius::md())
                .bg(theme.chip_rest)
                .text_size(TextSize::Xs.font_size())
                .text_color(quiet)
                .tooltip(Tooltip::text(text.clone()))
                .child(Icon::new(IconName::Schedule).size(px(12.)))
                .child(div().flex_1().min_w_0().truncate().text_color(ink).child(first_line))
                .child(row_button("queued-send", place, IconName::ArrowUp, "Send now", &theme, &this, PromptInputEvent::SendQueued(place)))
                .child(row_button("queued-remove", place, IconName::Close, "Remove", &theme, &this, PromptInputEvent::Unqueue(place)))
        });
        Some(div().flex().flex_col().gap(px(QUEUED_GAP)).px(px(2.)).pb(px(6.)).children(rows).into_any_element())
    }
}

fn row_button(
    name: &'static str,
    place: usize,
    icon: IconName,
    words: &'static str,
    theme: &Theme,
    owner: &WeakEntity<PromptInput>,
    event: PromptInputEvent,
) -> impl IntoElement {
    let owner = owner.clone();
    let (ink, rest) = (super::helpers::chip_ink(theme), theme.chip_rest);
    div()
        .id(ElementId::NamedInteger(name.into(), place as u64))
        .debug_selector(move || format!("{name}-{place}"))
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(px(22.))
        .rounded(px(4.))
        .text_color(crate::theme::mix(ink, rest, 0.4))
        .hover(move |d| d.bg(crate::theme::mix(rest, ink, 0.2)).text_color(ink))
        .tooltip(Tooltip::text(words))
        .child(Icon::new(icon).size(px(12.)))
        .on_mouse_down(gpui_kit::MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .on_click(move |_, _, cx| {
            cx.stop_propagation();
            owner.update(cx, |_, cx| cx.emit(event.clone())).ok();
        })
}
