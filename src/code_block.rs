//! beui's CodeBlock (`components/agents/code-block.tsx`), class for class:
//!
//! - Header `h-10 gap-2.5 px-3`: an optional filename `text-xs` at 80% foreground, the language in
//!   `text-[10px]` uppercase, a status pill (`Writing`/info while streaming, `Ready`/success once
//!   complete) pushed to the right, then a `size-7` Copy button.
//! - Body: a scrollable `text-foreground/85` mono block with a `2.75rem` line-number gutter and an
//!   optional `bg-info/[0.07]` tint on `highlight_lines`. beui divides header from body with a hairline
//!   border; borderless here uses `card_strong` under the body instead (theme.rs's tonal-fill rule).
//! - Syntax colours as in the editor ([`crate::syntax`]), from the language tag when it names a grammar,
//!   else from the file name. The code is highlighted as one text, off the UI thread, and cached.

mod helpers;
mod structs;
mod types;

pub use structs::CodeBlock;
pub use types::CodeBlockStatus;

#[cfg(test)]
use helpers::{gutter_numbers, whole_text_runs};

#[cfg(test)]
mod tests;
