use super::types::QuestionView;

/// What the reader's picks and words answer to `question`: for several choices the labels picked, joined by a comma, then the
/// words of their own; for a single choice, the words if there are any, else the label.
pub(super) fn answer_text(question: &QuestionView, picked: &[usize], custom: &str) -> String {
    // One answer to a single choice: words of the reader's own are it, whatever was picked before them.
    if !question.multiple && !custom.trim().is_empty() {
        return custom.trim().to_string();
    }
    let mut parts: Vec<String> = picked
        .iter()
        .filter_map(|&i| question.options.get(i))
        .map(|(label, _)| label.to_string())
        .collect();
    let custom = custom.trim();
    if !custom.is_empty() {
        parts.push(custom.to_string());
    }
    parts.join(", ")
}

/// Whether a question has an answer: something picked, or words written.
pub(super) fn is_answered(picked: &[usize], custom: &str) -> bool {
    !picked.is_empty() || !custom.trim().is_empty()
}

/// The picks of a question after the option `at` is pressed: a single choice takes it alone, and a press on the one already
/// picked lets it go; several choices toggle it.
pub(super) fn pressed(picked: &[usize], at: usize, multiple: bool) -> Vec<usize> {
    let has = picked.contains(&at);
    if multiple {
        let mut next: Vec<usize> = picked.iter().copied().filter(|&i| i != at).collect();
        if !has {
            next.push(at);
            next.sort_unstable();
        }
        next
    } else if has {
        Vec::new()
    } else {
        vec![at]
    }
}
