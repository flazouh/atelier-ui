use gpui_kit::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, SharedString, Styled, TestAppContext, Window, div, px, rems,
    component::text::{TextView, TextViewStyle},
};
use crate::{
    theme::{ActiveTheme, Appearance, set_appearance},
    typography::TextSize,
};

struct Pair {
    text: String,
    width: f32,
}
impl Render for Pair {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let style = TextViewStyle::default().paragraph_gap(rems(0.75));
        let shell = |d: gpui_kit::Div| {
            d.flex_none().max_w(px(self.width))
                .text_color(theme.foreground.opacity(0.9))
                .text_size(TextSize::Sm.font_size())
                .line_height(px(24.))
        };
        div()
            .flex()
            .flex_col()
            .items_start()
            .gap(px(20.))
            .child(shell(div().debug_selector(|| "as-markdown".into())).child(TextView::markdown("md", self.text.clone()).style(style)))
            .child(shell(div().debug_selector(|| "as-runs".into())).child(SharedString::from(self.text.clone())))
    }
}

const PARAGRAPHS: [&str; 5] = [
    "The build finished and every test passed, so the change is ready to merge whenever you are.",
    "I read the file once first, because the Edit tool needs that. Then I changed the last line and nothing else.",
    "A short line.",
    "Supercalifragilisticexpialidocious words, and antidisestablishmentarianism too, break in odd places when the column is narrow.",
    "It costs 1.2ms per frame (p95 3.4ms) on the Mac, but the software renderer on the HP is ten times slower, so compare only one machine.",
];

/// The tail paragraph is drawn as plain runs while it grows and as Markdown once it ends. The two must break the same
/// lines at the same width, or the paragraph jumps when it moves over.
#[gpui_kit::test]
fn a_paragraph_breaks_the_same_lines_as_runs_and_as_markdown(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
        cx.set_reduce_motion(true);
    });
    let mut worst = Vec::new();
    for text in PARAGRAPHS {
        for width in [160., 200., 240., 280., 320., 360., 420., 480., 560., 640.] {
            let t = text.to_string();
            let (_view, cx) = cx.add_window_view(move |_, _| Pair { text: t, width });
            cx.run_until_parked();
            let (md, runs) = (cx.debug_bounds("as-markdown").unwrap(), cx.debug_bounds("as-runs").unwrap());
            if md.size != runs.size {
                worst.push(format!("{width}px: markdown {:?}, runs {:?}: {text}", md.size, runs.size));
            }
        }
    }
    assert!(worst.is_empty(), "the two lay out differently:\n{}", worst.join("\n"));
}

mod flow {
    use std::time::{Duration, Instant};
    use super::super::*;

    #[test]
    fn the_tail_starts_after_the_last_blank_line() {
        assert_eq!(split_tail("One paragraph, still growing"), 0);
        let text = "First paragraph.\n\nSecond, still gro";
        assert_eq!(&text[split_tail(text)..], "Second, still gro");
        let text = "A.\n\nB.\n\n";
        assert_eq!(&text[split_tail(text)..], "", "a finished paragraph leaves an empty tail");
        let text = "A.\n\n\n\nB";
        assert_eq!(&text[split_tail(text)..], "B", "more than one blank line is one gap");
    }

    #[test]
    fn a_blank_line_inside_a_code_fence_does_not_end_the_paragraph() {
        let text = "Here:\n\n```rust\nfn a() {}\n\nfn b() {";
        assert_eq!(split_tail(text), text.len(), "inside an open fence there is no tail to fade");
        let closed = "Here:\n\n```rust\nfn a() {}\n```\n\nDone, and more";
        assert_eq!(&closed[split_tail(closed)..], "Done, and more");
    }

    #[test]
    fn only_a_plain_paragraph_fades() {
        assert!(is_plain("The build passed and the change is ready"));
        assert!(is_plain("It costs 1.2 ms (p95 3.4 ms) on the Mac"));
        for not in ["# A heading", "- a list item", "* another", "1. numbered", "> quoted", "```code", "| a | b |", "    indented code", "Use `Edit` once", "a **bold** word", "see [the docs](x)", "a_b_c_d", "line one\nline two"] {
            assert!(!is_plain(not), "{not:?} needs Markdown");
        }
    }

