use super::*;
use std::time::Duration;

fn delays(count: usize) -> Vec<u64> {
    (0..count).map(|i| stagger_delay(i, count).as_millis() as u64).collect()
}

#[test]
fn no_items_have_no_delays() {
    assert!(delays(0).is_empty());
}

#[test]
fn one_item_starts_at_once() {
    assert_eq!(delays(1), [0]);
}

#[test]
fn five_items_step_by_35ms_until_the_cap() {
    assert_eq!(delays(5), [0, 35, 70, 100, 100]);
}

#[test]
fn twenty_items_never_start_later_than_the_cap() {
    let d = delays(20);
    assert_eq!(&d[..4], [0, 35, 70, 100]);
    assert!(d.iter().all(|&ms| ms <= 100));
    // The last item still settles within the stagger cap.
    let last = stagger_delay(19, 20) + duration::ENTER;
    assert!(last <= STAGGER_CAP, "last item settles at {last:?}");
}

#[test]
fn an_index_past_the_count_is_treated_as_the_last_item() {
    assert_eq!(stagger_delay(40, 3), stagger_delay(2, 3));
}

#[test]
fn an_arriving_item_starts_hidden_and_six_pixels_low() {
    assert_eq!(frame(0., false), EntranceFrame { opacity: 0., y: ENTER_RISE });
}

#[test]
fn halfway_it_is_half_faded_and_half_risen() {
    assert_eq!(frame(0.5, false), EntranceFrame { opacity: 0.5, y: ENTER_RISE / 2. });
}

#[test]
fn at_the_end_it_has_settled() {
    assert_eq!(frame(1., false), EntranceFrame { opacity: 1., y: 0. });
}

#[test]
fn under_reduce_motion_it_only_fades() {
    assert_eq!(frame(0., true), EntranceFrame { opacity: 0., y: 0. });
    assert_eq!(frame(0.5, true), EntranceFrame { opacity: 0.5, y: 0. });
}

#[test]
fn the_curve_is_the_morph_ease_over_200ms_or_120ms_under_reduce_motion() {
    assert_eq!(curve(false), Curve::Ease(0.2, ease::MORPH));
    assert_eq!(curve(true), Curve::Ease(0.12, ease::MORPH));
    assert_eq!(duration::ENTER, Duration::from_millis(200));
    assert_eq!(duration::ENTER_REDUCED, Duration::from_millis(120));
    assert_eq!((STAGGER_STEP, STAGGER_CAP), (Duration::from_millis(35), Duration::from_millis(300)));
}

fn ids(names: &[&'static str]) -> Vec<ElementId> {
    names.iter().map(|&n| ElementId::from(n)).collect()
}

#[test]
fn items_on_the_first_paint_do_not_animate() {
    let mut list = Arrivals::default();
    assert_eq!(list.arrive(&ids(&["a", "b", "c"])), [None, None, None]);
}

#[test]
fn items_added_later_enter_with_a_stagger() {
    let mut list = Arrivals::default();
    list.arrive(&ids(&["a"]));
    let ms = |d: Option<Duration>| d.map(|d| d.as_millis());
    let got: Vec<_> = list.arrive(&ids(&["a", "b", "c"])).into_iter().map(ms).collect();
    assert_eq!(got, [None, Some(0), Some(35)]);
}

#[test]
fn an_id_that_already_showed_never_animates_again() {
    let mut list = Arrivals::default();
    list.arrive(&[]);
    list.arrive(&ids(&["a"]));
    list.arrive(&[]);
    assert_eq!(list.arrive(&ids(&["a"])), [None]);
}
