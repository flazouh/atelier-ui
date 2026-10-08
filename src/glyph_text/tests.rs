use super::GlyphText;
use crate::typography::TextSize;
use gpui_kit::{
    Context, HighlightStyle, InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    Styled, TestAppContext, Window, div, px, red,
};
use std::rc::Rc;
const TEXTS: [&str; 3] = [
    "Thinking",
    "The build finished and every test passed, so the change is ready to merge whenever you are.",
    "Supercalifragilisticexpialidocious words, and antidisestablishmentarianism too, break in odd places.",
];
struct Pair {
    text: &'static str,
    width: f32,
}
impl Render for Pair {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let every_other = self
            .text
            .char_indices()
            .filter(|(i, _)| i % 2 == 0)
            .map(|(i, c)| {
                (
                    i..i + c.len_utf8(),
                    HighlightStyle {
                        color: Some(red()),
                        ..Default::default()
                    },
                )
            });
        let shell = |d: gpui_kit::Div| {
            d.flex_none()
                .max_w(px(self.width))
                .text_size(TextSize::Sm.font_size())
                .line_height(px(24.))
        };
        div()
            .flex()
            .flex_col()
            .items_start()
            .child(
                shell(div().debug_selector(|| "plain".into())).child(SharedString::from(self.text)),
            )
            .child(
                shell(div().debug_selector(|| "glyphs".into())).child(
                    GlyphText::new(self.text)
                        .highlights(every_other)
                        .ink(Rc::new(|at, c| c.opacity(at))),
                ),
            )
    }
}
/// The tail is drawn as a `GlyphText` while it grows and as plain text once it ends, and the Thinking label swaps the
/// same way when its shimmer stops: the two must take the same room and break the same lines, or the text jumps.
#[gpui_kit::test]
fn colored_glyphs_lay_out_as_the_plain_text_does(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    let mut worst = Vec::new();
    for text in TEXTS {
        for width in [60., 120., 200., 320., 480., 800.] {
            let (_view, cx) = cx.add_window_view(move |_, _| Pair { text, width });
            cx.run_until_parked();
            let (plain, glyphs) = (
                cx.debug_bounds("plain").unwrap(),
                cx.debug_bounds("glyphs").unwrap(),
            );
            if plain.size != glyphs.size {
                worst.push(format!(
                    "{width}px: plain {:?}, glyphs {:?}: {text}",
                    plain.size, glyphs.size
                ));
            }
        }
    }
    assert!(
        worst.is_empty(),
        "the two lay out differently:\n{}",
        worst.join("\n")
    );
}
