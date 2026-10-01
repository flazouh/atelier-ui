//! The keys of the task list and board. GitQuiet's table (`keys.rs`) has no commands for tasks, so these
//! are atelier's own, in Linear's model: `j` and `k` or the arrows to move, `x` to select, Enter to open, and
//! a letter for each field to change on the selected tasks. A bare letter is never read while a text input
//! has focus (see [`crate::keys::typing`]); the commands with a modifier are always read.
use crate::keys::Press;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TaskCommand {
    Down,
    Up,
    First,
    Last,
    /// `x`: select or unselect the task under the cursor.
    Select,
    /// ⌘A: select every task shown.
    SelectAll,
    /// Esc: clear the selection.
    Clear,
    Open,
    /// `s`: change the status of the selected tasks.
    SetStatus,
    /// `p`
    SetPriority,
    /// `a`
    SetAssignee,
    /// `i`: assign to yourself.
    AssignToMe,
    /// `l`
    SetLabels,
    /// `c`: a new task.
    NewTask,
    /// `f`: filter.
    Filter,
    /// `←` and `→` on a group header.
    Fold,
    Unfold,
    /// `]`: move the task to the next status.
    MoveNext,
    /// `[`: move the task to the previous status.
    MovePrev,
}

impl TaskCommand {
    pub const ALL: &'static [TaskCommand] = &[
        Self::Down,
        Self::Up,
        Self::First,
        Self::Last,
        Self::Select,
        Self::SelectAll,
        Self::Clear,
        Self::Open,
        Self::SetStatus,
        Self::SetPriority,
        Self::SetAssignee,
        Self::AssignToMe,
        Self::SetLabels,
        Self::NewTask,
        Self::Filter,
        Self::Fold,
        Self::Unfold,
        Self::MoveNext,
        Self::MovePrev,
    ];

    /// What the command does, in a few words.
    pub fn words(self) -> &'static str {
        match self {
            Self::Down => "Next task",
            Self::Up => "Previous task",
            Self::First => "First task",
            Self::Last => "Last task",
            Self::Select => "Select",
            Self::SelectAll => "Select all",
            Self::Clear => "Clear the selection",
            Self::Open => "Open",
            Self::SetStatus => "Set status",
            Self::SetPriority => "Set priority",
            Self::SetAssignee => "Set assignee",
            Self::AssignToMe => "Assign to me",
            Self::SetLabels => "Set labels",
            Self::NewTask => "New task",
            Self::Filter => "Filter",
            Self::Fold => "Fold the group",
            Self::Unfold => "Unfold the group",
            Self::MoveNext => "Next status",
            Self::MovePrev => "Previous status",
        }
    }

    /// The chord in the notation of [`crate::keys::cap`], for a control's cap.
    pub fn chord(self) -> &'static str {
        match self {
            Self::Down => "j",
            Self::Up => "k",
            Self::First => "g",
            Self::Last => "G",
            Self::Select => "x",
            Self::SelectAll => "⌘a",
            Self::Clear => "Escape",
            Self::Open => "Enter",
            Self::SetStatus => "s",
            Self::SetPriority => "p",
            Self::SetAssignee => "a",
            Self::AssignToMe => "i",
            Self::SetLabels => "l",
            Self::NewTask => "c",
            Self::Filter => "f",
            Self::Fold => "←",
            Self::Unfold => "→",
            Self::MoveNext => "]",
            Self::MovePrev => "[",
        }
    }
}

/// The command a press asks for, if any. `typing` is whether a text input has focus: then only a press
/// with a modifier, and Escape, can be a command.
pub fn read(press: &Press, typing: bool) -> Option<TaskCommand> {
    use TaskCommand::*;
    if press.secondary {
        return (press.key == "a" && !press.shift && !press.alt).then_some(SelectAll);
    }
    if press.alt {
        return None;
    }
    match press.key.as_str() {
        "escape" | "Escape" => return Some(Clear),
        "down" => return Some(Down),
        "up" => return Some(Up),
        "left" => return Some(Fold),
        "right" => return Some(Unfold),
        "enter" | "Enter" if !typing => return Some(Open),
        "home" => return Some(First),
        "end" => return Some(Last),
        _ => {}
    }
    if typing {
        return None;
    }
    Some(match (press.key.as_str(), press.shift) {
        ("j", false) => Down,
        ("k", false) => Up,
        ("g", false) => First,
        ("g", true) | ("G", _) => Last,
        ("x", false) => Select,
        ("s", false) => SetStatus,
        ("p", false) => SetPriority,
        ("a", false) => SetAssignee,
        ("i", false) => AssignToMe,
        ("l", false) => SetLabels,
        ("c", false) => NewTask,
        ("f", false) => Filter,
        ("]", _) => MoveNext,
        ("[", _) => MovePrev,
        _ => return None,
    })
}

#[cfg(test)]
mod tests;
