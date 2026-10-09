use gpui_kit::Hsla;

use crate::{
    theme::{Theme, mix},
    update_button::{
        consts::{TRACK, WASH},
        enums::UpdateState,
    },
};

/// The colours of one state.
pub(in crate::update_button) struct Look {
    pub(in crate::update_button) fill: Hsla,
    pub(in crate::update_button) text: Hsla,
    pub(in crate::update_button) arc: Hsla,
    pub(in crate::update_button) track: Hsla,
}

/// The colours of `state` at hover level `hover` (only a ready button reacts to it).
pub(in crate::update_button) fn look(state: UpdateState, theme: &Theme, hover: f32) -> Look {
    match state {
        UpdateState::Ready => Look {
            fill: mix(theme.primary, theme.primary_hover(), hover),
            text: theme.primary_foreground,
            arc: theme.primary_foreground,
            track: theme.primary_foreground.opacity(TRACK),
        },
        UpdateState::Downloading(_) | UpdateState::Restarting => Look {
            fill: theme.foreground.opacity(WASH),
            text: theme.foreground,
            arc: theme.foreground,
            track: theme.foreground.opacity(TRACK),
        },
    }
}
