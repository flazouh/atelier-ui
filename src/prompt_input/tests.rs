use std::{cell::RefCell, rc::Rc};

use gpui_kit::{Context, Entity, TestAppContext, VisualTestContext, Window};

use super::*;
use crate::theme::{Appearance, set_appearance};

fn open(cx: &mut TestAppContext) -> (Entity<PromptInput>, Rc<RefCell<Vec<PromptInputEvent>>>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
    });
    let (prompt, cx) = cx.add_window_view(|window, cx| {
        PromptInput::new("Ask", "", window, cx).modes(vec!["Ask first".into(), "Plan".into()])
    });
    let heard = Rc::new(RefCell::new(Vec::new()));
    let log = heard.clone();
    cx.update(|_, cx| cx.subscribe(&prompt, move |_, e: &PromptInputEvent, _| log.borrow_mut().push(e.clone())).detach());
    cx.update(|window, cx| prompt.read(cx).focus_handle(cx).focus(window, cx));
    (prompt, heard, cx)
}

/// Enter sends, and so does ⌘↵; Shift-Enter makes a new line.
#[gpui_kit::test]
fn enter_and_command_enter_send_and_shift_enter_does_not(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = open(cx);
    cx.simulate_input("one");
    cx.simulate_keystrokes("enter");
    cx.simulate_input("two");
    cx.simulate_keystrokes("secondary-enter");
    cx.simulate_input("three");
    cx.simulate_keystrokes("shift-enter");
    let sent: Vec<String> = heard.borrow().iter().filter_map(|e| match e {
        PromptInputEvent::Submit(t) => Some(t.to_string()),
        _ => None,
    }).collect();
    assert_eq!(sent, ["one", "two"]);
    assert_eq!(cx.update(|_, cx| prompt.read(cx).text(cx).to_string()), "three\n");
}

/// A press on the box's padding, off the text, still puts the caret in the text.
#[gpui_kit::test]
fn a_press_on_the_frame_focuses_the_text(cx: &mut TestAppContext) {
    let (prompt, _, cx) = open(cx);
    cx.update(|window, cx| window.blur(cx));
    let frame = cx.debug_bounds("prompt-frame").expect("the frame is drawn");
    cx.simulate_click(frame.origin + gpui_kit::point(gpui_kit::px(3.), gpui_kit::px(3.)), gpui_kit::Modifiers::default());
    cx.run_until_parked();
    assert!(cx.update(|window, cx| prompt.read(cx).focus_handle(cx).is_focused(window)));
}

fn commands() -> Vec<crate::command_item::CommandItem> {
    use crate::command_item::{CommandItem, CommandSource};
    let item = |name: &str, source, hint: Option<&str>| CommandItem {
        name: name.to_string().into(),
        source,
        summary: format!("what {name} does").into(),
        args_hint: hint.map(|h| h.to_string().into()),
    };
    vec![
        item("compact", CommandSource::Agent, None),
        item("review", CommandSource::Atelier, None),
        item("goal", CommandSource::Atelier, Some("<the goal>")),
    ]
}

fn commands_of(heard: &Rc<RefCell<Vec<PromptInputEvent>>>) -> Vec<(String, String)> {
    heard
        .borrow()
        .iter()
        .filter_map(|e| match e {
            PromptInputEvent::Command { name, args } => Some((name.to_string(), args.to_string())),
            _ => None,
        })
        .collect()
}

/// `/` opens the commands under the text, filtered as it is typed; Enter runs the one in front, and the box empties.
#[gpui_kit::test]
fn a_slash_offers_the_commands_and_enter_runs_one(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = open(cx);
    prompt.update(cx, |p, cx| p.set_commands(commands(), cx));
    cx.simulate_input("/co");
    cx.run_until_parked();
    assert!(cx.debug_bounds("command-row-compact").is_some(), "compact is offered for /co");
    assert!(cx.debug_bounds("command-row-review").is_none(), "review does not match co");
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert_eq!(commands_of(&heard), [("compact".to_string(), String::new())]);
    assert_eq!(cx.update(|_, cx| prompt.read(cx).text(cx).to_string()), "");
    assert!(cx.debug_bounds("command-row-compact").is_none(), "the list closes");
}

