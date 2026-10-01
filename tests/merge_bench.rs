//! What the merge model costs a frame: the blockers, the button and the standing line for one pull
//! request, as MergeBox and MergeButton each work them out on every render. The worst case is every
//! blocker at once. Target: under 50 µs, a fraction of an 8 ms frame.
//!     cargo test --release -p beui --test merge_bench -- --ignored --nocapture

use std::{hint::black_box, time::{Duration, Instant}};

use atelier_ui::merge::{MergeFacts, Queue, ReviewNeed, UpdateWay, blockers, button, first_choice, standing};

#[test]
#[ignore]
fn one_render_of_the_merge_model() {
    let facts = MergeFacts {
        draft: true,
        conflicts: vec!["src/request.rs".into(), "src/relay.rs".into()],
        behind: Some(vec![UpdateWay::Merge, UpdateWay::Rebase]),
        checks_failing: 1,
        checks_running: 2,
        review: ReviewNeed::ChangesAsked(vec!["Ada".into(), "Kai".into()]),
        queue: Some(Queue { queued: false, position: None }),
        auto_merge: Some(false),
        ..MergeFacts::default()
    };
    let choice = first_choice(&facts, None);
    const RUNS: u32 = 1000;
    let mut samples: Vec<Duration> = (0..20)
        .map(|_| {
            let at = Instant::now();
            for _ in 0..RUNS {
                let facts = black_box(&facts);
                black_box((blockers(facts), button(facts, &choice), standing(facts)));
            }
            at.elapsed() / RUNS
        })
        .collect();
    samples.sort();
    let us = |d: Duration| d.as_secs_f64() * 1e6;
    println!("merge model, every blocker: median {:.2} µs, p95 {:.2} µs, target < 50 µs", us(samples[10]), us(samples[18]));
}
