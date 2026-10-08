use gpui_kit::SharedString;

use super::structs::{Column, Group};
use super::types::{FADE, GAP, GROUP_GAP, MAX_WIDTH, MIN_WIDTH};

/// A column's width in a strip `viewport` wide: never wider than the strip, so a panel never draws
/// under the pane beside it. The width asked for stays as it is, for when the strip grows again. A
/// strip not yet measured (`0`) keeps the width asked for.
pub fn fitted(width: f32, viewport: f32) -> f32 {
    if viewport > 0. {
        width.min(viewport)
    } else {
        width
    }
}

/// `width` moved by a drag of `delta`, kept between the least and the most.
pub fn resized(width: f32, delta: f32) -> f32 {
    (width + delta).clamp(MIN_WIDTH, MAX_WIDTH)
}

/// The panels in groups. Grouped, the panels of one project sit together and the groups follow the
/// sidebar's order of projects (`project_order`); a project the sidebar does not know comes after those it
/// does, in the order its first panel was opened. Ungrouped, one group holds them all in open order.
pub fn arrange(
    panel_projects: &[SharedString],
    project_order: &[SharedString],
    grouped: bool,
) -> Vec<Group> {
    if !grouped {
        return if panel_projects.is_empty() {
            Vec::new()
        } else {
            vec![Group {
                project: None,
                members: (0..panel_projects.len()).collect(),
            }]
        };
    }
    let rank = |project: &SharedString| {
        project_order
            .iter()
            .position(|p| p == project)
            .unwrap_or(usize::MAX)
    };
    let mut groups: Vec<Group> = Vec::new();
    for (i, project) in panel_projects.iter().enumerate() {
        match groups
            .iter_mut()
            .find(|g| g.project.as_ref() == Some(project))
        {
            Some(group) => group.members.push(i),
            None => groups.push(Group {
                project: Some(project.clone()),
                members: vec![i],
            }),
        }
    }
    groups.sort_by_key(|g| g.project.as_ref().map_or(usize::MAX, rank));
    groups
}

/// The panels in the order the strip shows them.
pub fn flat(groups: &[Group]) -> Vec<usize> {
    groups
        .iter()
        .flat_map(|g| g.members.iter().copied())
        .collect()
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
            out.push(Column {
                width: width_of(panel),
                gap_before,
            });
        }
    }
    out
}

/// How strongly the strip's left and right edges fade at scroll `offset` of a row that scrolls `max` at most: 0 where
/// nothing lies beyond the edge, 1 from one fade's width of row beyond it, between as the last of it goes.
pub fn edge_fades(offset: f32, max: f32) -> (f32, f32) {
    if max <= 0. {
        return (0., 0.);
    }
    let offset = offset.clamp(0., max);
    ((offset / FADE).min(1.), ((max - offset) / FADE).min(1.))
}
