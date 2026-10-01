use std::time::Instant;

use gpui_kit::{App, SharedString, Window};

use super::structs::{Keys, Press};
use super::types::{Command, KEYBOARD, PATIENCE, Profile, Waiting};

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

pub(super) fn commands_for(profile: Profile, wanted: impl Fn(&str) -> bool) -> Option<Command> {
    Command::ALL.iter().copied().find(|c| chords(profile, *c).iter().any(|chord| wanted(chord)))
}

/// Whether a chord held with a modifier (`⌘b`, `⌘⇧b`) is this press.
pub(super) fn combo_matches(chord: &str, press: &Press) -> bool {
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
