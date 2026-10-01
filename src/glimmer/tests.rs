use super::*;

// "Thinking…" is 9 clusters wide, so one cycle is 9 + 2 * 10 = 29 steps.
const W: i32 = 9;

#[test]
fn the_glimmer_starts_past_the_right_edge_and_walks_left() {
    assert_eq!(glimmer_index(0, W, false), 19);
    assert_eq!(glimmer_index(199, W, false), 19);
    assert_eq!(glimmer_index(200, W, false), 18);
    assert_eq!(glimmer_index(28 * 200, W, false), -9);
}

#[test]
fn the_glimmer_wraps_after_one_cycle() {
    assert_eq!(glimmer_index(29 * 200, W, false), 19);
    assert_eq!(glimmer_index(30 * 200, W, false), 18);
    assert_eq!(glimmer_index(29 * 50, W, true), -10);
}

#[test]
fn while_requesting_the_glimmer_walks_right_four_times_as_fast() {
    assert_eq!(glimmer_index(0, W, true), -10);
    assert_eq!(glimmer_index(49, W, true), -10);
    assert_eq!(glimmer_index(50, W, true), -9);
    assert_eq!(glimmer_index(28 * 50, W, true), 18);
}

#[test]
fn the_smooth_center_meets_the_stepped_index_on_each_step() {
    for step in 0..60u64 {
        for requesting in [false, true] {
            let ms = step * if requesting { 50 } else { 200 };
            assert_eq!(glimmer_center(ms, W, requesting), glimmer_index(ms, W, requesting) as f32, "{step} {requesting}");
        }
    }
    assert_eq!(glimmer_center(100, W, false), 18.5);
}

#[test]
fn the_band_is_full_at_its_center_and_gone_one_and_a_half_clusters_away() {
    assert_eq!(glimmer_weight(4, 4.), 1.);
    assert!((glimmer_weight(3, 4.) - 1. / 3.).abs() < 1e-6);
    assert!((glimmer_weight(5, 4.) - 1. / 3.).abs() < 1e-6);
    assert_eq!(glimmer_weight(4, 5.5), 0.);
    assert_eq!(glimmer_weight(4, 2.5), 0.);
    assert_eq!(glimmer_weight(0, 19.), 0.);
}

#[test]
fn stepped_lights_three_clusters() {
    let lit: Vec<usize> = (0..9).filter(|&g| stepped_lit(g, 4)).collect();
    assert_eq!(lit, vec![3, 4, 5]);
    assert!((0..9).all(|g| !stepped_lit(g, -100)));
}

#[test]
fn the_smooth_band_sleeps_while_it_is_off_the_text() {
    // Right to left: the band touches the last cluster once its center is under W + 0.5, which is
    // 9.5 steps in (1900ms), and leaves once it is at -1.5, 20.5 steps in.
    assert_eq!(band_wait_ms(0, W, false), 1900);
    assert_eq!(band_wait_ms(1000, W, false), 900);
    assert_eq!(band_wait_ms(1900, W, false), 0);
    assert_eq!(band_wait_ms(4000, W, false), 0);
    assert_eq!(band_wait_ms(4100, W, false), 29 * 200 - 4100 + 1900);
    // Left to right: it touches cluster 0 once its center passes -1.5, 8.5 steps in.
    assert_eq!(band_wait_ms(0, W, true), 425);
    assert_eq!(band_wait_ms(425, W, true), 0);
}

#[test]
fn plain_ascii_text_is_one_cluster_per_char() {
    assert_eq!(clusters("Thinking…").len(), 9);
}

#[test]
fn a_decomposed_accent_counts_as_one_cluster() {
    // "Réflexion…" written with a decomposed é: 'e' followed by U+0301 COMBINING ACUTE ACCENT.
    let text = "R\u{65}\u{0301}flexion\u{2026}";
    assert_eq!(clusters(text).len(), 10);
    assert_eq!(cluster_count(text), 10);
}

#[test]
fn a_zwj_emoji_sequence_counts_as_one_cluster() {
    // A family emoji: man, ZWJ, woman, ZWJ, girl. Five codepoints, one cluster.
    let text = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}";
    assert_eq!(clusters(text).len(), 1);
}

#[test]
fn a_variation_selector_attaches_to_its_base() {
    // Heavy black heart drawn as emoji, not text glyph.
    let text = "\u{2764}\u{FE0F}";
    assert_eq!(clusters(text).len(), 1);
}

#[test]
fn glimmer_highlights_colors_one_run_per_cluster_not_per_char() {
    let text = "R\u{65}\u{0301}"; // "Ré" with a decomposed é.
    let (message, glimmer) = (gpui_kit::rgb(0x000000).into(), gpui_kit::rgb(0xFFFFFF).into());
    let highlights = glimmer_highlights(text, message, glimmer, |i| if i == 1 { 1. } else { 0. });
    assert_eq!(highlights.len(), 2);
    // The second highlight covers both the 'e' and its combining mark, byte 1 through the end.
    assert_eq!(highlights[1].0, 1..text.len());
}