/// A command that takes words writes its name and waits for them; Enter then sends the whole line.
#[gpui_kit::test]
fn a_command_with_arguments_waits_for_them(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = open(cx);
    prompt.update(cx, |p, cx| p.set_commands(commands(), cx));
    cx.simulate_input("/goa");
    cx.run_until_parked();
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert_eq!(cx.update(|_, cx| prompt.read(cx).text(cx).to_string()), "/goal ");
    cx.simulate_input("ship it");
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert_eq!(commands_of(&heard), [("goal".to_string(), "ship it".to_string())]);
}

/// `@` opens the project's files; Enter writes the one in front as a mention.
#[gpui_kit::test]
fn an_at_sign_offers_the_files_and_enter_writes_a_mention(cx: &mut TestAppContext) {
    let (prompt, _heard, cx) = open(cx);
    prompt.update(cx, |p, cx| p.set_files(vec!["README.md".into(), "src/lib.rs".into(), "src/main.rs".into()], cx));
    cx.simulate_input("look at @li");
    cx.run_until_parked();
    assert!(cx.debug_bounds("file-row-src/lib.rs").is_some(), "src/lib.rs is offered for @li");
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert_eq!(cx.update(|_, cx| prompt.read(cx).text(cx).to_string()), "look at @src/lib.rs ");
}

/// Escape closes the list and keeps the text; Down moves to the next row.
#[gpui_kit::test]
fn escape_closes_the_list_and_down_moves_in_it(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = open(cx);
    prompt.update(cx, |p, cx| p.set_commands(commands(), cx));
    cx.simulate_input("/");
    cx.run_until_parked();
    cx.simulate_keystrokes("down enter");
    cx.run_until_parked();
    assert_eq!(commands_of(&heard), [("review".to_string(), String::new())], "down, then Enter, runs the second");
    cx.simulate_input("/re");
    cx.run_until_parked();
    cx.simulate_keystrokes("escape");
    cx.run_until_parked();
    assert!(cx.debug_bounds("command-row-review").is_none(), "Escape closes the list");
    assert_eq!(cx.update(|_, cx| prompt.read(cx).text(cx).to_string()), "/re", "and keeps the text");
}

/// The prompt at the foot of a tall window, as in a session panel.
struct Foot(Entity<PromptInput>);

impl gpui_kit::Render for Foot {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui_kit::IntoElement {
        gpui_kit::div().size_full().flex().flex_col().justify_end().child(self.0.clone()).child(gpui_kit::div().h(gpui_kit::px(46.)))
    }
}

/// The list opens above the box, even with one row and room below: it never covers what is under the box.
#[gpui_kit::test]
fn the_list_opens_above_the_box(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
    });
    let mut made = None;
    let (_root, cx) = cx.add_window_view(|window, cx| {
        let prompt = cx.new(|cx| PromptInput::new("Ask", "", window, cx));
        made = Some(prompt.clone());
        Foot(prompt)
    });
    let prompt = made.unwrap();
    cx.update(|window, cx| prompt.read(cx).focus_handle(cx).focus(window, cx));
    prompt.update(cx, |p, cx| p.set_commands(commands(), cx));
    cx.simulate_input("/goa");
    cx.run_until_parked();
    let frame = cx.debug_bounds("prompt-frame").expect("the frame is drawn");
    let list = cx.debug_bounds("prompt-picker").expect("the list is drawn");
    assert!(list.bottom() <= frame.top(), "the list ({list:?}) is above the box ({frame:?})");
}

/// Frames of 16 ms for `ms` on the frozen clock, each drawn.
fn run_for(ms: u64, cx: &mut VisualTestContext) {
    crate::motion::clock::freeze();
    for _ in 0..(ms / 16).max(1) {
        crate::motion::clock::advance(std::time::Duration::from_millis(16));
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
    }
}

/// The mode select in the composer: a press opens it, a press on its trigger shuts it, and the next press opens it
/// again the first time, with the pointer on the trigger throughout.
#[gpui_kit::test]
fn the_composers_select_opens_again_the_first_time_after_a_close(cx: &mut TestAppContext) {
    let (_prompt, _heard, cx) = open(cx);
    let at = cx.debug_bounds("prompt-mode-select").expect("the mode select is drawn").center();
    cx.simulate_mouse_move(at, None, gpui_kit::Modifiers::default());
    run_for(100, cx);
    for press in 1..=3 {
        cx.simulate_click(at, gpui_kit::Modifiers::default());
        run_for(700, cx);
        let is_open = cx.debug_bounds("select-option-0").is_some();
        assert_eq!(is_open, press % 2 == 1, "press {press}: the list is {}", if press % 2 == 1 { "open" } else { "shut" });
    }
}

