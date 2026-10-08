//! What a review of changed files shares, for the agent's turn and the pull request view alike: its
//! progress and the words for it, the order Next and Previous walk, a whole-file decision, and the keys.
//!
//! The keys are GitQuiet's ([`crate::keys`]): `s` next file, `w` previous, `x` mark the file, `r` review
//! mode, Escape to close, ⌘B and ⌘⇧B for the two panes. Bare and shifted letters only work while no
//! text input has focus: Escape in the editor moves focus to the review pane, and the letters work from
//! there. ⌘B and ⌘⇧B hold a modifier and never type, so they work from the editor too. The editor keeps
//! an Escape it has a use for (extra carets, a completion) and passes on the rest. The review's own keys hold a modifier and work from the editor too: accept or reject the
//! whole file with `secondary-shift-enter` and `secondary-shift-backspace`, and the hunk under the caret
//! with the inline review's `secondary-enter` and `secondary-backspace`. Every file at once, from the bar's
//! menu: `secondary-alt-enter` accepts all and `secondary-alt-backspace` rejects all. A review that
//! offers more than one scope (one turn or the whole session) switches with `secondary-shift-t`.

use std::rc::Rc;

use gpui_kit::{
    App, FocusHandle, InteractiveElement, KeyBinding, KeyDownEvent, SharedString, Window,
    base::input,
};

use crate::{
    inline_review::{Decision, InlineHunk},
    keys::{self, Command, Press},
};

gpui_kit::actions!(
    review,
    [
        /// Accepts every hunk of the open file.
        AcceptFile,
        /// Rejects every hunk of the open file.
        RejectFile,
        /// Accepts every hunk of every file.
        AcceptAll,
        /// Rejects every hunk of every file.
        RejectAll,
        /// Moves to the review's other scope.
        SwitchScope,
    ]
);

/// The key context the owner puts on the whole review pane, so the file keys work from the editor too.
pub const CONTEXT: &str = "Review";

pub(crate) fn bind_keys(cx: &mut App) {
    let pane = Some(CONTEXT);
    cx.bind_keys([
        KeyBinding::new("secondary-shift-enter", AcceptFile, pane),
        KeyBinding::new("secondary-shift-backspace", RejectFile, pane),
        KeyBinding::new("secondary-alt-enter", AcceptAll, pane),
        KeyBinding::new("secondary-alt-backspace", RejectAll, pane),
        KeyBinding::new("secondary-shift-t", SwitchScope, pane),
    ]);
}

/// The caps for the review's own keys, which GitQuiet's table has no command for.
pub mod caps {
    pub const ACCEPT_FILE: &str = if cfg!(target_os = "macos") {
        "⌘⇧↵"
    } else {
        "⌃⇧↵"
    };
    pub const REJECT_FILE: &str = if cfg!(target_os = "macos") {
        "⌘⇧⌫"
    } else {
        "⌃⇧⌫"
    };
    pub const ACCEPT_ALL: &str = if cfg!(target_os = "macos") {
        "⌘⌥↵"
    } else {
        "⌃⌥↵"
    };
    pub const REJECT_ALL: &str = if cfg!(target_os = "macos") {
        "⌘⌥⌫"
    } else {
        "⌃⌥⌫"
    };
    pub const SWITCH_SCOPE: &str = if cfg!(target_os = "macos") {
        "⌘⇧T"
    } else {
        "⌃⇧T"
    };
}

/// `word` with its first letter in capitals: "seen" reads "Seen".
fn capitalized(word: &str) -> String {
    let mut chars = word.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}
/// How far a review has come.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReviewProgress {
    pub files: usize,
    pub reviewed: usize,
    pub added: usize,
    pub removed: usize,
}

impl ReviewProgress {
    pub fn changed_text(&self) -> SharedString {
        format!("{} changed", self.files).into()
    }

    /// "3 of 7 reviewed", then "All 7 reviewed", or "Reviewed" for one file.
    pub fn reviewed_text(&self) -> SharedString {
        self.reviewed_text_as("reviewed")
    }

    /// The same count with another word, such as "3 of 7 seen" in a pull request.
    pub fn reviewed_text_as(&self, word: &str) -> SharedString {
        if self.is_done() && self.files == 1 {
            capitalized(word).into()
        } else if self.is_done() {
            format!("All {} {word}", self.files).into()
        } else {
            format!("{} of {} {word}", self.reviewed, self.files).into()
        }
    }

