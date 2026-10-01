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