    fn at(base: Instant, ms: u64) -> Instant {
        base + Duration::from_millis(ms)
    }

    /// A new piece starts clear and reaches full ink after the fade; older text has no run, so it is full at once.
    #[test]
    fn a_piece_fades_in_by_its_age_and_older_text_needs_no_run() {
        let base = Instant::now();
        let mut flow = Flow::default();
        let text1 = "Hello";
        flow.observe(text1, base);
        let text2 = "Hello there";
        flow.observe(text2, at(base, 40));
        let runs = flow.alphas(text2, 0, at(base, 60));
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].0, 0..5);
        assert_eq!(runs[1].0, 5..11);
        assert!(runs[1].1 < runs[0].1, "the newer piece is fainter: {runs:?}");
        assert!(runs[1].1 > 0. && runs[0].1 < 1.);
        let later = flow.alphas(text2, 0, at(base, 40 + 100));
        assert!(later.is_empty(), "both are settled: nothing to draw differently");
    }

    #[test]
    fn the_fade_is_over_after_100_ms_so_no_frame_is_asked_for() {
        let base = Instant::now();
        let mut flow = Flow::default();
        flow.observe("Hi", base);
        assert!(flow.is_fading(at(base, 50)));
        assert!(!flow.is_fading(at(base, 100)));
        assert!(!Flow::default().is_fading(base), "a stream that has not started asks for nothing");
    }

    /// Cursor's `fade-in-fast`: CSS `ease-in-out`, so a piece starts slow, is at half ink halfway, and lands slow.
    #[test]
    fn a_piece_eases_in_and_out() {
        let base = Instant::now();
        let mut flow = Flow::default();
        flow.observe("Hi", base);
        let ink = |ms| flow.alphas("Hi", 0, at(base, ms))[0].1;
        assert!((ink(50) - 0.5).abs() < 0.01, "halfway: {}", ink(50));
        assert!(ink(25) < 0.2, "a slow start: {}", ink(25));
        assert!(ink(75) > 0.8, "a slow landing: {}", ink(75));
    }
    #[test]
    fn pieces_before_the_tail_are_left_out_and_a_piece_across_it_is_cut() {
        let base = Instant::now();
        let mut flow = Flow::default();
        flow.observe("First.\n\nSec", base);
        let text = "First.\n\nSecond";
        flow.observe(text, at(base, 50));
        let tail = split_tail(text);
        let runs = flow.alphas(text, tail, at(base, 60));
        assert!(runs.iter().all(|(r, _)| r.start >= tail), "{runs:?}");
        assert_eq!(runs.first().map(|(r, _)| r.start), Some(tail));
        assert_eq!(runs.last().map(|(r, _)| r.end), Some(text.len()));
    }

    #[test]
    fn a_text_that_is_replaced_starts_over() {
        let base = Instant::now();
        let mut flow = Flow::default();
        flow.observe("A long first answer", base);
        flow.observe("Short", at(base, 10));
        let runs = flow.alphas("Short", 0, at(base, 20));
        assert!(runs.iter().all(|(r, _)| r.end <= 5), "{runs:?}");
    }

    #[test]
    fn many_pieces_keep_few_marks() {
        let base = Instant::now();
        let mut flow = Flow::default();
        let mut text = String::new();
        for i in 0..400 {
            text.push_str("tok ");
            flow.observe(&text, at(base, i * 16));
        }
        assert!(flow.marks() <= 16, "{} marks after 400 pieces in 6.4s", flow.marks());
    }
}

/// The composite the stream draws (Markdown for what is finished, runs for the tail under a 12px gap) breaks the same
/// lines as the whole text as Markdown, so nothing moves when the tail joins the rest.
struct Composite {
    text: String,
    width: f32,
}
impl Render for Composite {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let style = TextViewStyle::default().paragraph_gap(rems(0.75));
        let shell = |d: gpui_kit::Div| {
            d.flex_none().max_w(px(self.width)).text_color(theme.foreground.opacity(0.9)).text_size(TextSize::Sm.font_size()).line_height(px(24.))
        };
        let cut = super::split_tail(&self.text);
        let (head, tail) = self.text.split_at(cut);
        let head = head.trim_end_matches('\n');
        div()
            .flex()
            .flex_col()
            .items_start()
            .gap(px(20.))
            .child(shell(div().debug_selector(|| "whole".into())).child(TextView::markdown("whole", self.text.clone()).style(style.clone())))
            .child(
                shell(div().debug_selector(|| "composite".into()))
                    .child(TextView::markdown("head", head.to_string()).style(style))
                    .child(div().mt(px(12.)).child(SharedString::from(tail.to_string()))),
            )
    }
}

