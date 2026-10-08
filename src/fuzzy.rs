//! Matching a few typed letters against names and paths, as a file finder does: the letters must come
//! in order, case aside, and a match counts for more when a letter starts a word or follows the one
//! before it. Nothing here knows what the texts are.

/// How well `query` matches `text`, or `None` when a letter of it is missing. Higher is better; an
/// empty query matches everything equally. Each letter scores 1, 8 more when it starts a word and 8
/// more when it follows the letter before; a letter after a gap costs 3 and up to 4 more for the gap's
/// length. So a run of letters beats a scatter of word starts. The best placement wins.
pub fn score(query: &str, text: &str) -> Option<i32> {
    let text: Vec<char> = text.chars().collect();
    let lower: Vec<char> = text
        .iter()
        .map(|c| c.to_lowercase().next().unwrap_or(*c))
        .collect();
    let query: Vec<char> = query
        .chars()
        .filter(|c| !c.is_whitespace())
        .filter_map(|c| c.to_lowercase().next())
        .collect();
    if query.is_empty() {
        return Some(0);
    }
    let gap = |from: usize, to: usize| 3 + (to - from - 1).min(4) as i32;
    // best[i]: the best score with the letters so far placed and the last one at `i`.
    let mut best: Vec<Option<i32>> = (0..text.len())
        .map(|i| (lower[i] == query[0]).then(|| 1 + word_bonus(&text, i) - (i.min(4) as i32)))
        .collect();
    for &wanted in &query[1..] {
        let mut next = vec![None; text.len()];
        for i in 0..text.len() {
            if lower[i] != wanted {
                continue;
            }
            next[i] = (0..i)
                .filter_map(|k| best[k].map(|s| s + if k + 1 == i { 8 } else { -gap(k, i) }))
                .max()
                .map(|s| s + 1 + word_bonus(&text, i));
        }
        best = next;
    }
    best.into_iter().flatten().max()
}

fn word_bonus(text: &[char], i: usize) -> i32 {
    if starts_word(text, i) { 8 } else { 0 }
}

/// Whether the letter at `i` starts a word: the text's start, after a separator, or a capital after a
/// small letter.
fn starts_word(text: &[char], i: usize) -> bool {
    match i.checked_sub(1).map(|p| text[p]) {
        None => true,
        Some(before) => {
            matches!(before, '/' | '\\' | '_' | '-' | '.' | ' ' | ':')
                || (before.is_lowercase() && text[i].is_uppercase())
        }
    }
}

/// The indices of `texts` that match `query`, best first; a tie goes to the shorter text, then to the
/// earlier one. At most `limit`.
pub fn rank<'a>(query: &str, texts: impl IntoIterator<Item = &'a str>, limit: usize) -> Vec<usize> {
    let mut scored: Vec<(i32, usize, usize)> = texts
        .into_iter()
        .enumerate()
        .filter_map(|(i, text)| Some((score(query, text)?, text.len(), i)))
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
    scored.into_iter().take(limit).map(|(_, _, i)| i).collect()
}

#[cfg(test)]
mod tests;
