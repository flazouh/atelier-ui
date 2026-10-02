use std::sync::Arc;

use gpui_kit::{App, Window};

use crate::pr::PrChipData;

/// The link scheme [`link_prs`](crate::pr_chip::link_prs) writes and [`PrChips`](crate::pr_chip::PrChips) reads.
pub(super) const SCHEME: &str = "atelier-pr:";

/// The pill's height, and where its text sits, so the pill lines up with the words around it.
pub(super) const PILL_HEIGHT: f32 = 20.;

pub(super) const PILL_BASELINE: f32 = 14.;

/// The hover card's widest, so the title wraps at a comfortable measure.
pub(super) const CARD_MAX_WIDTH: f32 = 320.;

/// A callback for a pressed chip. It must be `Send + Sync` because Markdown renderers are.
pub type PrOpenHandler = Arc<dyn Fn(&PrChipData, &mut Window, &mut App) + Send + Sync>;
