use crate::sprite::Strip;

/// A one-frame mark: Hugeicons' `bot`, which holds still.
pub(super) const NEUTRAL_STRIP: Strip = Strip {
    path: "icons/bot.svg",
    bytes: include_bytes!("../../assets/icons/bot.svg"),
    frames: 1,
    frame_ms: 1000,
    loops: false,
};