#[gpui_kit::test]
fn the_tail_under_its_gap_lays_out_as_one_more_paragraph(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
        cx.set_reduce_motion(true);
    });
    let mut off = Vec::new();
    for tail in PARAGRAPHS {
        for width in [200., 260., 320., 420., 560.] {
            let text = format!("{}\n\n{}\n\n{tail}", PARAGRAPHS[0], PARAGRAPHS[1]);
            let (_view, cx) = cx.add_window_view(move |_, _| Composite { text, width });
            cx.run_until_parked();
            let (whole, composite) = (cx.debug_bounds("whole").unwrap(), cx.debug_bounds("composite").unwrap());
            if whole.size != composite.size {
                off.push(format!("{width}px: whole {:?}, composite {:?}: {tail}", whole.size, composite.size));
            }
        }
    }
    assert!(off.is_empty(), "the tail moves the text:\n{}", off.join("\n"));
}

mod in_text {
    use gpui_kit::{Context, IntoElement, ParentElement, Render, Styled, TestAppContext, Window, div, px};
    use crate::{
        agent_text::{AgentText, AgentTextStatus},
        theme::{Appearance, set_appearance},
    };

    struct Page {
        text: String,
        status: AgentTextStatus,
        fade: bool,
    }
    impl Render for Page {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().w(px(420.)).child(AgentText::new("answer", self.text.clone()).status(self.status).fade_tail(self.fade))
        }
    }
    fn show(text: &str, status: AgentTextStatus, fade: bool, reduce: bool, cx: &mut TestAppContext) -> bool {
        cx.update(|cx| {
            gpui_kit::init(cx);
            set_appearance(Appearance::Dark, cx);
            cx.set_reduce_motion(reduce);
        });
        let text = text.to_string();
        let (_view, cx) = cx.add_window_view(move |_, _| Page { text, status, fade });
        cx.run_until_parked();
        cx.debug_bounds("stream-tail").is_some()
    }

    #[gpui_kit::test]
    fn a_streaming_plain_tail_is_drawn_as_runs(cx: &mut TestAppContext) {
        assert!(show("Done with the first part.\n\nNow the second", AgentTextStatus::Streaming, true, false, cx));
    }

    #[gpui_kit::test]
    fn the_first_paragraph_streams_as_runs_too(cx: &mut TestAppContext) {
        assert!(show("Just starting to answer", AgentTextStatus::Streaming, true, false, cx));
    }

    #[gpui_kit::test]
    fn nothing_fades_unless_asked_or_after_the_stream_or_under_reduce_motion(cx: &mut TestAppContext) {
        let text = "Done with the first part.\n\nNow the second";
        assert!(!show(text, AgentTextStatus::Streaming, false, false, cx), "not asked for");
        assert!(!show(text, AgentTextStatus::Complete, true, false, cx), "the answer is complete");
        assert!(!show(text, AgentTextStatus::Streaming, true, true, cx), "Reduce Motion");
    }

    #[gpui_kit::test]
    fn a_tail_that_needs_markdown_is_left_to_the_view(cx: &mut TestAppContext) {
        assert!(!show("Intro.\n\nUse `Edit` once", AgentTextStatus::Streaming, true, false, cx));
        assert!(!show("Intro.\n\n- first item", AgentTextStatus::Streaming, true, false, cx));
        assert!(!show("Intro.\n\n```rust\nlet a = 1;\n\nlet b", AgentTextStatus::Streaming, true, false, cx));
    }
}

mod cost {
    use std::time::{Duration, Instant};
    use gpui_kit::{IntoElement, ParentElement, Styled, TestAppContext, div, px};
    use crate::{
        agent_text::{AgentText, AgentTextStatus},
        theme::{Appearance, set_appearance},
    };

    const WORDS: [&str; 12] = ["the", "build", "passed", "and", "every", "test", "ran", "without", "a", "single", "failure", "today"];

