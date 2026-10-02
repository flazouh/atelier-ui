/// The line height that goes with a text size, on the 4px grid: 12 is 16, 14 is 20, 16 is 24.
pub fn line_for(size: f32) -> f32 {
    (size * 1.4 / 4.).round() * 4.
}

/// The text cut into runs: `(text, is_digits)`.
pub fn runs(text: &str) -> Vec<(String, bool)> {
    let mut out: Vec<(String, bool)> = Vec::new();
    for c in text.chars() {
        let digit = c.is_ascii_digit();
        match out.last_mut() {
            Some((run, d)) if *d == digit => run.push(c),
            _ => out.push((c.to_string(), digit)),
        }
    }
    out
}
