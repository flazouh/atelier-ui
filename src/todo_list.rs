//! beui's TodoList (`components/agents/todo-list.tsx`), class for class:
//!
//! - Section `rounded-2xl`, borderless here: a `card` fill instead of `border-border/70`.
//! - Header `h-11 gap-2.5 px-3.5`: `size-6` icon slot with `ListTodo` `size-4`, title `text-sm font-medium
//!   text-foreground/90`, count `text-xs font-medium tabular-nums`, chevron `size-3.5` at 50% muted that turns
//!   on `SPRING_SWAP`. When every step is done the icon becomes a filled green check.
//! - Body `px-2 pb-2`, rows `min-h-9 gap-2.5 rounded-xl px-1.5 py-1`: a `size-5` status mark, the title at
//!   `text-sm leading-5`, and an optional detail at 55% muted. Done titles get a strike that grows from the
//!   left over 280ms after 60ms.
//! - The body reveals like `AgentDisclosure`: opacity and a 4px rise over 220ms (140ms to close).
//! - The list closes by itself when every step is done, and opens again when one is not.

mod helpers;
mod structs;
mod types;

pub use helpers::progress;
pub use structs::{Todo, TodoList};
pub use types::TodoStatus;

#[cfg(test)]
use helpers::plan_rows;
#[cfg(test)]
use types::RowPlan;

#[cfg(test)]
mod tests;
