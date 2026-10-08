use gpui_kit::SharedString;

use super::structs::CommitData;

/// "none yet", "one, 2h ago", "6, newest 2h ago".
pub fn how_many(commits: &[CommitData]) -> SharedString {
    let Some(newest) = commits.iter().max_by_key(|c| c.at) else {
        return "none yet".into();
    };
    let age = &newest.age;
    match (commits.len(), age.is_empty()) {
        (1, true) => "one".into(),
        (1, false) => format!("one, {age}").into(),
        (n, true) => format!("{n}").into(),
        (n, false) => format!("{n}, newest {age}").into(),
    }
}
