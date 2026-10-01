//! Where an agent session stands, as the sidebar and the tabs say it. The status is data: the app sets it
//! from the session's events, and nothing here knows what an agent is.

mod helpers;
mod types;

pub use helpers::short_reason;
pub use types::{Mark, Need, SessionStatus};

#[cfg(test)]
mod tests;
