//! What the keyboard can ask for, and which keys ask for it: GitQuiet's table (`src/keys/commands.ts`),
//! ported with its words.
//!
//! One table, read by two things: the matcher that turns a press into a [`Command`], and the buttons that
//! wear a cap. The cap on a button comes out of the same table ([`chord_for`]), so the letter on a
//! control and the letter that works are the same letter by construction.
//!
//! Bare letters never type into text and are never taken from it: an owner reads a press with [`read`]
//! only while no text input has focus (see [`typing`]). The review's own keys, which GitQuiet has no
//! word for (accept or reject a hunk or a whole file), hold a modifier and stay GPUI key bindings.
//!
//! A chord is the key as the reader presses it: `s`, `T` (Shift and t), `/`, `Escape`, `⌘b` (Command or
//! Control and b), `⌘⇧b`. Two keys one after the other are one chord with a space, `g d`.

use std::time::{Duration, Instant};

use gpui_kit::{App, Global, Keystroke, SharedString, Window};

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

/// The chords that reach `command` in `profile`. The first is the one a button's cap shows.
pub fn chords(profile: Profile, command: Command) -> &'static [&'static str] {
    use Command::*;
    match profile {
        Profile::Off => &[],
        Profile::Standard => match command {
            GoToFile => &["t"],
            GoToName => &["T"],
            FileNames => &["o"],
            Uses => &["u"],
            NextFile => &["s"],
            PreviousFile => &["w"],
            MarkFile => &["x"],
            ReviewMode => &["r"],
            OpenAside => &["A"],
            Search => &["f", "/"],
            Dismiss => &["Escape"],
            ToggleDetails => &["⌘b"],
            ToggleFiles => &["⌘⇧b"],
            WorkingSet => &["g d"],
            Repositories => &["g r"],
            Activity => &["g f"],
            Home => &["g g"],
            GoToTasks => &["g t"],
            Commit => &["⌘⇧c"],
            Push => &["⌘⇧u"],
            OpenPull => &["⌘⇧r"],
            Comment => &["c"],
            UndoDecision => &["U"],
        },
        Profile::Vim => match command {
            GoToFile => &["t"],
            GoToName => &["T"],
            FileNames => &["o"],
            Uses => &["u"],
            NextFile => &["j"],
            PreviousFile => &["k"],
            MarkFile => &["x"],
            ReviewMode => &["r"],
            OpenAside => &["O"],
            Search => &["/"],
            Dismiss => &["Escape"],
            ToggleDetails => &["⌘b"],
            ToggleFiles => &["⌘⇧b"],
            WorkingSet => &["g d"],
            Repositories => &["g r"],
            Activity => &["g f"],
            Home => &["g h"],
            GoToTasks => &["g t"],
            Commit => &["⌘⇧c"],
            Push => &["⌘⇧u"],
            OpenPull => &["⌘⇧r"],
            Comment => &["g c"],
            UndoDecision => &["U"],
        },
    }
}

/// The chord a button that answers `command` wears, or `None` in a profile that gives it none.
pub fn chord_for(profile: Profile, command: Command) -> Option<&'static str> {
    chords(profile, command).first().copied()
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

pub fn word(command: Command) -> &'static str {
    KEYBOARD.iter().find(|(c, _, _)| *c == command).map_or("", |(_, w, _)| w)
}

pub fn gist(command: Command) -> &'static str {
    KEYBOARD.iter().find(|(c, _, _)| *c == command).map_or("", |(_, _, g)| g)
}

/// Only moving between files repeats while the key is held, as GitQuiet's `heldDown`: a held `x` must
/// not flip a mark back and forth.
pub fn held_down(command: Command) -> bool {
    matches!(command, Command::NextFile | Command::PreviousFile)
}

/// The profile in force for the whole app, and a sequence half pressed. One keyboard, so one of each.
#[derive(Clone, Debug, Default)]
pub struct Keys {
    pub profile: Profile,
    waiting: Waiting,
}

impl Global for Keys {}

pub fn profile(cx: &App) -> Profile {
    cx.try_global::<Keys>().map_or(Profile::default(), |k| k.profile)
}

pub fn set_profile(profile: Profile, cx: &mut App) {
    cx.set_global(Keys { profile, waiting: None });
}

/// [`read`] with the app's profile and its half-pressed sequence, now.
pub fn read_now(press: &Press, cx: &mut App) -> Option<Command> {
    let mut keys = cx.try_global::<Keys>().cloned().unwrap_or_default();
    let command = read(press, keys.profile, &mut keys.waiting, Instant::now());
    cx.set_global(keys);
    command
}

