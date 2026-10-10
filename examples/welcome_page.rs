//! The welcome page, the first step of onboarding. `WELCOME_THEME=light|dark`.
use atelier_ui::{
    WelcomePage,
    theme::{Appearance, set_appearance},
};
use gpui_kit::{
    AppContext, Bounds, Context, IntoElement, ParentElement, Render, Styled, Window, WindowBounds, WindowOptions,
    div, px, size,
};
struct Page;
impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .child(WelcomePage::new("welcome").on_continue(|_, _| {}))
    }
}
fn main() {
    gpui_kit::application()
        .with_assets(atelier_ui::icon::Assets)
        .run(|cx| {
            atelier_ui::init(cx);
            match std::env::var("WELCOME_THEME").as_deref() {
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
