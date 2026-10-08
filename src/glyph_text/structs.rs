use super::types::Ink;
use gpui_kit::{
    App, AvailableSpace, Bounds, Element, ElementId, GlobalElementId, HighlightStyle, Hsla,
    InspectorElementId, IntoElement, LayoutId, Pixels, SharedString, Size, WhiteSpace, Window,
    WrappedLine, point,
};
use std::{cell::RefCell, ops::Range, panic::Location, rc::Rc};
/// What the measure found: the shaped lines, in the text's one color.
pub struct Shaped {
    lines: Vec<WrappedLine>,
    line_height: Pixels,
    color: Hsla,
}
pub struct GlyphText {
    text: SharedString,
    highlights: Vec<(Range<usize>, HighlightStyle)>,
    ink: Option<Ink>,
}
impl GlyphText {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            highlights: Vec::new(),
            ink: None,
        }
    }
    /// Colors byte ranges the way `StyledText::with_highlights` does: a highlight's color is blended over the text's.
    /// Only the color is used.
    pub fn highlights(
        mut self,
        highlights: impl IntoIterator<Item = (Range<usize>, HighlightStyle)>,
    ) -> Self {
        self.highlights = highlights.into_iter().collect();
        self
    }
    /// Colors each glyph from where it sits, after its highlights.
    pub fn ink(mut self, ink: Ink) -> Self {
        self.ink = Some(ink);
        self
    }
    fn color_at(&self, index: usize, base: Hsla) -> Hsla {
        match self
            .highlights
            .iter()
            .find(|(range, _)| range.contains(&index))
            .and_then(|(_, h)| h.color)
        {
            Some(color) => base.blend(color),
            None => base,
        }
    }
}
impl IntoElement for GlyphText {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}
impl Element for GlyphText {
    type RequestLayoutState = Rc<RefCell<Option<Rc<Shaped>>>>;
    type PrepaintState = ();
    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        _: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let style = window.text_style();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let line_height = window.pixel_snap(
            style
                .line_height
                .to_pixels(font_size.into(), window.rem_size()),
        );
        let wraps = style.white_space == WhiteSpace::Normal;
        let run = style.to_run(self.text.len());
        let text = self.text.clone();
        let shaped: Rc<RefCell<Option<Rc<Shaped>>>> = Rc::default();
        let keep = shaped.clone();
        let layout = window.request_measured_layout(
            Default::default(),
            move |known, available, window, _| {
                let wrap_width = if wraps {
                    known.width.or(match available.width {
                        AvailableSpace::Definite(width) => Some(width),
                        _ => None,
                    })
                } else {
                    None
                };
                let lines: Vec<WrappedLine> = window
                    .text_system()
                    .shape_text(
                        text.clone(),
                        font_size,
                        std::slice::from_ref(&run),
                        wrap_width,
                        None,
                    )
                    .map(|lines| lines.into_iter().collect())
                    .unwrap_or_default();
                let mut size = Size::<Pixels>::default();
                for line in &lines {
                    let line_size = line.size(line_height);
                    size.height += line_size.height;
                    size.width = size.width.max(line_size.width).ceil();
                }
                *keep.borrow_mut() = Some(Rc::new(Shaped {
                    lines,
                    line_height,
                    color: run.color,
                }));
                size
            },
        );
        (layout, shaped)
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        _: &mut Window,
        _: &mut App,
    ) {
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        shaped: &mut Self::RequestLayoutState,
        _: &mut (),
        window: &mut Window,
        _: &mut App,
    ) {
        let Some(shaped) = shaped.borrow().clone() else {
            return;
        };
        let mut top = bounds.origin.y;
        for line in &shaped.lines {
            let layout = &line.unwrapped_layout;
            let baseline =
                (shaped.line_height - layout.ascent - layout.descent) / 2. + layout.ascent;
            let glyphs: Vec<_> = layout
                .runs
                .iter()
                .enumerate()
                .flat_map(|(r, run)| {
                    run.glyphs
                        .iter()
                        .enumerate()
                        .map(move |(g, glyph)| (r, g, run.font_id, glyph))
                })
                .collect();
            // Where each row starts and ends along the unwrapped line.
            let mut rows = vec![Pixels::default()];
            for wrap in &line.wrap_boundaries {
                rows.push(layout.runs[wrap.run_ix].glyphs[wrap.glyph_ix].position.x);
            }
            rows.push(layout.width);
            let mut row = 0;
            for (k, &(run_ix, glyph_ix, font_id, glyph)) in glyphs.iter().enumerate() {
                if line
                    .wrap_boundaries
                    .iter()
                    .any(|w| w.run_ix == run_ix && w.glyph_ix == glyph_ix)
                {
                    row += 1;
                }
                let next = glyphs
                    .get(k + 1)
                    .map_or(layout.width, |(_, _, _, g)| g.position.x);
                let (start, end) = (rows[row], rows[row + 1]);
                let center = (glyph.position.x + next.min(end)) / 2. - start;
                let width = f32::from(end - start).max(1.);
                let mut color = self.color_at(glyph.index, shaped.color);
                if let Some(ink) = &self.ink {
                    color = ink(f32::from(center) / width, color);
                }
                let origin = point(
                    bounds.origin.x + glyph.position.x - start,
                    top + shaped.line_height * row as f32 + baseline + glyph.position.y,
                );
                let painted = if glyph.is_emoji {
                    window.paint_emoji(origin, font_id, glyph.id, layout.font_size)
                } else {
                    window.paint_glyph(origin, font_id, glyph.id, layout.font_size, color)
                };
                painted.ok();
            }
            top += line.size(shaped.line_height).height;
        }
    }
}
