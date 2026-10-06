//! The agent asks the reader a question with choices, and the card is where it is answered. It follows beui's Approval Card in
//! its question mode: one question at a time with its options as radio rows (a single choice) or check rows (several), an
//! "other" line for words of the reader's own, a step counter and dots when there are several questions, and Submit at the
//! last.
//!
//! The card reads whatever has come, so it shows a question while the agent is still writing it: [`QuestionStatus::Streaming`]
//! lists the questions and options so far, with no way to answer; [`QuestionStatus::Pending`] is the same questions to be
//! answered; [`QuestionStatus::Answered`] folds the card into what the reader said.
mod helpers;
mod structs;
mod types;
pub use structs::QuestionCard;
pub use types::{QuestionStatus, QuestionView};
#[cfg(test)]
mod tests;
