use gpui_kit::SharedString;

use super::{Column, DEFAULT_WIDTH, GAP, GROUP_GAP, Geometry, MAX_WIDTH, MIN_WIDTH, arrange, columns, fitted, flat, resized};

fn geometry(widths: &[f32]) -> Geometry {
    Geometry::new(widths.iter().enumerate().map(|(i, w)| Column { width: *w, gap_before: if i == 0 { 0. } else { 10. } }).collect())
}

fn names(list: &[&str]) -> Vec<SharedString> {
    list.iter().map(|s| SharedString::from(s.to_string())).collect()
}

#[test]
fn columns_sit_left_to_right_with_their_gaps() {
    let g = geometry(&[100., 200., 100.]);
    assert_eq!((g.left(0), g.left(1), g.left(2)), (0., 110., 320.));
    assert_eq!((g.right(0), g.right(2), g.total()), (100., 420., 420.));
    assert_eq!(g.len(), 3);
    assert_eq!(g.width_of(1), 200.);
}

#[test]
fn the_row_scrolls_only_as_far_as_it_is_wider_than_the_viewport() {
    let g = geometry(&[100., 200., 100.]);
    assert_eq!(g.max_offset(300.), 120.);
    assert_eq!(g.max_offset(500.), 0.);
    assert_eq!((g.clamp(-5., 300.), g.clamp(50., 300.), g.clamp(999., 300.)), (0., 50., 120.));
    assert_eq!(Geometry::default().max_offset(100.), 0.);
}

#[test]
fn only_the_columns_the_viewport_shows_are_visible_and_a_margin_adds_the_next() {
    // Twelve columns of 100 with gaps of 10: 110 apart.
    let g = geometry(&[100.; 12]);
    assert_eq!(g.visible(0., 300., 0.), 0..3);
    assert_eq!(g.visible(0., 300., 50.), 0..4, "a margin shows one more");
    assert_eq!(g.visible(500., 300., 0.), 4..8);
    assert_eq!(g.visible(g.max_offset(300.), 300., 0.), 9..12);
    assert_eq!(g.visible(0., 5000., 0.), 0..12);
    assert_eq!(Geometry::default().visible(0., 300., 100.), 0..0);
}

#[test]
fn a_column_only_partly_in_view_counts_and_one_just_out_of_it_does_not() {
    let g = geometry(&[100.; 4]);
    assert_eq!(g.visible(95., 100., 0.), 0..2, "the first ends at 100, past the viewport's left edge");
    assert_eq!(g.visible(100., 100., 0.), 1..2, "the first ends where the viewport begins");
    assert_eq!(g.visible(0., 110., 0.), 0..1, "the second begins where the viewport ends");
}

#[test]
fn a_scroll_settles_on_the_nearest_column_edge_or_the_end() {
    let g = geometry(&[100.; 6]); // lefts 0, 110, 220, 330, 440, 550; total 650
    assert_eq!(g.snap(0., 300.), 0.);
    assert_eq!(g.snap(50., 300.), 0.);
    assert_eq!(g.snap(60., 300.), 110.);
    assert_eq!(g.snap(230., 300.), 220.);
    assert_eq!(g.snap(340., 300.), 330.);
    assert_eq!(g.snap(500., 300.), 350., "the end of the row, 650 - 300, is nearer than a column edge");
    assert_eq!(g.snap(9999., 300.), 350.);
    assert_eq!(geometry(&[100.]).snap(40., 300.), 0.);
    assert_eq!(Geometry::default().snap(10., 300.), 0.);
}

#[test]
fn revealing_a_column_scrolls_the_least_that_shows_it_whole() {
    let g = geometry(&[100.; 6]);
    assert_eq!(g.reveal(0., 300., 1), 0., "already whole");
    assert_eq!(g.reveal(0., 300., 3), 130., "scrolled right until its right edge is at the viewport's");
    assert_eq!(g.reveal(300., 300., 1), 110., "scrolled left until its left edge is at the viewport's");
    assert_eq!(g.reveal(0., 300., 5), 350., "clamped at the end of the row");
    let wide = geometry(&[500., 100.]);
    assert_eq!(wide.reveal(0., 300., 0), 0.);
    assert_eq!(wide.reveal(200., 300., 0), 0., "wider than the viewport: its left edge");
}

