use std::ops::Range;

use gpui_kit::SharedString;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Column {
    pub width: f32,
    /// The space before it; 0 for the first.
    pub gap_before: f32,
}

/// Where the columns sit in a row that scrolls.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Geometry {
    pub(super) columns: Vec<Column>,
    /// Each column's left edge, and the row's end at the last place.
    pub(super) lefts: Vec<f32>,
}

impl Geometry {
    pub fn new(columns: Vec<Column>) -> Self {
        let mut lefts = Vec::with_capacity(columns.len() + 1);
        let mut x = 0.;
        for column in &columns {
            x += column.gap_before;
            lefts.push(x);
            x += column.width;
        }
        lefts.push(x);
        Self { columns, lefts }
    }

    pub fn len(&self) -> usize {
        self.columns.len()
    }

    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    pub fn left(&self, column: usize) -> f32 {
        self.lefts[column]
    }

    pub fn right(&self, column: usize) -> f32 {
        self.lefts[column] + self.columns[column].width
    }

    pub fn width_of(&self, column: usize) -> f32 {
        self.columns[column].width
    }

    /// The row's whole width.
    pub fn total(&self) -> f32 {
        self.lefts.last().copied().unwrap_or(0.)
    }

    /// The furthest the row scrolls: 0 when it fits.
    pub fn max_offset(&self, viewport: f32) -> f32 {
        (self.total() - viewport).max(0.)
    }

    pub fn clamp(&self, offset: f32, viewport: f32) -> f32 {
        offset.clamp(0., self.max_offset(viewport))
    }

    /// The columns any part of which is within `margin` of the viewport at `offset`. Only these are laid
    /// out; the rest keep their state and take no layout.
    pub fn visible(&self, offset: f32, viewport: f32, margin: f32) -> Range<usize> {
        if self.columns.is_empty() {
            return 0..0;
        }
        let (from, to) = (offset - margin, offset + viewport + margin);
        let first = (0..self.len())
            .find(|&i| self.right(i) > from)
            .unwrap_or(self.len());
        let last = (first..self.len())
            .find(|&i| self.left(i) >= to)
            .unwrap_or(self.len());
        first..last
    }

    /// Where a scroll settles: the offset that puts the nearest column's left edge at the viewport's left
    /// edge, or the end of the row if that is nearer.
    pub fn snap(&self, offset: f32, viewport: f32) -> f32 {
        let end = self.max_offset(viewport);
        (0..self.len())
            .map(|i| self.left(i).min(end))
            .chain([end])
            .min_by(|a, b| (a - offset).abs().total_cmp(&(b - offset).abs()))
            .unwrap_or(0.)
    }

    /// The least scroll from `offset` that shows `column` whole. A column wider than the viewport lines its
    /// left edge up with the viewport's.
    pub fn reveal(&self, offset: f32, viewport: f32, column: usize) -> f32 {
        let (left, right) = (self.left(column), self.right(column));
        let target = if right - left >= viewport || left < offset {
            left
        } else if right > offset + viewport {
            right - viewport
        } else {
            offset
        };
        self.clamp(target, viewport)
    }

    /// The column under `x`, an x in the row's own coordinates.
    pub fn column_at(&self, x: f32) -> Option<usize> {
        (0..self.len()).find(|&i| x >= self.left(i) && x < self.right(i))
    }
}

/// Panels of one project, side by side.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    /// The project, or `None` for the one flat group of an ungrouped strip.
    pub project: Option<SharedString>,
    /// Indices into the panels, in the order they were opened.
    pub members: Vec<usize>,
}
