/// A token count as the axis says it: `2 M`, `1.5 M`, `500 k`, `42`.
pub fn tokens_label(tokens: u64) -> String {
    let say = |value: f64, unit: &str| {
        if (value - value.round()).abs() < 0.05 {
            format!("{} {unit}", value.round() as u64)
        } else {
            format!("{value:.1} {unit}")
        }
    };
    match tokens {
        1_000_000.. => say(tokens as f64 / 1e6, "M"),
        1_000.. => say(tokens as f64 / 1e3, "k"),
        _ => tokens.to_string(),
    }
}
