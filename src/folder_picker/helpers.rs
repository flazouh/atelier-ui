use gpui_kit::SharedString;

use super::types::FolderError;

/// The directory to list and the start of the name being typed, from the text in the field. A path with no `/` is under
/// the home folder.
pub fn split_path(text: &str) -> (String, String) {
    match text.rfind('/') {
        Some(i) => (text[..=i].to_string(), text[i + 1..].to_string()),
        None => ("~/".to_string(), text.to_string()),
    }
}

/// The folders whose names start with `prefix`, without regard to case. A folder whose name starts with a dot shows only
/// when the prefix starts with one.
pub fn matches<'a>(folders: &'a [SharedString], prefix: &str) -> Vec<&'a SharedString> {
    let prefix = prefix.to_lowercase();
    folders
        .iter()
        .filter(|name| {
            (prefix.starts_with('.') || !name.starts_with('.'))
                && name.to_lowercase().starts_with(&prefix)
        })
        .collect()
}

/// The text after Tab: one match completes with a `/`, several complete as far as their names agree, none leave the text.
pub fn tab_complete(text: &str, found: &[&SharedString]) -> String {
    let (dir, _) = split_path(text);
    match found {
        [] => text.to_string(),
        [one] => format!("{dir}{one}/"),
        [first, rest @ ..] => {
            let mut common: &str = first;
            for name in rest {
                let agree = common
                    .chars()
                    .zip(name.chars())
                    .take_while(|(a, b)| a.to_lowercase().eq(b.to_lowercase()))
                    .map(|(a, _)| a.len_utf8())
                    .sum();
                common = &common[..agree];
            }
            let typed = &text[dir.len()..];
            if common.chars().count() > typed.chars().count() {
                format!("{dir}{common}")
            } else {
                text.to_string()
            }
        }
    }
}

/// The folder to open for the text: no trailing `/`, except for the root itself.
pub fn folder_of(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return "~".into();
    }
    if trimmed.len() > 1 {
        trimmed.trim_end_matches('/').to_string()
    } else {
        trimmed.to_string()
    }
}

/// What the picker says for `error` on the folder named by `path` (a path with or without a trailing `/`): plain words,
/// not the system's. Whether it is worth a warning tone is [`FolderError::is_quiet`].
pub fn error_words(path: &str, error: &FolderError) -> String {
    let trimmed = if path.len() > 1 {
        path.trim_end_matches('/')
    } else {
        path
    };
    let (parent, name) = match trimmed.rfind('/') {
        Some(0) => ("/", &trimmed[1..]),
        Some(i) => (&trimmed[..i], &trimmed[i + 1..]),
        None => ("~", trimmed),
    };
    match error {
        FolderError::Missing if name.is_empty() => "There is no such folder.".to_string(),
        FolderError::Missing => format!("No folder named {name} in {parent}"),
        FolderError::Denied => format!("You may not look in {trimmed}"),
        FolderError::NotAFolder => format!("{name} is a file, not a folder"),
        FolderError::Other(why) => {
            let why = why.split(" (os error").next().unwrap_or(why).trim();
            format!("{trimmed} could not be read: {why}")
        }
    }
}
