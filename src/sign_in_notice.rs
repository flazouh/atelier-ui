//! The box over the composer when an agent has no sign-in: which agent and account, one button that signs in, and
//! room beside it for another way on, such as a handoff. It says what to do instead of the agent's own "run /login",
//! which a headless agent cannot.

mod helpers;
mod structs;
mod types;

pub use helpers::words;
pub use structs::SignInNotice;
pub use types::SignInState;

#[cfg(test)]
mod tests;
