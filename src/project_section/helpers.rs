use gpui_kit::{IntoElement, ParentElement, SharedString, Styled, div};

use crate::scale::px;
use crate::{sidebar_model::Connection, typography::MONO_FONT_FAMILY};
use super::types::MenuChoice;

/// The words for how a project is connected: `None` when nothing needs saying.
pub fn connection_words(connection: Connection) -> Option<&'static str> {
    match connection {
        Connection::Connected => None,
        Connection::Connecting => Some("Connecting…"),
        Connection::Reconnecting => Some("Reconnecting…"),
        Connection::Offline => Some("Offline"),
    }
}

/// Why `choice` cannot be chosen now, or `None` when it can: pull requests need a forge remote.
pub fn unavailable(choice: MenuChoice, pulls: Option<&str>) -> Option<&str> {
    (choice == MenuChoice::PullRequests).then_some(pulls).flatten()
}

pub(super) fn chip(text: SharedString, theme: &crate::theme::Theme) -> impl IntoElement {
    div()
        .flex_none()
        .px(px(6.))
        .h(px(18.))
        .flex()
        .items_center()
        .rounded_full()
        .bg(theme.card_strong)
        .font_family(MONO_FONT_FAMILY)
        .text_size(px(10.))
        .text_color(theme.muted_foreground)
        .child(text)
}
