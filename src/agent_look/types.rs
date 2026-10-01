use crate::sprite::Strip;

/// A one-frame mark: Material's `smart_toy`, which holds still.
pub(super) const NEUTRAL_STRIP: Strip = Strip {
    path: "icons/smart_toy.svg",
    bytes: include_bytes!("../../assets/icons/smart_toy.svg"),
    frames: 1,
    frame_ms: 1000,
    loops: false,
};
