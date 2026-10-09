//! The release sheet in a flush modal, with three releases. `RELEASE_THEME=light|dark`.
use atelier_ui::{
    Modal, ReleaseNote, ReleaseSheet, ReleaseVersion,
    theme::{Appearance, set_appearance},
};
use gpui_kit::{
    AppContext, Bounds, Context, IntoElement, ParentElement, Render, Styled, Window, WindowBounds,
    WindowOptions, div, px, size,
};

struct Page;

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let note = |lead: &'static str, text: &'static str| ReleaseNote::new(lead, text);
        let date = || Some("Oct 9, 2026".into());
        let sheet = ReleaseSheet::new("sheet", "0.1.8")
            .date(date())
            .notes([
                note("The composer's stop", "The stop icon is a little smaller and red."),
                note("The session header", "The Stop button is gone. The composer stops a turn."),
                note("What's new", "The chip in the title bar no longer lies over the layout menu."),
            ])
            .earlier([
                ReleaseVersion::new("0.1.7", [note("Tall panels", "Zoomed in, or in a small window, the update sheet and the changelog were cut at the top. A panel now stays inside the window and scrolls.")])
                    .date(date()),
                ReleaseVersion::new("0.1.6", [
                    note("The usage dashboard", "Press the usage chips at the bottom of the window to see what Claude Code and Codex spent on this computer. It shows tokens per day, the models and the sessions, with a chart for each session."),
                    note("Several accounts", "Each Claude Code account gets its own tile and its own shade, grouped under one caption."),
                    note("The version in the status bar", "It stands at the bottom left, left of the CPU. CPU and RAM moved to the right. Press the version to read the changelog."),
                ])
                .date(date()),
            ])
            .on_close(|_, _| {});
        div()
            .size_full()
            .child(Modal::new("modal").width(860.).flush().child(sheet))
    }
}

fn main() {
    gpui_kit::application()
        .with_assets(atelier_ui::icon::Assets)
        .run(|cx| {
            atelier_ui::init(cx);
            match std::env::var("RELEASE_THEME").as_deref() {
                Ok("light") => set_appearance(Appearance::Light, cx),
                _ => set_appearance(Appearance::Dark, cx),
            }
            let bounds = Bounds::centered(None, size(px(1280.), px(900.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_, cx| cx.new(|_| Page),
            )
            .expect("open the window");
            cx.activate(true);
        });
}