    /// The count in the fewest characters: "1/254 seen", then "All 254 seen".
    pub fn short_text(&self, word: &str) -> SharedString {
        if self.is_done() && self.files == 1 {
            capitalized(word).into()
        } else if self.is_done() {
            format!("All {} {word}", self.files).into()
        } else {
            format!("{}/{} {word}", self.reviewed, self.files).into()
        }
    }

    /// The count with no word after it, for the narrowest bar: "1/254", "All 254".
    pub fn count_text(&self) -> SharedString {
        if self.is_done() && self.files > 1 {
            format!("All {}", self.files).into()
        } else {
            format!("{}/{}", self.reviewed, self.files).into()
        }
    }

    pub fn fraction(&self) -> f32 {
        if self.files == 0 {
            0.
        } else {
            self.reviewed.min(self.files) as f32 / self.files as f32
        }
    }

    pub fn is_done(&self) -> bool {
        self.files > 0 && self.reviewed >= self.files
    }

    /// Next becomes Done once every file is reviewed.
    pub fn next_label(&self) -> &'static str {
        if self.is_done() { "Done" } else { "Next" }
    }
}

/// The file `by` steps from `current` in `order`: 1 for Next, -1 for Previous. With no current file,
/// Next starts at the first and Previous at the last. `None` past either end.
pub fn step(
    order: &[SharedString],
    current: Option<&SharedString>,
    by: isize,
) -> Option<SharedString> {
    let at = match current.and_then(|c| order.iter().position(|p| p == c)) {
        Some(at) => at as isize + by,
        None if by > 0 => 0,
        None => order.len() as isize - 1,
    };
    usize::try_from(at)
        .ok()
        .and_then(|at| order.get(at))
        .cloned()
}

/// The same decision for every hunk of a file, for [`crate::inline_review::apply`].
pub fn whole_file(hunks: &[InlineHunk], decision: Decision) -> Vec<(InlineHunk, Decision)> {
    hunks.iter().map(|h| (h.clone(), decision)).collect()
}

/// A review action's callback.
pub type ReviewHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// What the owner does for each review action, from the bar's buttons and from the keys alike. A
/// command with no handler is left for whatever is around the review.
#[derive(Clone, Default)]
pub struct ReviewHandlers {
    pub on_next: Option<ReviewHandler>,
    pub on_previous: Option<ReviewHandler>,
    pub on_accept_file: Option<ReviewHandler>,
    pub on_reject_file: Option<ReviewHandler>,
    pub on_put_back: Option<ReviewHandler>,
    /// Every hunk of every file, from the bar's menu.
    pub on_accept_all: Option<ReviewHandler>,
    pub on_reject_all: Option<ReviewHandler>,
    /// The other scope, from the bar's switch.
    pub on_switch_scope: Option<ReviewHandler>,
    /// `x`: the file's mark, read or unread.
    pub on_mark: Option<ReviewHandler>,
    /// `r`: the files on the whole screen, and back.
    pub on_review_mode: Option<ReviewHandler>,
    /// Escape, while the pane has focus.
    pub on_dismiss: Option<ReviewHandler>,
    /// ⌘B: the details pane.
    pub on_toggle_details: Option<ReviewHandler>,
    /// ⌘⇧B: the files pane.
    pub on_toggle_files: Option<ReviewHandler>,
    /// `u`: the uses of the name under the pointer or the caret.
    pub on_uses: Option<ReviewHandler>,
    /// `o`: the names this file writes down.
    pub on_file_names: Option<ReviewHandler>,
    /// `T`: any name the repository writes down.
    pub on_go_to_name: Option<ReviewHandler>,
    /// `t`: any file in the repository.
    pub on_go_to_file: Option<ReviewHandler>,
    /// ⌘⇧C, ⌘⇧U, ⌘⇧R: ship what the review kept.
    pub on_commit: Option<ReviewHandler>,
    pub on_push: Option<ReviewHandler>,
    pub on_open_pull: Option<ReviewHandler>,
    /// U: the open file as it was before its last decision.
    pub on_undo_decision: Option<ReviewHandler>,
    /// `c`: a comment on the line the caret is on.
    pub on_comment: Option<ReviewHandler>,
}

