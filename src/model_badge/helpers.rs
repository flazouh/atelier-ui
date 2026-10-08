use gpui_kit::SharedString;

/// The monogram's letter: the label's first, capitalised.
pub fn monogram_letter(label: &str) -> SharedString {
    label
        .chars()
        .next()
        .map(|c| c.to_uppercase().collect::<String>())
        .unwrap_or_else(|| "?".into())
        .into()
}
