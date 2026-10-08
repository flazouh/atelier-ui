//! UsageDashboard: the body of the usage modal. How many tokens went where: a tile for each account and key, a stacked
//! chart of the days, the models, and the sessions, each of which opens to its own days. It is stateless: the app holds
//! the range, the selection and the open session, and gives new data when a press asks for it.
//!
//! - Put it in a [`Modal`](crate::modal::Modal); the modal owns the close.
//! - Hue and shade of a [`Series`] come from the theme's chart palette ([`Theme::series`](crate::theme::Theme::series)).
//! - In a stacked bar the segments are square and touch; only the top one has rounded top corners. A one-colour mini
//!   chart has rounded top corners only.
//! - Tiles, group captions, the range control and the session rows are in the Tab order; Enter or Space presses them.
mod consts;
mod enums;
mod helpers;
mod impls;
mod structs;
pub use enums::{Selection, SourceKind, UsageRange};
pub use helpers::{axis, bar_segments, mini_bars, spark_points, tokens_label};
pub use structs::{
    Axis, BarSegment, MiniBar, Series, UsageDashboard, UsageDay, UsageModel, UsageSession, UsageSource,
};
#[cfg(test)]
mod tests;