    /// The answer after `n` tokens: five words to a token, a blank line every 14 tokens.
    fn answer(n: usize) -> String {
        let mut out = String::new();
        for i in 0..n {
            for k in 0..3 {
                out.push_str(WORDS[(i * 3 + k) % WORDS.len()]);
                out.push(' ');
            }
            if i % 14 == 13 {
                out.push_str("\n\n");
            }
        }
        out
    }

    fn percentile(sorted: &[Duration], p: f32) -> Duration {
        sorted[((sorted.len() as f32 - 1.) * p).round() as usize]
    }

    struct Bench {
        text: String,
        fade: bool,
    }
    impl gpui_kit::Render for Bench {
        fn render(&mut self, _: &mut gpui_kit::Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
            div().w(px(700.)).child(AgentText::new("answer", self.text.clone()).status(AgentTextStatus::Streaming).fade_tail(self.fade))
        }
    }

    /// A measurement, not a check: the CPU cost of each frame while an answer streams in at 60 tokens a second,
    /// drawn as it always was (the whole text as Markdown) and with the tail fading. Run it by hand:
    /// `cargo test -p beui streaming_frame_cost -- --ignored --nocapture`.
    #[gpui_kit::test]
    #[ignore = "a measurement, run by hand"]
    fn streaming_frame_cost(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            set_appearance(Appearance::Dark, cx);
            cx.set_reduce_motion(false);
        });
        let (view, cx) = cx.add_window_view(|_, _| Bench { text: String::new(), fade: false });
        let draw = |text: String, fade: bool, cx: &mut gpui_kit::VisualTestContext| {
            view.update(cx, |b, cx| {
                b.text = text;
                b.fade = fade;
                cx.notify();
            });
            let start = Instant::now();
            let view = view.clone();
            cx.draw(gpui_kit::point(px(0.), px(0.)), gpui_kit::size(px(700.), px(1800.)), move |_, _| view.clone().into_any_element());
            start.elapsed()
        };
        // Warm up: fonts, caches.
        for n in 0..40 {
            draw(answer(n), false, cx);
        }
        for round in 0..5 {
            for fade in [false, true] {
                let mut times = Vec::new();
                for n in 40..340 {
                    std::thread::sleep(Duration::from_millis(16));
                    times.push(draw(answer(n), fade, cx));
                }
                let tail_drawn = cx.debug_bounds("stream-tail").is_some();
                assert_eq!(tail_drawn, fade, "the tail is drawn in runs exactly when the fade is on");
                times.sort();
                println!(
                    "round {round} fade {fade:5}: p50 {:>6.2} ms  p95 {:>6.2} ms  max {:>6.2} ms  ({} tokens, {} chars at the end)",
                    percentile(&times, 0.5).as_secs_f64() * 1e3,
                    percentile(&times, 0.95).as_secs_f64() * 1e3,
                    times.last().unwrap().as_secs_f64() * 1e3,
                    times.len(),
                    answer(340).len(),
                );
            }
        }
    }
}

struct Arriving {
    shown: usize,
}
const LINE: &str = "Your workshop for crafting with agents.";
impl Render for Arriving {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(240.)).text_size(TextSize::Sm.font_size()).line_height(px(24.)).child(
            div().debug_selector(|| "streamed".into()).child(super::Streamed::new("line", LINE).shown(self.shown)),
        )
    }
}

/// What has not arrived keeps its place: the text takes the room of the whole of it from the first frame, so
/// nothing under or beside it moves as it fills.
#[gpui_kit::test]
fn a_streamed_text_takes_the_room_of_the_whole_text_whatever_has_arrived(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
    });
    let (host, cx) = cx.add_window_view(|_, _| Arriving { shown: 0 });
    cx.run_until_parked();
    let empty = cx.debug_bounds("streamed").expect("it is drawn with nothing in");
    assert!(empty.size.height >= px(48.), "the whole text is placed, on its two lines: {:?}", empty.size);
    for shown in [5, 14, LINE.len()] {
        host.update(cx, |h, cx| {
            h.shown = shown;
            cx.notify();
        });
        cx.run_until_parked();
        assert_eq!(cx.debug_bounds("streamed").unwrap().size, empty.size, "with {shown} bytes in");
    }
}
