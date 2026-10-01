//! Merging a pull request, as a pure model: what the repository allows, what holds a merge up and in
//! which order, and what the merge button offers. The app hands the facts over from the forge; nothing
//! here asks anything. The words follow GitQuiet's (`src/ui/Ask.tsx`, `src/ui/Merge.tsx`): a merge that
//! cannot happen says why rather than hiding, and the button says what a press would do.
//!
//! Merging has no keyboard shortcut, on purpose: a merge is hard to undo, and a key would make an
//! accidental one easy. The merge button is still reached and driven from the keyboard (Tab to it,
//! then its menu with Enter, Space or Down), so no action needs the pointer.

mod helpers;
mod structs;
mod types;

pub use helpers::{blockers, button, first_choice, offered, pick_method, standing};
pub use structs::{ButtonState, Choice, MergeFacts, Queue};
pub use types::{Action, Blocker, MergeMethod, PullState, ReviewNeed, Rights, UpdateWay};

#[cfg(test)]
mod tests;
