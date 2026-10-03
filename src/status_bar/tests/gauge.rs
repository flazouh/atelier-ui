use crate::status_bar::{Gauge, Pressure};

#[test]
fn a_gauge_is_calm_warm_and_hot_by_how_much_is_used() {
    assert_eq!(Gauge::new("5h", 0.0, None).pressure(), Pressure::Calm);
    assert_eq!(Gauge::new("5h", 0.59, None).pressure(), Pressure::Calm);
    assert_eq!(Gauge::new("5h", 0.6, None).pressure(), Pressure::Warm);
    assert_eq!(Gauge::new("5h", 0.85, None).pressure(), Pressure::Hot);
    assert_eq!(Gauge::new("5h", 1.0, None).pressure(), Pressure::Hot);
}

#[test]
fn a_fraction_outside_the_range_is_held_in_it() {
    assert_eq!(Gauge::new("7d", 1.4, None).fraction(), 1.);
    assert_eq!(Gauge::new("7d", -0.2, None).fraction(), 0.);
    assert_eq!(Gauge::new("7d", f32::NAN, None).fraction(), 0.);
}

#[test]
fn a_gauge_says_its_percent_and_when_it_resets() {
    assert_eq!(Gauge::new("7d", 0.77, None).words(), "7d: 77% used");
    assert_eq!(
        Gauge::new("5h", 1.0, Some(30)).words(),
        "5h: 100% used, resets in under a minute"
    );
    assert_eq!(
        Gauge::new("5h", 0.5, Some(12 * 60 + 5)).words(),
        "5h: 50% used, resets in 12 min"
    );
    assert_eq!(
        Gauge::new("5h", 0.5, Some(2 * 3600 + 5 * 60)).words(),
        "5h: 50% used, resets in 2 h 5 min"
    );
    assert_eq!(
        Gauge::new("7d", 0.5, Some(3 * 86_400 + 4 * 3600)).words(),
        "7d: 50% used, resets in 3 d 4 h"
    );
}
