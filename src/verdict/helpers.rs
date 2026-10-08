use gpui_kit::SharedString;

use super::types::{Decision, Verb};

/// The verbs offered: no Approve on one's own pull request.
pub fn offered(mine: bool) -> Vec<Verb> {
    Verb::ALL.into_iter().filter(|v| !(mine && *v == Verb::Approve)).collect()
}

/// Whether a verb can be pressed now: nothing while one is on its way, and words for the two that need
/// them.
pub fn enabled(verb: Verb, text: &str, sending: bool) -> bool {
    !sending && (verb.wordless() || !text.trim().is_empty())
}

/// The header's words: what the reader said, if anything, and whether it was about the last commit.
pub fn summary(stated: Option<(Decision, bool)>) -> SharedString {
    match stated {
        None => "not read yet by you".into(),
        Some((decision, current)) => {
            let said = match decision {
                Decision::Approved => "You approved this",
                Decision::ChangesRequested => "You asked for changes",
                Decision::Commented => "You commented on this",
                Decision::Dismissed => "Your review was dismissed",
            };
            if current { said.into() } else { format!("{said}, at an older commit").into() }
        }
    }
}

/// The commit, as the box names it.
pub fn about(head_sha: &str) -> SharedString {
    head_sha.chars().take(7).collect::<String>().into()
}

/// What is said when a verb that needs words is pressed on an empty box.
pub fn needs_words(verb: Verb) -> Option<&'static str> {
    match verb {
        Verb::Approve => None,
        Verb::RequestChanges => Some("Requesting changes needs words."),
        Verb::Comment => Some("A comment needs words."),
    }
}

/// Said once under the verbs while the box is empty.
pub fn hint(mine: bool) -> &'static str {
    if mine { "A comment needs words." } else { "An approval needs no words. The other two do." }
}

pub fn placeholder(mine: bool) -> &'static str {
    if mine { "Answer the review" } else { "Say what you found" }
}
