use super::types::{EXTENSIONS, FOLDERS, NAMES};

/// Whether the path ends in an image extension the badge can draw.
pub fn is_icon_file(path: &str) -> bool {
    path.rsplit_once('.').is_some_and(|(_, ext)| EXTENSIONS.contains(&ext.to_lowercase().as_str()))
}

pub(super) fn stem(path: &str) -> String {
    let file = path.rsplit('/').next().unwrap_or(path);
    file.rsplit_once('.').map_or(file, |(name, _)| name).to_lowercase()
}

pub(super) fn folder(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(dir, _)| dir)
}

/// 0 for a mark-like name in a conventional folder, 1 for the name alone, 2 for the folder alone, 3 for the rest.
pub fn rank_of(path: &str) -> u8 {
    let named = NAMES.contains(&stem(path).as_str());
    let placed = FOLDERS.contains(&folder(path));
    match (named, placed) {
        (true, true) => 0,
        (true, false) => 1,
        (false, true) => 2,
        (false, false) => 3,
    }
}

/// The image files among `paths`, most likely first.
pub fn rank(paths: &[&str]) -> Vec<String> {
    let mut found: Vec<&str> = paths.iter().copied().filter(|p| is_icon_file(p)).collect();
    found.sort_by(|a, b| {
        rank_of(a)
            .cmp(&rank_of(b))
            .then_with(|| a.matches('/').count().cmp(&b.matches('/').count()))
            .then_with(|| a.cmp(b))
    });
    found.into_iter().map(str::to_string).collect()
}

/// The paths that hold every word of `query`, without regard to case.
pub fn filter(paths: &[String], query: &str) -> Vec<String> {
    let words: Vec<String> = query.to_lowercase().split_whitespace().map(str::to_string).collect();
    paths.iter().filter(|p| words.iter().all(|w| p.to_lowercase().contains(w))).cloned().collect()
}
