use std::time::Instant;

use gpui_kit::rgb;

use super::*;
use crate::sprite::Strip;

fn labels() -> PhaseLabels {
    PhaseLabels::default()
}

fn thinking(elapsed_s: f32) -> SharedString {
    label(&labels(), ThinkingPhase::Thinking { since: Instant::now() }, elapsed_s)
}

#[test]
fn the_label_changes_at_each_threshold() {
    assert_eq!(thinking(0.), "Thinking…");
    assert_eq!(thinking(14.9), "Thinking…");
    assert_eq!(thinking(15.), "Still thinking…");
    assert_eq!(thinking(29.9), "Still thinking…");
    assert_eq!(thinking(30.), "Thinking more…");
    assert_eq!(thinking(45.), "Thinking some more…");
    assert_eq!(thinking(60.), "Almost done thinking…");
    assert_eq!(thinking(600.), "Almost done thinking…");
}

#[test]
fn a_finished_thought_names_its_length() {
    assert_eq!(label(&labels(), ThinkingPhase::Thought { seconds: 4 }, 99.), "Thought for 4s");
    assert_eq!(label(&labels(), ThinkingPhase::Thought { seconds: 0 }, 99.), "Thought", "no time to tell, so none is claimed");
}

#[test]
fn other_phases_have_fixed_labels() {
    let cases = [
        (ThinkingPhase::Connecting, "Connecting…"),
        (ThinkingPhase::Sending, "Sending…"),
        (ThinkingPhase::Starting, "Starting session…"),
        (ThinkingPhase::Preparing, "Preparing…"),
        (ThinkingPhase::Waiting, "Waiting for the agent…"),
        (ThinkingPhase::RunningTools, "Running tools…"),
    ];
    for (phase, text) in cases {
        assert_eq!(label(&labels(), phase, 99.), text, "{phase:?}");
    }
}

#[test]
fn the_label_wakes_at_the_next_threshold_only_while_thinking() {
    let since = Instant::now();
    let labels = labels();
    assert_eq!(next_label_change_s(&labels, ThinkingPhase::Thinking { since }, 0.), Some(15.));
    assert_eq!(next_label_change_s(&labels, ThinkingPhase::Thinking { since }, 44.), Some(1.));
    assert_eq!(next_label_change_s(&labels, ThinkingPhase::Thinking { since }, 60.), None);
    assert_eq!(next_label_change_s(&labels, ThinkingPhase::Sending, 0.), None);
}

#[test]
fn tokens_and_tasks_read_as_short_counts() {
    assert_eq!(tokens_text(0), "0 tokens");
    assert_eq!(tokens_text(1), "1 token");
    assert_eq!(tokens_text(999), "999 tokens");
    assert_eq!(tokens_text(1_234), "1.2k tokens");
    assert_eq!(tokens_text(12_000), "12k tokens");
    assert_eq!(tasks_text(1), "1 task");
    assert_eq!(tasks_text(3), "3 tasks");
}

#[test]
fn segments_are_spaced_without_a_middle_dot() {
    assert!(!SEGMENT_GAP_TEXT.contains('·'));
    assert_eq!(segment_text("4s"), "4s");
}

/// A look from some other agent: its own strips, colours, and words.
fn other_look() -> AgentLook {
    let strip = |path| Strip { path, bytes: b"", frames: 4, frame_ms: 50, loops: true };
    AgentLook {
        mark: Mark { working: strip("other/work.svg"), orbiting: strip("other/orbit.svg"), color: rgb(0x2255AA).into(), icon_frame: 0 },
        message: rgb(0x0044CC).into(),
        glimmer: rgb(0x66AAFF).into(),
        labels: PhaseLabels { waiting: "Waiting for Other…".into(), thinking: &[(0., "Pondering…")], ..PhaseLabels::default() },
    }
}

#[test]
fn the_main_row_orbits_while_subagents_run() {
    let mark = other_look().mark;
    assert_eq!(mark_strip(&mark, false, 0), mark.working);
    assert_eq!(mark_strip(&mark, false, 2), mark.orbiting);
    assert_eq!(mark_strip(&mark, true, 2), mark.working);
}

#[test]
fn another_agents_look_reaches_the_thinking_label() {
    let look = other_look();
    let muted = rgb(0x888888).into();
    assert_eq!(label(&look.labels, ThinkingPhase::Waiting, 0.), "Waiting for Other…");
    assert_eq!(label(&look.labels, ThinkingPhase::Thinking { since: Instant::now() }, 90.), "Pondering…");
    assert_eq!(label_color(&look, true, muted), look.message);
    assert_eq!(label_color(&look, false, muted), muted);
    // The glimmer walks across the label in the look's own two colours.
    let lit = glimmer::glimmer_highlights("Pondering…", look.message, look.glimmer, |g| if g == 0 { 1. } else { 0. });
    assert!(same_color(lit[0].1.color, look.glimmer), "{:?}", lit[0].1.color);
    assert!(same_color(lit[1].1.color, look.message), "{:?}", lit[1].1.color);
}

/// `mix` goes through RGB and back, so compare with a little slack.
fn same_color(got: Option<Hsla>, want: Hsla) -> bool {
    let (Some(got), want) = (got.map(|c| c.to_rgb()), want.to_rgb()) else { return false };
    [got.r - want.r, got.g - want.g, got.b - want.b, got.a - want.a].iter().all(|d| d.abs() < 1e-3)
}

#[test]
fn the_loading_mark_picks_one_of_the_strips_it_was_given() {
    let strip = |path| Strip { path, bytes: b"", frames: 4, frame_ms: 50, loops: true };
    let working = strip("other/work.svg");
    let variants = [strip("a.svg"), strip("b.svg"), strip("c.svg")];
    for roll in 0..9 {
        assert_eq!(loading_strip(&variants, working, roll), variants[(roll % 3) as usize]);
    }
    // Different rolls reach every strip.
    let seen: std::collections::HashSet<_> = (0..3).map(|roll| loading_strip(&variants, working, roll).path).collect();
    assert_eq!(seen.len(), 3);
}

#[test]
fn without_variants_the_loading_mark_plays_the_working_strip() {
    let working = other_look().mark.working;
    assert_eq!(loading_strip(&[], working, 7), working);
}

#[test]
fn rolls_differ_between_rows() {
    let rolls: std::collections::HashSet<_> = (0..8).map(|_| roll()).collect();
    assert!(rolls.len() > 1);
}
