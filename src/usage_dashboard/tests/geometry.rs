use gpui_kit::SharedString;

use crate::usage_dashboard::{
    Series, UsageSource, SourceKind, axis, bar_segments, mini_bars, spark_points, tokens_label,
};

const CLAUDE: Series = Series::new(0, 0);
const CODEX: Series = Series::new(1, 0);
const ROUTER: Series = Series::new(2, 0);

#[test]
fn segments_add_up_to_the_day_and_touch() {
    let parts = [(CLAUDE, 3_400_000), (CODEX, 1_300_000), (ROUTER, 640_000)];
    let per_token = 188. / 8_000_000.;
    let segments = bar_segments(&parts, per_token, 4.);
    let sum: f32 = segments.iter().map(|s| s.height).sum();
    assert_eq!(segments.len(), 3);
    assert!((sum - 5_340_000. * per_token).abs() <= 0.5, "sum {sum}");
    // Every height is a whole pixel run, so the edges meet with no gap.
    assert!(segments.iter().all(|s| s.height.fract() == 0.));
}

#[test]
fn the_tallest_day_is_the_tallest_bar_and_the_rest_scale_to_it() {
    let scale = axis(7_100_000);
    let per_token = 188. / scale.top as f32;
    let tall: f32 = bar_segments(&[(CLAUDE, 7_100_000)], per_token, 4.).iter().map(|s| s.height).sum();
    let half: f32 = bar_segments(&[(CLAUDE, 3_550_000)], per_token, 4.).iter().map(|s| s.height).sum();
    assert!(tall <= 188.);
    assert!((tall / 2. - half).abs() <= 1.);
}

#[test]
fn only_the_top_segment_has_rounded_top_corners() {
    let parts = [(CLAUDE, 5_000_000), (CODEX, 2_000_000), (ROUTER, 1_000_000)];
    let segments = bar_segments(&parts, 188. / 8_000_000., 4.);
    let radii: Vec<f32> = segments.iter().map(|s| s.top_radius).collect();
    assert_eq!(radii, vec![0., 0., 4.]);
}

#[test]
fn a_short_top_segment_does_not_round_past_its_height() {
    let segments = bar_segments(&[(CLAUDE, 8_000_000), (CODEX, 80_000)], 188. / 8_000_000., 4.);
    assert_eq!(segments.last().unwrap().series, CODEX);
    assert!(segments.last().unwrap().top_radius <= segments.last().unwrap().height);
}

#[test]
fn a_part_with_nothing_is_left_out_and_a_lone_part_is_rounded() {
    let segments = bar_segments(&[(CLAUDE, 4_000_000), (CODEX, 0)], 188. / 8_000_000., 4.);
    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].top_radius, 4.);
    assert!(bar_segments(&[], 1., 4.).is_empty());
}

#[test]
fn the_axis_tops_at_a_round_number_that_holds_the_tallest_day() {
    assert_eq!(axis(7_100_000).top, 8_000_000);
    assert_eq!(axis(7_100_000).ticks, vec![2_000_000, 4_000_000, 6_000_000, 8_000_000]);
    assert_eq!(axis(8_000_000).top, 8_000_000);
    assert!(axis(0).top > 0);
    for max in [1, 999, 123_456, 9_999_999, 41_200_000] {
        assert!(axis(max).top >= max);
    }
}

#[test]
fn tokens_read_as_the_axis_says_them() {
    assert_eq!(tokens_label(2_000_000), "2 M");
    assert_eq!(tokens_label(1_500_000), "1.5 M");
    assert_eq!(tokens_label(500_000), "500 k");
    assert_eq!(tokens_label(42), "42");
}

#[test]
fn a_mini_chart_scales_to_its_tallest_and_rounds_only_the_top() {
    let bars = mini_bars(&[1., 4., 2., 0.], 100., 3.);
    assert_eq!(bars[1].height, 100.);
    assert_eq!(bars[0].height, 25.);
    assert_eq!(bars[3].height, 0.);
    assert!(bars[..3].iter().all(|b| b.top_radius == 3.));
    assert_eq!(bars[3].top_radius, 0.);
    assert!(mini_bars(&[f32::NAN, -1.], 100., 3.).iter().all(|b| b.height == 0.));
}

#[test]
fn a_sparkline_stays_in_its_box() {
    for points in [spark_points(&[1., 3., 2.]), spark_points(&[5.]), spark_points(&[0., 0.])] {
        assert!(points.iter().all(|(x, y)| (0. ..=1.).contains(x) && (0. ..=1.).contains(y)));
    }
    assert!(spark_points(&[]).is_empty());
    assert_eq!(spark_points(&[1., 3., 2.]).len(), 3);
    assert_eq!(spark_points(&[5.]).len(), 2);
}

fn source(id: &str, group: &str, series: Series) -> UsageSource {
    UsageSource {
        id: id.into(),
        name: id.into(),
        caption: SharedString::default(),
        group: group.into(),
        series,
        limit: None,
        value: SharedString::default(),
        note: SharedString::default(),
        kind: SourceKind::Key,
    }
}

#[test]
fn sources_in_a_row_with_one_caption_stand_in_one_group() {
    use crate::usage_dashboard::helpers::{groups, is_selected, legend};
    use crate::usage_dashboard::Selection;
    let sources = vec![
        source("a", "Claude Code · 2 accounts", Series::new(0, 0)),
        source("b", "Claude Code · 2 accounts", Series::new(0, 1)),
        source("c", "Codex", Series::new(1, 0)),
    ];
    let grouped = groups(&sources);
    assert_eq!(grouped.iter().map(|g| (g.first, g.len)).collect::<Vec<_>>(), vec![(0, 2), (2, 1)]);
    let names: Vec<_> = legend(&sources).iter().map(|e| (e.name.to_string(), e.shades.clone())).collect();
    assert_eq!(names, vec![("Claude Code".to_string(), vec![0, 1]), ("Codex".to_string(), vec![0])]);
    assert!(is_selected(&Selection::Group("Codex".into()), &sources[2]));
    assert!(is_selected(&Selection::Source("b".into()), &sources[1]));
    assert!(!is_selected(&Selection::Source("b".into()), &sources[0]));
}
