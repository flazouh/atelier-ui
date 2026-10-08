use super::*;

#[test]
fn a_size_is_scaled_by_the_zoom_and_comes_back() {
    assert_eq!(
        f32::from(px(14.)),
        14.,
        "at the design size it is the design size"
    );
    set_zoom(1.5);
    assert_eq!(f32::from(px(14.)), 21.);
    assert_eq!(design(gpui_kit::px(21.)), 14.);
    set_zoom(1.);
}

#[test]
fn the_zoom_stays_between_the_ends_and_on_a_tenth() {
    assert_eq!(set_zoom(9.), MAX);
    assert_eq!(set_zoom(0.1), MIN);
    assert!((set_zoom(1.23) - 1.2).abs() < 1e-5);
    set_zoom(1.);
}

/// Each thread has its own zoom, so one test's zoom is not another's.
#[test]
fn a_zoom_on_one_thread_is_not_the_zoom_of_another() {
    set_zoom(2.);
    let other = std::thread::spawn(zoom).join().unwrap();
    assert_eq!(other, 1.);
    set_zoom(1.);
}
