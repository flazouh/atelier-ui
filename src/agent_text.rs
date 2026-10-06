//! The agent's prose, married to beui's StreamingResponse (`components/agents/streaming-response.tsx`),
//! class for class. The Markdown body itself is gpui-component's `TextView`, restyled, not rewritten.
//!
//! - Content: `text-sm leading-6 text-foreground/90`, its rhythm from `TextViewStyle`'s paragraph gap
//!   (beui's `my-3`, 12px), matching the response's own list and paragraph spacing.
//! - Completion actions row, once the response is no longer streaming: `gap-0.5` of `size-7` icon
//!   buttons: Copy (flips to a check for 1.6s), Retry, then Helpful / Not helpful once complete, each
//!   `rounded-md` with a muted-to-`card` hover and a persistent `card` fill while active. Fades up 4px
//!   over 220ms, like beui's `AnimatePresence`.
//! - Source summary: a stack of up to three domain-initial chips, the count in tabular nums, and a
//!   chevron that turns on `SPRING_SWAP` to open a `card` panel listing each source's title and domain.
//! - Status: [`AgentTextStatus::Streaming`] (the default, like beui's) hides the row entirely;
//!   `Error` shows Copy/Retry/sources, exactly as
//!   beui draws no separate error style in this component.
//!
//! - Pull requests: with [`AgentText::pr_resolver`], every `#N` the resolver knows shows as a
//!   [`crate::pr_chip::PrChip`]; the rest stay plain text. Finding the pull request is the app's job.
//!
//! Pair this with [`crate::thinking::Thinking`] for the run's busy state before the first token: show
//! `Thinking` while nothing has arrived yet, then swap to an `AgentText` in `Streaming` (silent, no row)
//! and finally `Complete` (with the actions row) once the response settles.

mod helpers;
mod structs;
mod types;

pub use structs::{AgentText, AgentTextSource};
pub use types::AgentTextStatus;
