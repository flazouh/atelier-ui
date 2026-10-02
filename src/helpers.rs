use gpui_kit::{App, Context, Subscription, Window};

/// Sets up fonts, text rendering, and the theme macOS uses now. Call once, before opening a window.
/// Pass [`Assets`](crate::icon::Assets) to the application so icons load.
pub fn init(cx: &mut App) {
    // Must run before the first glyph is drawn.
    super::typography::disable_font_smoothing();
    gpui_kit::init(cx);
    super::code_editor::bind_keys(cx);
    super::inline_review::bind_keys(cx);
    super::agent_panels::bind_keys(cx);
    super::review::bind_keys(cx);
    super::select::bind_keys(cx);
    super::new_task::bind_keys(cx);
    super::changed_file_tree::bind_keys(cx);
    // After the inline review's keys, so the composer's own win inside it.
    super::line_comment::bind_keys(cx);
    super::comment_composer::bind_keys(cx);
    super::accessibility::sync_reduce_motion(cx);
    super::typography::load_fonts(cx);
    super::theme::follow_system(cx);
}

/// Keeps Reduce Motion and light or dark in step with macOS while the app runs. Call from the root view
/// of each window and keep the subscriptions for the window's life.
pub fn watch_system<T: 'static>(window: &mut Window, cx: &mut Context<T>) -> [Subscription; 2] {
    [
        // macOS has no Reduce Motion event, so read it again whenever the user comes back to the window.
        cx.observe_window_activation(window, |_, window, cx| {
            if window.is_window_active() {
                super::accessibility::sync_reduce_motion(cx);
            }
        }),
        // GPUI also calls this when the window opens; act only on a real change, so a theme the user
        // picked in the app is not reset.
        {
            let mut last = super::theme::Appearance::of_system(window.appearance());
            cx.observe_window_appearance(window, move |_, window, cx| {
                let now = super::theme::Appearance::of_system(window.appearance());
                if now != last {
                    last = now;
                    super::theme::set_appearance(now, cx);
                }
            })
        },
    ]
}
