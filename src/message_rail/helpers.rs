/// `text` cut at a word near `limit` characters, with an ellipsis, as the web does: at the last space past 65% of the
/// limit, else at the limit.
pub fn excerpt(text: &str, limit: usize) -> String {
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.chars().count() <= limit {
        return text;
    }
    let cut: String = text.chars().take(limit).collect();
    let boundary = cut.rfind(' ').filter(|&b| cut[..b].chars().count() as f32 > limit as f32 * 0.65).unwrap_or(cut.len());
    format!("{}…", cut[..boundary].trim())
}

/// How long tick `index` is, from 0 to 1, with `lit` the tick that is full length: 1, then .68, .44, and .25 for the rest.
pub fn tick_scale(index: usize, lit: Option<usize>) -> f32 {
    match lit.map(|l| l.abs_diff(index)) {
        Some(0) => 1.,
        Some(1) => 0.68,
        Some(2) => 0.44,
        _ => 0.25,
    }
}

/// How tall each tick's box is: 14 as on the web, less when so many messages would not fit in `room`.
pub fn item_size(count: usize, room: f32) -> f32 {
    if count == 0 { 14. } else { (room / count as f32).clamp(4., 14.) }
}
