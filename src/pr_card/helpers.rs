use gpui_kit::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, Window, div,
};

use crate::scale::px;
use crate::{
    copy_feedback::CopyFeedback,
    focus::PressStop,
    icon::{Icon, IconName},
    theme::{Theme, radius},
    tooltip::Tooltip,
    };
use super::structs::LinkActions;

/// A 24px icon button with a tooltip, for Copy link and Open in browser. It stops the press here, so the
/// row or chip under it does not open too.
pub(crate) fn icon_action(
    id: ElementId,
    icon: IconName,
    tip: &'static str,
    theme: &Theme,
    on_press: impl Fn(&mut Window, &mut App) + 'static,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let key = ElementId::NamedChild(std::sync::Arc::new(id.clone()), "focus".into());
    div()
        .id(id)
        .flex()
        .flex_none()
        .size(px(24.))
        .items_center()
        .justify_center()
        .rounded(radius::md())
        .cursor_pointer()
        .text_color(theme.muted_foreground)
        .hover(|s| s.bg(theme.muted_hover()).text_color(theme.foreground))
        .tooltip(Tooltip::text(tip))
        .press_stop(key, radius::md(), window, cx)
        .on_mouse_down(gpui_kit::MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .on_click(move |_, window, cx| {
            cx.stop_propagation();
            on_press(window, cx);
        })
        .child(Icon::new(icon).size(px(13.)))
}

/// Copy link, which flips to a check for a moment, then Open in browser.
pub(crate) fn link_actions(
    id: &ElementId,
    url: &SharedString,
    state: &gpui_kit::Entity<LinkActions>,
    theme: &Theme,
    window: &mut Window,
    cx: &mut App,
) -> [gpui_kit::AnyElement; 2] {
    let child = |name: &'static str| ElementId::NamedChild(std::sync::Arc::new(id.clone()), name.into());
    let copied = state.read(cx).copy.copied();
    let (copy_url, open_url, copy_state) = (url.to_string(), url.to_string(), state.clone());
    [
        icon_action(child("copy"), if copied { IconName::Check } else { IconName::Copy }, "Copy link", theme, move |_, cx| {
            CopyFeedback::click(&copy_state, |s: &mut LinkActions| &mut s.copy, copy_url.clone(), cx)
        }, window, cx)
        .into_any_element(),
        icon_action(child("open"), IconName::OpenInNew, "Open in browser", theme, move |_, cx| cx.open_url(&open_url), window, cx)
            .into_any_element(),
    ]
}
