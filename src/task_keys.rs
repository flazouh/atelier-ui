//! The keys of the task list and board. GitQuiet's table (`keys.rs`) has no commands for tasks, so these
//! are atelier's own, in Linear's model: `j` and `k` or the arrows to move, `x` to select, Enter to open, and
//! a letter for each field to change on the selected tasks. A bare letter is never read while a text input
//! has focus (see [`crate::keys::typing`]); the commands with a modifier are always read.

mod helpers;
mod types;

pub use helpers::read;
pub use types::TaskCommand;

#[cfg(test)]
mod tests;
