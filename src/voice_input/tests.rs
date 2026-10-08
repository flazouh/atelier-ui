use super::*;

#[test]
fn the_clock_counts_minutes_and_pads_seconds() {
    assert_eq!(clock(0), "0:00");
    assert_eq!(clock(7), "0:07");
    assert_eq!(clock(75), "1:15");
    assert_eq!(clock(600), "10:00");
}

#[test]
fn the_ring_starts_at_the_disc_and_thins_as_it_grows() {
    let (start_opacity, start_reach) = ring_at(0., 1.);
    let (end_opacity, end_reach) = ring_at(0.99, 1.);
    assert!(start_opacity > end_opacity);
    assert!(start_reach < 0.01 && end_reach > RING_REACH * 0.95);
}

#[test]
fn the_ring_waits_for_the_stop_square() {
    assert_eq!(ring_at(0.3, 0.).0, 0.);
    assert!(ring_at(0.3, 0.5).0 < ring_at(0.3, 1.).0);
}

#[test]
fn the_ring_breathes_again_and_again() {
    assert!(ring_phase(0.) < 1e-6);
    assert!((ring_phase(RING_SECONDS * 2.5) - 0.5).abs() < 1e-3);
}

#[test]
fn every_mode_has_its_own_key_so_the_bar_morphs_between_them() {
    let keys = [
        key(VoiceMode::Idle),
        key(VoiceMode::Setup),
        key(VoiceMode::Listening),
    ];
    assert_eq!(
        keys.iter().collect::<std::collections::HashSet<_>>().len(),
        3
    );
}