#[test]
fn the_column_under_a_point_is_found_and_the_gap_is_no_column() {
    let g = geometry(&[100., 100.]);
    assert_eq!((g.column_at(0.), g.column_at(99.9), g.column_at(105.), g.column_at(110.), g.column_at(210.)), (Some(0), Some(0), None, Some(1), None));
}

#[test]
fn a_drag_resizes_a_column_within_its_least_and_most() {
    assert_eq!(resized(DEFAULT_WIDTH, 30.), DEFAULT_WIDTH + 30.);
    assert_eq!(resized(DEFAULT_WIDTH, -1000.), MIN_WIDTH);
    assert_eq!(resized(DEFAULT_WIDTH, 5000.), MAX_WIDTH);
}

#[test]
fn panels_of_one_project_sit_together_and_the_groups_follow_the_sidebar() {
    let panels = names(&["web", "api", "web", "docs", "api", "web"]);
    let order = names(&["api", "web"]);
    let groups = arrange(&panels, &order, true);
    let shape: Vec<_> = groups.iter().map(|g| (g.project.as_ref().map(|p| p.to_string()), g.members.clone())).collect();
    assert_eq!(
        shape,
        [
            (Some("api".to_string()), vec![1, 4]),
            (Some("web".to_string()), vec![0, 2, 5]),
            (Some("docs".to_string()), vec![3]),
        ],
        "a project the sidebar does not know comes last"
    );
    assert_eq!(flat(&groups), [1, 4, 0, 2, 5, 3]);
}

#[test]
fn projects_the_sidebar_does_not_know_keep_the_order_they_were_first_opened() {
    let groups = arrange(&names(&["b", "a", "b"]), &[], true);
    assert_eq!(groups.iter().map(|g| g.project.clone().unwrap().to_string()).collect::<Vec<_>>(), ["b", "a"]);
}

#[test]
fn ungrouped_is_one_flat_row_in_open_order() {
    let groups = arrange(&names(&["web", "api", "web"]), &names(&["api", "web"]), false);
    assert_eq!(groups.len(), 1);
    assert_eq!((groups[0].project.clone(), flat(&groups)), (None, vec![0, 1, 2]));
    assert!(arrange(&[], &[], false).is_empty() && arrange(&[], &[], true).is_empty());
}

#[test]
fn a_group_starts_a_wider_gap_than_a_column_in_a_group() {
    let groups = arrange(&names(&["a", "a", "b"]), &names(&["a", "b"]), true);
    let cols = columns(&groups, |panel| 100. + panel as f32);
    assert_eq!(
        cols,
        vec![
            Column { width: 100., gap_before: 0. },
            Column { width: 101., gap_before: GAP },
            Column { width: 102., gap_before: GROUP_GAP },
        ]
    );
}

#[test]
fn twelve_panels_have_the_geometry_of_a_long_row() {
    let panels: Vec<SharedString> = (0..12).map(|i| format!("p{}", i % 3).into()).collect();
    let order = names(&["p0", "p1", "p2"]);
    let groups = arrange(&panels, &order, true);
    let cols = columns(&groups, |_| DEFAULT_WIDTH);
    let g = Geometry::new(cols);
    assert_eq!(g.len(), 12);
    assert_eq!(g.total(), 12. * DEFAULT_WIDTH + 9. * GAP + 2. * GROUP_GAP);
}
/// A column never gets wider than the strip that shows it, so no panel draws under the next pane; a
/// strip not yet measured keeps the width asked for.
#[test]
fn a_column_fits_its_strip() {
    assert_eq!(fitted(480., 444.), 444.);
    assert_eq!(fitted(480., 900.), 480.);
    assert_eq!(fitted(480., 0.), 480.);
}

/// The strip fades an edge by how far the row runs beyond it, up to the fade's width, and not at all where it does not.
#[test]
fn an_edge_fades_by_how_much_row_lies_beyond_it() {
    use super::edge_fades;
    assert_eq!(edge_fades(0., 500.), (0., 1.), "at the start");
    assert_eq!(edge_fades(500., 500.), (1., 0.), "at the end");
    assert_eq!(edge_fades(250., 500.), (1., 1.), "in the middle");
    assert_eq!(edge_fades(0., 0.), (0., 0.), "a row that fits has no edge to fade");
    assert_eq!(edge_fades(12., 500.), (0.5, 1.), "half the fade's width to go on the left");
    assert_eq!(edge_fades(494., 500.), (1., 0.25), "six pixels to go on the right");
    assert_eq!(edge_fades(-3., 500.), (0., 1.), "an overshoot does not go negative");
}
