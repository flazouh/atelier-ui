use gpui_kit::Hsla;
use std::rc::Rc;
/// A glyph's color from where it sits: its center as a fraction of its row's width, 0 at the left edge and 1 at the
/// right, and the color the text and its highlights give it there.
pub type Ink = Rc<dyn Fn(f32, Hsla) -> Hsla>;