/// The cap a button wears for `chord`, in the glyphs [`crate::kbd::Kbd`] draws, as GitQuiet's caps do:
/// `⌘` for Command on macOS, `⌃` for Control elsewhere, and a capital for the letter a modifier goes with.
pub fn cap(chord: &str) -> SharedString {
    cap_on(chord, cfg!(target_os = "macos"))
}

/// [`cap`] for the Mac (`mac`) or for Linux and Windows, so a test reads both.
pub fn cap_on(chord: &str, mac: bool) -> SharedString {
    if chord == "Escape" {
        return "Esc".into();
    }
    if !chord.starts_with(['⌘', '⇧']) {
        return chord.to_string().into();
    }
    let (secondary, shift) = (chord.contains('⌘'), chord.contains('⇧'));
    let key = chord.trim_start_matches(['⌘', '⇧']);
    // A letter shows as a capital; a named key ("Tab") keeps its case.
    let key = if key.chars().count() == 1 { key.to_uppercase() } else { key.to_string() };
    let mut cap = String::new();
    if secondary {
        cap.push(if mac { '⌘' } else { '⌃' });
    }
    if shift {
        cap.push('⇧');
    }
    cap.push_str(&key);
    cap.into()
}

/// One key press as the table reads it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Press {
    /// The key as the reader means it: `s`, `T`, `/`, `Escape`. With Shift and no other modifier this is
    /// the character Shift made.
    pub key: String,
    /// Command on macOS, Control elsewhere.
    pub secondary: bool,
    pub alt: bool,
    pub shift: bool,
}

impl Press {
    pub fn from_keystroke(k: &Keystroke) -> Self {
        let m = &k.modifiers;
        let secondary = m.secondary();
        let key = match (k.key.as_str(), &k.key_char) {
            ("escape", _) => "Escape".to_string(),
            ("enter", _) => "Enter".to_string(),
            (_, Some(ch)) if !secondary && !m.alt && ch.chars().count() == 1 => ch.clone(),
            (key, _) => key.to_string(),
        };
        Self { key, secondary, alt: m.alt, shift: m.shift }
    }
}

/// How long a sequence waits for its second key: GitQuiet's `PATIENCE`.
pub const PATIENCE: Duration = Duration::from_millis(1500);

/// The first key of a sequence and when it was pressed.
pub type Waiting = Option<(String, Instant)>;

fn commands_for(profile: Profile, wanted: impl Fn(&str) -> bool) -> Option<Command> {
    Command::ALL.iter().copied().find(|c| chords(profile, *c).iter().any(|chord| wanted(chord)))
}

/// Whether a chord held with a modifier (`⌘b`, `⌘⇧b`) is this press.
fn combo_matches(chord: &str, press: &Press) -> bool {
    if !chord.starts_with(['⌘', '⇧']) || chord.contains(' ') {
        return false;
    }
    let (secondary, shift) = (chord.contains('⌘'), chord.contains('⇧'));
    let key = chord.trim_start_matches(['⌘', '⇧']);
    secondary == press.secondary && shift == press.shift && !press.alt && key.eq_ignore_ascii_case(&press.key)
}

/// The command `press` asks for, as GitQuiet's `read`: a press held with Command, Control or Alt only
/// reaches a chord that names that modifier; a key that opens a sequence waits up to [`PATIENCE`] for
/// its second one.
pub fn read(press: &Press, profile: Profile, waiting: &mut Waiting, now: Instant) -> Option<Command> {
    if press.secondary || press.alt {
        *waiting = None;
        return commands_for(profile, |chord| combo_matches(chord, press));
    }
    if let Some((leader, at)) = waiting.take()
        && now.saturating_duration_since(at) <= PATIENCE
    {
        let wanted = format!("{leader} {}", press.key);
        return commands_for(profile, |chord| chord == wanted);
    }
    if let Some(alone) = commands_for(profile, |chord| !chord.starts_with(['⌘', '⇧']) && !chord.contains(' ') && chord == press.key) {
        return Some(alone);
    }
    let opens = format!("{} ", press.key);
    if commands_for(profile, |chord| chord.starts_with(&opens)).is_some() {
        *waiting = Some((press.key.clone(), now));
    }
    None
}

/// Whether a text input has focus: then no bare letter may be read. A read-only one, such as a pull
/// request's diff, is read rather than typed in, so the letters work there. The innermost input
/// decides, so a reply box inside a read-only diff still types.
pub fn typing(window: &Window) -> bool {
    let stack = window.context_stack();
    stack.iter().rev().find(|context| context.contains("Input")).is_some_and(|input| !input.contains("readonly"))
}

#[cfg(test)]
mod tests;
