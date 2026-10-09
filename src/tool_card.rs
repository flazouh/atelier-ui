//! A tool call of an agent as a card, in the chat in place of a plain tool row: the provider's tile and name, the account, one
//! line that says what the call did, how it stands (running, waiting for approval, done, failed), who made it when an agent
//! did, and what it returned as rows. The app turns a result into the tree of [`ToolNode`]s; this module only draws it, and
//! draws every text as plain text. A list shows [`ROWS_SHOWN`] rows and a press on "Show more" shows the rest.
mod helpers;
mod structs;
mod types;
pub use structs::{ActionHandler, ToolCard};
pub use types::{
    FOOTER_MAX, ROWS_SHOWN, ToolAction, ToolCardData, ToolCardState, ToolGap, ToolIcon, ToolNode,
    ToolProvider, ToolRow, ToolText, ToolTone,
};
#[cfg(test)]
mod tests;
