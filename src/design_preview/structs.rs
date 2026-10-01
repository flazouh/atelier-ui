use gpui_kit::Global;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Choice {
    pub(super) tabs: usize,
}

impl Global for Choice {}
