use std::time::{Duration, Instant};

/// Everything the keyboard can ask for. The profile changes which keys reach a command, never what it
/// does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Command {
    GoToFile,
    GoToName,
    FileNames,
    Uses,
    NextFile,
    PreviousFile,
    MarkFile,
    ReviewMode,
    OpenAside,
    Search,
    Dismiss,
    ToggleDetails,
    ToggleFiles,
    WorkingSet,
    Repositories,
    Activity,
    Home,
    /// The project's tasks.
    GoToTasks,
    /// Commit what the review kept.
    Commit,
    Push,
    /// Open a pull request for the branch.
    OpenPull,
    /// A comment on the line the caret is on.
    Comment,
    /// Bring back the open file as it was before its last decision.
    UndoDecision,
}

impl Command {
    pub const ALL: &[Command] = &[
        Self::GoToFile,
        Self::GoToName,
        Self::FileNames,
        Self::Uses,
        Self::NextFile,
        Self::PreviousFile,
        Self::MarkFile,
        Self::ReviewMode,
        Self::OpenAside,
        Self::Search,
        Self::Dismiss,
        Self::ToggleDetails,
        Self::ToggleFiles,
        Self::WorkingSet,
        Self::Repositories,
        Self::Activity,
        Self::Home,
        Self::GoToTasks,
        Self::Commit,
        Self::Push,
        Self::OpenPull,
        Self::Comment,
        Self::UndoDecision,
    ];
}

/// Whose keys these are. `Off` is for a reader who wants none at all.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Profile {
    /// The left hand's letters: `w` and `s` to move, `x` to mark, `r` for review mode.
    #[default]
    Standard,
    /// `j` and `k` to move, and the letters vim has other plans for left alone.
    Vim,
    Off,
}

/// GitQuiet's words and gists for each command, as its keyboard sheet lists them.
pub const KEYBOARD: &[(Command, &str, &str)] = &[
    (Command::GoToFile, "Go to file", "Any file in the repository, by typing its name"),
    (Command::GoToName, "Go to name", "Any name the repository writes down, by typing it"),
    (Command::FileNames, "Names in this file", "What this file writes down, and where"),
    (Command::Uses, "Uses", "Everywhere in this file that means the name under the pointer"),
    (Command::NextFile, "Next file", "Down the rail to the file after this one"),
    (Command::PreviousFile, "Previous file", "Back up the rail to the file before it"),
    (Command::MarkFile, "Mark file", "Turn this file's mark over, read or unread"),
    (Command::ReviewMode, "Review mode", "The files on the whole screen, and back"),
    (Command::OpenAside, "Open aside", "The row the walk is on, in the side panel"),
    (Command::Search, "Search", "The filter over whichever list is on screen"),
    (Command::Dismiss, "Close", "The way out of whatever is open"),
    (Command::ToggleDetails, "Details pane", "The left column of merge, conversation and checks, and back"),
    (Command::ToggleFiles, "Files pane", "The tree and the diff on the right, and back"),
    (Command::WorkingSet, "Working set", "Everything waiting on you"),
    (Command::Repositories, "Repositories", "The repositories you keep"),
    (Command::Activity, "Activity", "The feed"),
    (Command::Home, "Home", "The front of the interface"),
    (Command::GoToTasks, "Tasks", "The project's tasks, in the right pane"),
    (Command::Commit, "Commit", "What the review kept, as one commit on the branch"),
    (Command::Push, "Push", "The branch to its remote"),
    (Command::OpenPull, "Open pull request", "A pull request for the branch"),
    (Command::UndoDecision, "Undo decision", "The open file as it was before its last decision"),
    (Command::Comment, "Comment", "A comment on the line the caret is on"),
];

/// How long a sequence waits for its second key: GitQuiet's `PATIENCE`.
pub const PATIENCE: Duration = Duration::from_millis(1500);

/// The first key of a sequence and when it was pressed.
pub type Waiting = Option<(String, Instant)>;