/// With the mode list open, a press on the model select shuts the mode list and opens the model list.
#[gpui_kit::test]
fn a_press_on_the_model_select_switches_from_the_open_mode_list(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
    });
    let (_prompt, cx) = cx.add_window_view(|window, cx| {
        PromptInput::new("Ask", "", window, cx)
            .models(vec![PromptModel::new("a", "Model A"), PromptModel::new("b", "Model B")])
            .modes(vec!["Ask first".into(), "Plan".into()])
    });
    cx.simulate_resize(gpui_kit::size(px(900.), px(600.)));
    run_for(100, cx);
    let model = cx.debug_bounds("prompt-model-select").expect("the model select is drawn");
    let mode = cx.debug_bounds("prompt-mode-select").expect("the mode select is drawn");
    let under = |cx: &mut VisualTestContext| {
        let option = cx.debug_bounds("select-option-0")?;
        let from = |b: Bounds<Pixels>| (option.left() - b.left()).abs();
        Some(if from(model) < from(mode) { "model" } else { "mode" })
    };
    cx.simulate_click(mode.center(), gpui_kit::Modifiers::default());
    run_for(700, cx);
    assert_eq!(under(cx), Some("mode"), "the first press opens the mode list");
    cx.simulate_click(model.center(), gpui_kit::Modifiers::default());
    run_for(700, cx);
    assert_eq!(under(cx), Some("model"), "the press on the model select opens the model list in its place");
}

/// Two session panels side by side, each with its prompt at the foot.
struct Panels([Entity<PromptInput>; 2]);

impl gpui_kit::Render for Panels {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui_kit::IntoElement {
        gpui_kit::div().size_full().flex().children(
            self.0.iter().map(|p| gpui_kit::div().w(px(450.)).h_full().flex().flex_col().justify_end().child(p.clone()).child(gpui_kit::div().h(px(46.)))),
        )
    }
}

/// With the mode list open in one panel, a press on the model select of the other panel shuts it and opens that one.
#[gpui_kit::test]
fn a_press_on_the_model_select_of_another_panel_switches_from_the_open_mode_list(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
    });
    let (_panels, cx) = cx.add_window_view(|window, cx| {
        Panels([0, 1].map(|_| {
            cx.new(|cx| {
                PromptInput::new("Ask", "", window, cx)
                    .models(vec![PromptModel::new("a", "Model A"), PromptModel::new("b", "Model B")])
                    .modes(vec!["Ask first".into(), "Plan".into()])
            })
        }))
    });
    cx.simulate_resize(gpui_kit::size(px(900.), px(600.)));
    run_for(100, cx);
    // The two panels are the same, 450px apart: the selects of the first, wherever the lookup finds them.
    let in_first = |b: Bounds<Pixels>| if b.left() < px(450.) { b } else { Bounds::new(gpui_kit::point(b.left() - px(450.), b.top()), b.size) };
    let model = in_first(cx.debug_bounds("prompt-model-select").expect("the model select is drawn"));
    let mode = in_first(cx.debug_bounds("prompt-mode-select").expect("the mode select is drawn"));
    let shift = |b: Bounds<Pixels>| Bounds::new(gpui_kit::point(b.left() + px(450.), b.top()), b.size);
    let under = |cx: &mut VisualTestContext| {
        let option = cx.debug_bounds("select-option-0")?;
        let places = [("first mode", mode), ("first model", model), ("second mode", shift(mode)), ("second model", shift(model))];
        places.into_iter().min_by(|a, b| (option.left() - a.1.left()).abs().partial_cmp(&(option.left() - b.1.left()).abs()).unwrap()).map(|p| p.0)
    };
    cx.simulate_click(mode.center(), gpui_kit::Modifiers::default());
    run_for(700, cx);
    assert_eq!(under(cx), Some("first mode"));
    cx.simulate_click(shift(model).center(), gpui_kit::Modifiers::default());
    run_for(700, cx);
    assert_eq!(under(cx), Some("second model"), "the press on the other panel's model select opens its list");
    cx.simulate_click(model.center(), gpui_kit::Modifiers::default());
    run_for(700, cx);
    assert_eq!(under(cx), Some("first model"), "and the first panel's model select, with the same ids, takes over");
}

#[test]
fn a_transcript_follows_the_text_after_one_space() {
    assert_eq!(append_transcript("", "hello there"), "hello there");
    assert_eq!(append_transcript("fix this", "  and that "), "fix this and that");
    assert_eq!(append_transcript("fix this ", "and that"), "fix this and that");
    assert_eq!(append_transcript("line one\n", "line two"), "line one\nline two");
}