macro_rules! setters {
    ($($name:ident),*) => {
        $(pub fn $name(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
            self.$name = Some(Rc::new(f));
            self
        })*
    };
}

impl ReviewHandlers {
    setters!(
        on_next,
        on_previous,
        on_accept_file,
        on_reject_file,
        on_put_back,
        on_accept_all,
        on_reject_all,
        on_switch_scope,
        on_mark,
        on_review_mode,
        on_dismiss,
        on_toggle_details,
        on_toggle_files,
        on_uses,
        on_file_names,
        on_go_to_name,
        on_go_to_file,
        on_commit,
        on_push,
        on_open_pull,
        on_comment,
        on_undo_decision
    );

    /// The handler a command from the key table runs here, if any.
    pub fn for_command(&self, command: Command) -> Option<&ReviewHandler> {
        match command {
            Command::NextFile => self.on_next.as_ref(),
            Command::PreviousFile => self.on_previous.as_ref(),
            Command::MarkFile => self.on_mark.as_ref(),
            Command::ReviewMode => self.on_review_mode.as_ref(),
            Command::Dismiss => self.on_dismiss.as_ref(),
            Command::ToggleDetails => self.on_toggle_details.as_ref(),
            Command::ToggleFiles => self.on_toggle_files.as_ref(),
            Command::Uses => self.on_uses.as_ref(),
            Command::FileNames => self.on_file_names.as_ref(),
            Command::GoToName => self.on_go_to_name.as_ref(),
            Command::GoToFile => self.on_go_to_file.as_ref(),
            Command::Commit => self.on_commit.as_ref(),
            Command::Push => self.on_push.as_ref(),
            Command::OpenPull => self.on_open_pull.as_ref(),
            Command::UndoDecision => self.on_undo_decision.as_ref(),
            Command::Comment => self.on_comment.as_ref(),
            _ => None,
        }
    }

    /// Wires the keys to these handlers on `pane`, the element that holds the whole review, which takes
    /// `focus`. Escape in a text input inside it hands focus to the pane, so the letters work from there;
    /// in a read-only one, where nothing is being typed, it is the review's Escape at once.
    pub fn keys<E: InteractiveElement>(&self, pane: E, focus: &FocusHandle) -> E {
        let (accept, reject) = (self.on_accept_file.clone(), self.on_reject_file.clone());
        let (accept_all, reject_all, switch) = (
            self.on_accept_all.clone(),
            self.on_reject_all.clone(),
            self.on_switch_scope.clone(),
        );
        let handlers = self.clone();
        let pane_focus = focus.clone();
        let dismiss = self.on_dismiss.clone();
        pane.key_context(CONTEXT)
            .track_focus(focus)
            .on_key_down(move |event: &KeyDownEvent, window, cx| {
                let press = Press::from_keystroke(&event.keystroke);
                // A bare or shifted letter would be text, so it waits until no input has focus. A chord
                // held with Command (Control elsewhere) never types, so ⌘B works from the editor too.
                if !press.secondary && keys::typing(window) {
                    return;
                }
                let Some(command) = keys::read_now(&press, cx) else {
                    return;
                };
                if event.is_held && !keys::held_down(command) {
                    return;
                }
                if let Some(f) = handlers.for_command(command) {
                    cx.stop_propagation();
                    f(window, cx);
                }
            })
            .on_action(move |_: &input::Escape, window, cx| {
                let reading = !keys::typing(window);
                pane_focus.focus(window, cx);
                if let Some(f) = dismiss.as_ref().filter(|_| reading) {
                    f(window, cx);
                }
            })
            .on_action(move |_: &AcceptFile, window, cx| {
                if let Some(f) = &accept {
                    f(window, cx)
                }
            })
            .on_action(move |_: &RejectFile, window, cx| {
                if let Some(f) = &reject {
                    f(window, cx)
                }
            })
            .on_action(move |_: &AcceptAll, window, cx| {
                if let Some(f) = &accept_all {
                    f(window, cx)
                }
            })
            .on_action(move |_: &RejectAll, window, cx| {
                if let Some(f) = &reject_all {
                    f(window, cx)
                }
            })
            .on_action(move |_: &SwitchScope, window, cx| {
                if let Some(f) = &switch {
                    f(window, cx)
                }
            })
    }
}

#[cfg(test)]
mod tests;
