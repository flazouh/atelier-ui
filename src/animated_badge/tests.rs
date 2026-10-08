use super::*;

#[test]
fn the_two_sizes_are_the_old_badge_and_one_step_up() {
    assert_eq!(BadgeSize::Small.metrics(), (20., 8., 4., 11., 12.));
    assert_eq!(BadgeSize::Medium.metrics(), (24., 10., 6., 12., 14.));
}

#[test]
fn the_pulse_swells_from_8_to_16_percent_and_back_over_1600_ms() {
    assert!((pulse_at(0) - 0.08).abs() < 1e-4);
    assert!((pulse_at(800) - 0.16).abs() < 1e-3);
    assert!((pulse_at(1599) - 0.08).abs() < 0.005);
}

#[test]
fn every_status_has_words_and_a_tinted_fill_with_no_border_and_the_words_read_on_the_fill() {
    use crate::theme::{TEXT_CONTRAST, contrast, mix};
    for theme in crate::themes::all() {
        for status in [
            BadgeStatus::Neutral,
            BadgeStatus::Info,
            BadgeStatus::Success,
            BadgeStatus::Warning,
            BadgeStatus::Danger,
            BadgeStatus::Loading,
        ] {
            let (ink, fill) = colors(status, theme);
            assert_ne!(ink, fill);
            // The fill is a wash over the page or card; the words on it keep the contrast the themes promise for them.
            let under = mix(theme.card, fill, fill.a);
            assert!(
                contrast(ink, under) >= TEXT_CONTRAST - 1.5,
                "{} {status:?}: {:.2}",
                theme.name,
                contrast(ink, under)
            );
        }
    }
}