fn click(cx: &mut VisualTestContext, name: &'static str) {
    let at = cx.debug_bounds(name).unwrap_or_else(|| panic!("{name} is not drawn")).center();
    cx.simulate_click(at, gpui_kit::Modifiers::default());
    cx.run_until_parked();
}

fn dictation(cx: &mut TestAppContext) -> (Entity<PromptInput>, Rc<RefCell<Vec<PromptInputEvent>>>, &mut VisualTestContext) {
    let (prompt, heard, cx) = open(cx);
    cx.update(|_, cx| prompt.update(cx, |p, cx| p.set_dictation(true, cx)));
    (prompt, heard, cx)
}

fn count(heard: &Rc<RefCell<Vec<PromptInputEvent>>>, event: PromptInputEvent) -> usize {
    heard.borrow().iter().filter(|e| **e == event).count()
}

/// The microphone asks the owner to start; it says nothing while the setup runs, and turns into Stop while it listens.
#[gpui_kit::test]
fn the_microphone_starts_dictation_and_becomes_stop_while_it_listens(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = dictation(cx);
    cx.run_until_parked();
    click(cx, "prompt-mic");
    assert_eq!(count(&heard, PromptInputEvent::DictationStart), 1);
    cx.update(|_, cx| prompt.update(cx, |p, cx| p.set_voice_setup(SetupPhase::Prepare, cx)));
    cx.run_until_parked();
    click(cx, "prompt-mic");
    assert_eq!(count(&heard, PromptInputEvent::DictationStart), 1, "the setup does not start again");
    assert_eq!(count(&heard, PromptInputEvent::DictationStop), 0);
    cx.update(|_, cx| prompt.update(cx, |p, cx| p.set_voice_listening(cx)));
    cx.run_until_parked();
    click(cx, "prompt-mic");
    assert_eq!(count(&heard, PromptInputEvent::DictationStop), 1);
}

/// Without `set_dictation` there is no microphone.
#[gpui_kit::test]
fn there_is_no_microphone_unless_dictation_is_on(cx: &mut TestAppContext) {
    let (_, _, cx) = open(cx);
    cx.run_until_parked();
    assert!(cx.debug_bounds("prompt-mic").is_none());
}

/// Send waits while dictation holds the box, and Enter does not send either.
#[gpui_kit::test]
fn nothing_is_sent_while_it_listens(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = dictation(cx);
    cx.simulate_input("hello");
    cx.update(|_, cx| prompt.update(cx, |p, cx| p.set_voice_listening(cx)));
    cx.run_until_parked();
    cx.simulate_keystrokes("enter");
    assert!(!heard.borrow().iter().any(|e| matches!(e, PromptInputEvent::Submit(_))));
    cx.update(|_, cx| prompt.update(cx, |p, cx| p.set_voice_idle(cx)));
    cx.run_until_parked();
    cx.simulate_keystrokes("enter");
    assert!(heard.borrow().iter().any(|e| matches!(e, PromptInputEvent::Submit(t) if t == "hello")));
}

/// A failed press shows why, still lets the user send and press the microphone again.
#[gpui_kit::test]
fn a_failed_press_can_be_sent_past_and_pressed_again(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = dictation(cx);
    cx.update(|_, cx| prompt.update(cx, |p, cx| p.set_voice_error("Microphone unavailable: no microphone found", cx)));
    cx.run_until_parked();
    assert_eq!(cx.update(|_, cx| prompt.read(cx).voice_mode()), VoiceMode::Failed);
    click(cx, "prompt-mic");
    assert_eq!(count(&heard, PromptInputEvent::DictationStart), 1);
    cx.simulate_input("still typing");
    cx.simulate_keystrokes("enter");
    assert!(heard.borrow().iter().any(|e| matches!(e, PromptInputEvent::Submit(_))));
}

/// The transcript lands at the end of what is written.
#[gpui_kit::test]
fn a_transcript_is_written_after_the_text(cx: &mut TestAppContext) {
    let (prompt, _, cx) = dictation(cx);
    cx.simulate_input("fix");
    cx.update(|window, cx| prompt.update(cx, |p, cx| p.insert_transcript("the tool cards", window, cx)));
    assert_eq!(cx.update(|_, cx| prompt.read(cx).text(cx).to_string()), "fix the tool cards");
}
