use std::time::{Duration, Instant};

use super::*;

fn at(t0: Instant, ms: u64) -> Instant {
    t0 + Duration::from_millis(ms)
}

fn alphas(ink: &Ink) -> Vec<f32> {
    ink.words().iter().map(|w| w.alpha).collect()
}

#[test]
fn a_new_word_starts_unseen_and_fades_in() {
    let t0 = Instant::now();
    let mut ink = Ink::new(t0);
    ink.observe("Okay so");
    assert_eq!(alphas(&ink), [0., 0.]);
    ink.step(at(t0, 50));
    let half = alphas(&ink);
    assert!(half.iter().all(|a| *a > 0.3 && *a < 0.9), "{half:?}");
    ink.step(at(t0, 400));
    assert_eq!(
        alphas(&ink),
        [1., 1.],
        "settled, and no frame is wanted any more"
    );
    assert!(!ink.moving());
}

#[test]
fn words_that_stay_are_not_touched_by_the_next_partial() {
    let t0 = Instant::now();
    let mut ink = Ink::new(t0);
    ink.observe("Okay so");
    ink.step(at(t0, 400));
    ink.observe("Okay so here is");
    assert_eq!(alphas(&ink), [1., 1., 0., 0.]);
}

#[test]
fn a_rewritten_word_dips_and_fades_back_in() {
    let t0 = Instant::now();
    let mut ink = Ink::new(t0);
    ink.observe("session type");
    ink.step(at(t0, 400));
    ink.observe("session title helper");
    let a = alphas(&ink);
    assert_eq!(a[0], 1.);
    assert!(a[1] < 0.5 && a[1] > 0., "dips, but does not vanish: {a:?}");
    assert_eq!(a[2], 0.);
    ink.step(at(t0, 900));
    assert_eq!(alphas(&ink), [1., 1., 1.]);
}

#[test]
fn a_word_that_was_still_coming_in_does_not_jump_up_when_rewritten() {
    let t0 = Instant::now();
    let mut ink = Ink::new(t0);
    ink.observe("a b");
    ink.step(at(t0, 20));
    let before = alphas(&ink)[1];
    ink.observe("a c");
    assert!(alphas(&ink)[1] <= before);
}

#[test]
fn a_shorter_text_drops_the_words_it_no_longer_has() {
    let t0 = Instant::now();
    let mut ink = Ink::new(t0);
    ink.observe("it is Mm-hmm.");
    ink.step(at(t0, 400));
    ink.observe("it is");
    assert_eq!(ink.words().len(), 2);
}

#[test]
fn the_words_know_where_they_sit_in_the_text() {
    let t0 = Instant::now();
    let mut ink = Ink::new(t0);
    ink.observe("Look at  the");
    let spans: Vec<&str> = ink
        .words()
        .iter()
        .map(|w| &"Look at  the"[w.range.clone()])
        .collect();
    assert_eq!(spans, ["Look", "at", "the"]);
}

#[test]
fn with_reduced_motion_every_word_is_whole_at_once() {
    let t0 = Instant::now();
    let mut ink = Ink::new(t0).still();
    ink.observe("a b");
    assert_eq!(alphas(&ink), [1., 1.]);
    ink.observe("a c d");
    assert_eq!(alphas(&ink), [1., 1., 1.]);
    assert!(!ink.moving());
}
