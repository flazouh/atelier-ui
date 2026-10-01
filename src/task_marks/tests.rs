use super::{label_color, pie, rect, status_color};
use crate::{
    task_model::{Priority, TaskStatus},
    theme::Theme,
};

#[test]
fn a_pie_grows_with_the_progress_and_starts_at_twelve_oclock_from_the_centre() {
    let half = pie(0.5, 5.);
    assert_eq!(half[0], (12., 12.));
    let (x, y) = half[1];
    assert!((x - 12.).abs() < 1e-4 && (y - 7.).abs() < 1e-4, "the first edge points up: {x} {y}");
    let (x, y) = *half.last().unwrap();
    assert!((x - 12.).abs() < 1e-3 && (y - 17.).abs() < 1e-3, "half a turn ends straight down: {x} {y}");
    assert!(pie(0.75, 5.).len() > half.len());
}

#[test]
fn the_rect_helper_lists_its_corners_clockwise() {
    assert_eq!(rect(1., 2., 3., 4.), [(1., 2.), (4., 2.), (4., 6.), (1., 6.)]);
}

#[test]
fn every_status_has_a_colour_and_the_stages_of_work_look_different() {
    let theme = Theme::dark();
    let colors: Vec<_> = TaskStatus::BOARD_ORDER.iter().map(|s| status_color(*s, &theme)).collect();
    assert_eq!(colors[0], colors[5], "not begun and canceled are equally quiet");
    assert_ne!(colors[1], colors[0]);
    assert_ne!(colors[2], colors[3]);
    assert_ne!(colors[2], colors[4]);
    assert_eq!(status_color(TaskStatus::Done, &theme), theme.info);
}

#[test]
fn the_eight_label_tones_repeat_after_eight_and_are_not_all_the_same() {
    let theme = Theme::light();
    assert_eq!(label_color(0, &theme), label_color(8, &theme));
    let distinct = (0..8).map(|t| format!("{:?}", label_color(t, &theme))).collect::<std::collections::HashSet<_>>().len();
    assert!(distinct >= 6, "{distinct} distinct tones");
}

#[test]
fn a_priority_fills_as_many_bars_as_it_says() {
    let filled: Vec<_> = Priority::ALL.iter().map(|p| p.bars()).collect();
    assert_eq!(filled, [0, 3, 2, 1, 0]);
}
