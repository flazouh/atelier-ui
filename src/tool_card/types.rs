use gpui_kit::SharedString;

use crate::{icon::IconName, task_model::TaskStatus};

/// A list shows this many rows, and a press on "Show more" shows the rest.
pub const ROWS_SHOWN: usize = 10;

/// The most buttons the footer has.
pub const FOOTER_MAX: usize = 3;

/// Where the call stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolCardState {
    Running,
    /// The call waits for the reader to allow it.
    Waiting,
    Done,
    Failed,
}

/// Whose tool it is: a small tile with the provider's first letter on one colour of the project palette, the
/// provider's name, and the account when the call names one. A provider that is not known has a neutral tile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolProvider {
    pub name: SharedString,
    pub account: Option<SharedString>,
    pub letter: SharedString,
    /// An index into the project palette ([`crate::project_badge::COUNT`] colours), or `None` for the neutral tile.
    pub color: Option<usize>,
}

/// A colour by meaning. The theme maps it to a colour.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolTone {
    #[default]
    Neutral,
    Info,
    Success,
    Warning,
    Danger,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolText {
    #[default]
    Body,
    Title,
    Muted,
    Code,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolGap {
    Xs,
    #[default]
    Sm,
    Md,
}

/// A mark in a row: one of the icon set, or the mark of a task's status, as the Tasks list draws it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolIcon {
    Named(IconName),
    Status(TaskStatus),
}

/// A button, and the word the app gets back when it is pressed. The card does nothing else with the key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolAction {
    pub label: SharedString,
    pub key: SharedString,
}

/// A row of a list, and what a press on it asks for.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolRow {
    pub node: ToolNode,
    pub action: Option<SharedString>,
}

/// What the body of a card is made of. Every text is plain text: the card never reads markup.
#[derive(Clone, Debug, PartialEq)]
pub enum ToolNode {
    Stack {
        row: bool,
        gap: ToolGap,
        children: Vec<ToolNode>,
    },
    Text {
        value: SharedString,
        style: ToolText,
        max_lines: Option<u8>,
    },
    Badge {
        value: SharedString,
        tone: ToolTone,
    },
    Metric {
        label: SharedString,
        value: SharedString,
    },
    Icon {
        icon: ToolIcon,
        tone: ToolTone,
    },
    Avatar {
        name: SharedString,
    },
    Code {
        value: SharedString,
        max_lines: Option<u8>,
    },
    /// `hidden` rows were left out by the app, and the card says so.
    List {
        rows: Vec<ToolRow>,
        hidden: usize,
        empty: Option<SharedString>,
    },
    Button {
        action: ToolAction,
    },
    Divider,
}

/// All a card shows.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolCardData {
    pub provider: ToolProvider,
    /// The summary line: "Searched Linear, 3 tasks".
    pub title: SharedString,
    pub state: ToolCardState,
    /// Who made the call, when an agent did: "Alex's agent".
    pub origin: Option<SharedString>,
    pub body: Option<ToolNode>,
    pub footer: Vec<ToolAction>,
}
