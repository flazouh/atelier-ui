//! beui's MessageBubble (`components/agents/message-bubble.tsx`), class for class:
//!
//! - Surface: borderless per the recipe. `solid` fills `foreground` (text flips to `background`); `soft`
//!   and `tint` fill `card`; `borderless` (beui's `outline`) fills `card_strong` in place of a border;
//!   `danger` fills `danger` at 10% (text `danger`); `ghost` draws no surface and drops the padding,
//!   stretching to the full row width.
//! - Shape: `rounded-2xl`, `px-3.5 py-2.5`, `text-sm leading-6`, capped at 82% width, at least 36px wide.
//! - Alignment: `start` or `end`; the row stacks the bubble on that edge of its column.
//! - Grouped corners: [`message_bubble_group`] stacks consecutive bubbles `gap-1.5` (compact) or
//!   `gap-3` (default) apart, so a speaker's turns read as one column instead of separate cards.
//! - Expandable content: [`MessageBubbleCollapsible`] clips long prose to a line count behind a bottom
//!   fade, with a "Show more/less" pill whose chevron turns on `SPRING_SWAP`.
//! - Entrance: the shared chat [`Entrance`]: the whole bubble fades and rises 6px, like every other
//!   chat item. Under Reduce Motion it only fades.

mod helpers;
mod structs;
mod types;

pub use helpers::message_bubble_group;
pub use structs::{MessageBubble, MessageBubbleCollapsible};
pub use types::{MessageBubbleAlign, MessageBubbleGroupSpacing, MessageBubbleVariant};
