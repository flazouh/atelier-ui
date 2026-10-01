use super::*;

#[test]
fn a_full_partial_line_keeps_every_point() {
    let line = [(0., 0.), (3., 4.), (3., 10.)];
    assert_eq!(partial(&line, 1.), line.to_vec());
}

#[test]
fn half_a_line_stops_at_half_its_length() {
    // Lengths 5 and 6: half of 11 is 5.5, so 0.5 into the second segment.
    let out = partial(&[(0., 0.), (3., 4.), (3., 10.)], 0.5);
    assert_eq!(out.len(), 3);
    assert!((out[2].1 - 4.5).abs() < 1e-4);
}

#[test]
fn an_arc_starts_where_asked_on_the_radius_9_circle() {
    let pts = arc_points(0., 0.25);
    assert!((pts[0].0 - 21.).abs() < 1e-4 && (pts[0].1 - 12.).abs() < 1e-4);
    let last = pts.last().unwrap();
    assert!((last.0 - 12.).abs() < 1e-3 && (last.1 - 21.).abs() < 1e-3);
}

#[test]
fn a_slash_crosses_the_disc_from_corner_to_corner() {
    let full = partial(&SLASH, 1.);
    assert_eq!(full.len(), 2);
    assert_eq!(full[0], SLASH[0]);
    assert_eq!(full[1], SLASH[1]);
}

#[test]
fn a_filled_mark_knocks_its_glyph_out_in_the_page_color() {
    let page: Hsla = gpui_kit::rgb(0x151515).into();
    let mark = Mark::filled(gpui_kit::rgb(0xB0B8B8).into(), page);
    assert_eq!(mark.fill_alpha, 1.);
    assert_eq!(mark.ring_alpha, 0.);
    assert_eq!(mark.glyph, Some(page));
}
