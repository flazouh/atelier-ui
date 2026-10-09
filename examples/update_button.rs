//! The update button in all its states: `UPDATE_THEME=light|dark`. Press the ready button, or Tab to it, to see the
//! hover and focus looks.
use atelier_ui::{
    UpdateButton,
    theme::{ActiveTheme, Appearance, set_appearance},
};
use gpui_kit::{
    AppContext, Bounds, Context, IntoElement, ParentElement, Render, Styled, Window, WindowBounds, WindowOptions, div, px,
    size,
};

struct Page;

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let cell = |button: UpdateButton| div().w(px(190.)).flex().justify_center().child(button);
        div().size_full().bg(theme.background).p(px(24.)).flex().child(
            div()
                .flex()
                .child(cell(UpdateButton::new("d12").downloading(0.12, "Updating 12%")))
                .child(cell(UpdateButton::new("d55").downloading(0.55, "Updating 55%")))
                .child(cell(UpdateButton::new("d92").downloading(0.92, "Updating 92%")))
                .child(cell(UpdateButton::new("ready").ready("Update to v0.1.9").on_click(|_, _| {})))
                .child(cell(UpdateButton::new("restart").restarting("Restarting…"))),
        )
    }
}

fn main() {
    gpui_kit::application().with_assets(atelier_ui::icon::Assets).run(|cx| {
        atelier_ui::init(cx);
        match std::env::var("UPDATE_THEME").as_deref() {
            Ok("light") => set_appearance(Appearance::Light, cx),
            _ => set_appearance(Appearance::Dark, cx),
        }
        let bounds = Bounds::centered(None, size(px(1000.), px(80.)), cx);
        cx.open_window(
            WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), ..Default::default() },
            |_, cx| cx.new(|_| Page),
        )
        .expect("open the window");
        cx.activate(true);
    });
}
