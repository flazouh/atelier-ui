use super::structs::Finder;

/// The pointer over a row makes it the one the keys are on, as the palette does.
pub(super) fn pick_hover(finder: &gpui_kit::WeakEntity<Finder>, entry: usize, cx: &mut gpui_kit::App) {
    finder
        .update(cx, |this, cx| {
            let at = entry.saturating_sub(1);
            if this.selected != at && at < this.shown.len() {
                this.selected = at;
                cx.notify();
            }
        })
        .ok();
}
