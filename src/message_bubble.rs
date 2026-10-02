//! beui's MessageBubble (`components/agents/message-bubble.tsx`), class for class:
//!
//! - Surface: borderless per the recipe. `solid` fills `foreground` (text flips to `background`); `soft`
//!   and `tint` fill `card`; `borderless` (beui's `outline`) fills `card_strong` in place of a border;
//!   `danger` fills `danger` at 10% (text `danger`); `ghost` draws no surface and drops the padding,
//!   stretching to the full row width.
//! - Shape: compact, as the pull request view's cards are: `rounded-xl` (12), `px-3 py-1.5`, `text-sm` on 20px lines,
//!   capped at 82% width, at least 36px wide. (beui's own is `rounded-2xl px-3.5 py-2.5 leading-6`, which is roomier
//!   than a long chat can afford.)
//! - Alignment: `start` or `end`; the row stacks the bubble on that edge of its column.
//! - Grouped corners: [`message_bubble_group`] stacks consecutive bubbles 4px (compact, the stack gap of the cards above the
//!   composer) or 12px (default) apart, so a speaker's turns read as one column instead of separate cards.
//! - Expandable content: [`MessageBubbleCollapsible`] clips long prose to a line count behind a bottom
//!   fade, with a "Show more/less" pill whose chevron turns on `SPRING_SWAP`.
//! - Entrance: the shared chat [`Entrance`](crate::entrance::Entrance): the whole bubble fades and rises 6px, like every other
//!   chat item. Under Reduce Motion it only fades.

mod helpers;
mod structs;
mod types;

pub use helpers::message_bubble_group;
pub use structs::{MessageBubble, MessageBubbleCollapsible};
pub use types::{
    LINE, MessageBubbleAlign, MessageBubbleGroupSpacing, MessageBubbleVariant, PAD_X, PAD_Y,
};
