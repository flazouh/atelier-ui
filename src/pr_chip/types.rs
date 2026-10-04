use std::{sync::Arc, time::Duration};

use gpui_kit::{App, Window};

use crate::pr::PrChipData;

/// The link scheme [`link_prs`](crate::pr_chip::link_prs) writes and [`PrChips`](crate::pr_chip::PrChips) reads.
pub(super) const SCHEME: &str = "atelier-pr:";

/// The pill's height, and where its text sits, so the pill lines up with the words around it.
pub(super) const PILL_HEIGHT: f32 = 20.;

pub(super) const PILL_BASELINE: f32 = 14.;

/// The widest the title gets in the pill, so a long one does not take the line.
pub(super) const PILL_TITLE_WIDTH: f32 = 200.;

/// The card's width: fixed, so the card does not jump from one pull request to the next.
pub(super) const CARD_WIDTH: f32 = 340.;

/// How long the pointer rests on a pill before its card opens: long enough that crossing a line of text opens none.
pub(super) const OPEN_DELAY: Duration = Duration::from_millis(120);

pub(super) const CLOSE_DELAY: Duration = Duration::from_millis(120);

/// A card that closed this recently leaves the next one to open at once, as the pointer goes from chip to chip.
pub(super) const WARM_FOR: Duration = Duration::from_millis(600);

/// The squares of the size bar, as GitHub draws them.
pub(super) const SIZE_SQUARES: u32 = 5;

/// A callback for a pressed chip. It must be `Send + Sync` because Markdown renderers are.
pub type PrOpenHandler = Arc<dyn Fn(&PrChipData, &mut Window, &mut App) + Send + Sync>;
