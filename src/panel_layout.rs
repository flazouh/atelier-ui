//! The geometry of agent panels side by side, with no window: where each column sits, which ones a
//! scroll position shows, where a scroll settles, and how panels group by project. The strip draws what
//! this decides.
use std::ops::Range;

use gpui_kit::SharedString;

/// A column's width when the reader has not sized it, and the least and most it can be.
pub const DEFAULT_WIDTH: f32 = 480.;
pub const MIN_WIDTH: f32 = 320.;
pub const MAX_WIDTH: f32 = 960.;
/// The space between two columns of a group, and between two groups.
pub const GAP: f32 = 8.;
pub const GROUP_GAP: f32 = 24.;

/// A column's width in a strip `viewport` wide: never wider than the strip, so a panel never draws
/// under the pane beside it. The width asked for stays as it is, for when the strip grows again. A
/// strip not yet measured (`0`) keeps the width asked for.
pub fn fitted(width: f32, viewport: f32) -> f32 {
    if viewport > 0. { width.min(viewport) } else { width }
}
/// `width` moved by a drag of `delta`, kept between the least and the most.
pub fn resized(width: f32, delta: f32) -> f32 {
    (width + delta).clamp(MIN_WIDTH, MAX_WIDTH)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Column {
    pub width: f32,
    /// The space before it; 0 for the first.
    pub gap_before: f32,
}

/// Where the columns sit in a row that scrolls.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Geometry {
    columns: Vec<Column>,
    /// Each column's left edge, and the row's end at the last place.
    lefts: Vec<f32>,
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
        let first = (0..self.len()).find(|&i| self.right(i) > from).unwrap_or(self.len());
        let last = (first..self.len()).find(|&i| self.left(i) >= to).unwrap_or(self.len());
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

/// The panels in groups. Grouped, the panels of one project sit together and the groups follow the
/// sidebar's order of projects (`project_order`); a project the sidebar does not know comes after those it
/// does, in the order its first panel was opened. Ungrouped, one group holds them all in open order.
pub fn arrange(panel_projects: &[SharedString], project_order: &[SharedString], grouped: bool) -> Vec<Group> {
    if !grouped {
        return if panel_projects.is_empty() { Vec::new() } else { vec![Group { project: None, members: (0..panel_projects.len()).collect() }] };
    }
    let rank = |project: &SharedString| project_order.iter().position(|p| p == project).unwrap_or(usize::MAX);
    let mut groups: Vec<Group> = Vec::new();
    for (i, project) in panel_projects.iter().enumerate() {
        match groups.iter_mut().find(|g| g.project.as_ref() == Some(project)) {
            Some(group) => group.members.push(i),
            None => groups.push(Group { project: Some(project.clone()), members: vec![i] }),
        }
    }
    groups.sort_by_key(|g| g.project.as_ref().map_or(usize::MAX, rank));
    groups
}

/// The panels in the order the strip shows them.
pub fn flat(groups: &[Group]) -> Vec<usize> {
    groups.iter().flat_map(|g| g.members.iter().copied()).collect()
}

/// The columns for `groups`, with `width_of(panel)` for each panel.
pub fn columns(groups: &[Group], width_of: impl Fn(usize) -> f32) -> Vec<Column> {
    let mut out = Vec::new();
    for (g, group) in groups.iter().enumerate() {
        for (m, &panel) in group.members.iter().enumerate() {
            let gap_before = match (g, m) {
                (0, 0) => 0.,
                (_, 0) => GROUP_GAP,
                _ => GAP,
            };
            out.push(Column { width: width_of(panel), gap_before });
        }
    }
    out
}


/// How wide the fade at a strip's edge is.
pub const FADE: f32 = 24.;
/// How strongly the strip's left and right edges fade at scroll `offset` of a row that scrolls `max` at most: 0 where
/// nothing lies beyond the edge, 1 from one fade's width of row beyond it, between as the last of it goes.
pub fn edge_fades(offset: f32, max: f32) -> (f32, f32) {
    if max <= 0. {
        return (0., 0.);
    }
    let offset = offset.clamp(0., max);
    ((offset / FADE).min(1.), ((max - offset) / FADE).min(1.))
}

#[cfg(test)]
mod tests;
