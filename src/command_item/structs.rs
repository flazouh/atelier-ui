use gpui_kit::SharedString;

use super::types::CommandSource;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandItem {
    /// Without the slash.
    pub name: SharedString,
    pub source: CommandSource,
    pub summary: SharedString,
    /// What goes after the name. A command with one keeps the list closed and waits for its words.
    pub args_hint: Option<SharedString>,
}
