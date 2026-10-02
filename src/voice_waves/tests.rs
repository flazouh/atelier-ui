use super::*;

#[test]
fn every_ribbon_tapers_to_nothing_at_both_ends() {
    for ribbon in RIBBONS {
        for phase in [0., 1., 4.] {
            assert!(amplitude(ribbon, 0., 1., phase) < 1e-3);
            assert!(amplitude(ribbon, 1., 1., phase) < 1e-3);
        }
    }
}

#[test]
fn a_louder_voice_makes_taller_waves() {
    let ribbon = RIBBONS[0];
    let peak = |level| (0..=40).map(|k| amplitude(ribbon, k as f32 / 40., level, 0.7)).fold(0., f32::max);
    assert!(peak(0.9) > peak(0.4) * 1.5);
    assert!(peak(0.4) > peak(0.1));
}

#[test]
fn quiet_still_leaves_a_thin_line() {
    let peak = (0..=40).map(|k| amplitude(RIBBONS[0], k as f32 / 40., 0., 0.)).fold(0., f32::max);
    assert!(peak > 0. && peak <= FLOOR + 1e-6);
}

#[test]
fn amplitude_stays_inside_the_field() {
    for ribbon in RIBBONS {
        for k in 0..=100 {
            let a = amplitude(ribbon, k as f32 / 100., 5., k as f32);
            assert!((0. ..=1.).contains(&a));
        }
    }
}

#[test]
fn the_level_rises_faster_than_it_falls() {
    let up = smooth(0., 1., 0.05);
    let down = 1. - smooth(1., 0., 0.05);
    assert!(up > down * 2.);
    assert!(smooth(0.5, 0.5, 0.05) - 0.5 < 1e-6);
}
