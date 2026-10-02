use gpui_kit::{FontWeight, IntoElement, ParentElement, Styled, div, prelude::FluentBuilder};

use crate::scale::px;
use crate::{
    icon::{Icon, IconName},
    sidebar_model::Location,
    typography::{MONO_FONT_FAMILY, TextSize},
};
use super::types::GROUP_HEADER;

/// A project's header over its group: the project's name and where it lives.
pub fn group_header(project: &crate::panel_types::ProjectLabel, theme: &crate::theme::Theme) -> impl IntoElement {
    let (icon, host) = match &project.location {
        Location::Local => (IconName::Folder, None),
        Location::Ssh { host } => (IconName::Dns, Some(host.clone())),
    };
    div()
        .flex()
        .items_center()
        .gap(px(6.))
        .h(px(GROUP_HEADER))
        .px(px(4.))
        .text_size(TextSize::Xs.font_size())
        .text_color(theme.muted_foreground)
        .child(Icon::new(icon).size(px(13.)))
        .child(div().font_weight(FontWeight::MEDIUM).text_color(theme.foreground).child(project.name.clone()))
        .when_some(host, |d, host| {
            d.child(
                div()
                    .px(px(6.))
                    .h(px(16.))
                    .flex()
                    .items_center()
                    .rounded_full()
                    .bg(theme.card_strong)
                    .font_family(MONO_FONT_FAMILY)
                    .text_size(px(10.))
                    .child(host),
            )
        })
}
