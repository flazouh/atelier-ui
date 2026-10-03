use gpui_kit::ParentElement;
use gpui_kit::Styled;
use gpui_kit::Focusable;
use gpui_kit::AppContext;
use std::{cell::RefCell, rc::Rc};

use gpui_kit::{Context, Entity, TestAppContext, VisualTestContext, Window};

use super::*;
use crate::theme::{Appearance, set_appearance};
use crate::voice_input::VoiceDevice;

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
        item("tidy", CommandSource::Skill, None),
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

/// A skill picked from the list goes into the box and waits; Enter then sends it.
#[gpui_kit::test]
fn a_picked_skill_waits_in_the_box(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = open(cx);
    prompt.update(cx, |p, cx| p.set_commands(commands(), cx));
    cx.simulate_input("/tid");
    cx.run_until_parked();
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert!(commands_of(&heard).is_empty(), "the pick does not run the skill");
    assert_eq!(cx.update(|_, cx| prompt.read(cx).text(cx).to_string()), "/tidy ");
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert_eq!(commands_of(&heard), [("tidy".to_string(), String::new())]);
}

/// With skills set to run when picked, the pick runs the skill and empties the box.
#[gpui_kit::test]
fn a_picked_skill_runs_when_set_to(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = open(cx);
    prompt.update(cx, |p, cx| {
        p.set_commands(commands(), cx);
        p.set_run_picked_skills(true);
    });
    cx.simulate_input("/tid");
    cx.run_until_parked();
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert_eq!(commands_of(&heard), [("tidy".to_string(), String::new())]);
    assert_eq!(cx.update(|_, cx| prompt.read(cx).text(cx).to_string()), "");
}

fn sent(heard: &Rc<RefCell<Vec<PromptInputEvent>>>) -> Vec<String> {
    heard.borrow().iter().filter_map(|e| match e {
        PromptInputEvent::Submit(t) => Some(t.to_string()),
        _ => None,
    }).collect()
}

/// `@` opens the project's files; Enter takes the `@` words out of the text and shows the file as a chip,
/// which the message sends as a mention.
#[gpui_kit::test]
fn an_at_sign_offers_the_files_and_enter_adds_a_chip(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = open(cx);
    prompt.update(cx, |p, cx| p.set_files(vec!["README.md".into(), "src/lib.rs".into(), "src/main.rs".into()], cx));
    cx.simulate_input("look at @li");
    cx.run_until_parked();
    assert!(cx.debug_bounds("file-row-src/lib.rs").is_some(), "src/lib.rs is offered for @li");
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert_eq!(cx.update(|_, cx| prompt.read(cx).text(cx).to_string()), "look at ");
    assert!(cx.debug_bounds("file-chip-src/lib.rs").is_some(), "the file shows as a chip");
    cx.simulate_input("please");
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert_eq!(sent(&heard), ["@src/lib.rs look at please"]);
    assert!(cx.debug_bounds("file-chip-src/lib.rs").is_none(), "a sent message takes its chips");
}

/// A chip's remove button takes the file off the message; one file is one chip.
#[gpui_kit::test]
fn a_chip_comes_off_and_a_file_is_one_chip(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = open(cx);
    prompt.update(cx, |p, cx| p.set_files(vec!["README.md".into(), "src/lib.rs".into()], cx));
    for _ in 0..2 {
        cx.simulate_input("@li");
        cx.run_until_parked();
        cx.simulate_keystrokes("enter");
        cx.run_until_parked();
    }
    cx.simulate_input("@READ");
    cx.run_until_parked();
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert_eq!(cx.update(|_, cx| prompt.read(cx).attached().len()), 2);
    let remove = cx.debug_bounds("file-chip-remove-src/lib.rs").expect("a chip has a remove button");
    cx.simulate_click(remove.center(), gpui_kit::Modifiers::default());
    cx.run_until_parked();
    assert!(cx.debug_bounds("file-chip-src/lib.rs").is_none());
    cx.simulate_input("go");
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert_eq!(sent(&heard), ["@README.md go"]);
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
    // The menu unfolds over a third of a second; the tests press its rows at once.
    cx.update(|_, cx| {
        cx.set_reduce_motion(true);
        prompt.update(cx, |p, cx| p.set_dictation(true, cx))
    });
    (prompt, heard, cx)
}

fn count(heard: &Rc<RefCell<Vec<PromptInputEvent>>>, event: PromptInputEvent) -> usize {
    heard.borrow().iter().filter(|e| **e == event).count()
}

/// A press listens at once, before the owner has done anything, and the next press stops it.
#[gpui_kit::test]
fn a_press_listens_at_once_and_the_next_one_stops(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = dictation(cx);
    cx.run_until_parked();
    click(cx, "prompt-mic");
    assert_eq!(count(&heard, PromptInputEvent::DictationStart), 1);
    assert_eq!(cx.update(|_, cx| prompt.read(cx).voice_mode()), VoiceMode::Listening);
    click(cx, "prompt-mic");
    assert_eq!(count(&heard, PromptInputEvent::DictationStop), 1);
    assert_eq!(cx.update(|_, cx| prompt.read(cx).voice_mode()), VoiceMode::Idle);
}

/// While earlier words wait for the model, what was typed can be sent, and the microphone records again.
#[gpui_kit::test]
fn words_waiting_for_the_model_hold_nothing_up(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = dictation(cx);
    cx.update(|_, cx| prompt.update(cx, |p, cx| p.set_voice_setup(SetupPhase::Download(0.3), cx)));
    cx.run_until_parked();
    cx.simulate_input("typed meanwhile");
    cx.simulate_keystrokes("enter");
    assert!(heard.borrow().iter().any(|e| matches!(e, PromptInputEvent::Submit(t) if t == "typed meanwhile")));
    click(cx, "prompt-mic");
    assert_eq!(count(&heard, PromptInputEvent::DictationStart), 1);
}

/// A press taken back (a key that was part of a shortcut) ends with no stop: the owner throws it away.
#[gpui_kit::test]
fn a_cancelled_press_ends_without_a_stop(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = dictation(cx);
    cx.update(|_, cx| prompt.update(cx, |p, cx| p.press_mic(cx)));
    cx.update(|_, cx| prompt.update(cx, |p, cx| p.cancel_mic(cx)));
    assert_eq!(count(&heard, PromptInputEvent::DictationCancel), 1);
    assert_eq!(count(&heard, PromptInputEvent::DictationStop), 0);
    assert_eq!(cx.update(|_, cx| prompt.read(cx).voice_mode()), VoiceMode::Idle);
}

/// While words wait for the model, the ✕ beside the setup throws them away and gives the box back.
#[gpui_kit::test]
fn the_cross_beside_the_setup_discards_the_waiting_words(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = dictation(cx);
    cx.update(|_, cx| prompt.update(cx, |p, cx| p.set_voice_setup(SetupPhase::Download(0.4), cx)));
    for _ in 0..3 {
        cx.run_until_parked();
        cx.update(|_, cx| prompt.update(cx, |_, cx| cx.notify()));
    }
    click(cx, "prompt-voice-discard");
    assert_eq!(count(&heard, PromptInputEvent::DictationDiscard), 1);
    assert_eq!(cx.update(|_, cx| prompt.read(cx).voice_mode()), VoiceMode::Idle);
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
    cx.simulate_input("still typing");
    cx.simulate_keystrokes("enter");
    assert!(heard.borrow().iter().any(|e| matches!(e, PromptInputEvent::Submit(_))));
    click(cx, "prompt-mic");
    assert_eq!(count(&heard, PromptInputEvent::DictationStart), 1);
}

/// The transcript lands at the end of what is written.
#[gpui_kit::test]
fn a_transcript_is_written_after_the_text(cx: &mut TestAppContext) {
    let (prompt, _, cx) = dictation(cx);
    cx.simulate_input("fix");
    cx.update(|window, cx| prompt.update(cx, |p, cx| p.insert_transcript("the tool cards", window, cx)));
    assert_eq!(cx.update(|_, cx| prompt.read(cx).text(cx).to_string()), "fix the tool cards");
}

fn written(prompt: &Entity<PromptInput>, cx: &mut VisualTestContext) -> String {
    cx.update(|_, cx| prompt.read(cx).text(cx).to_string())
}

/// Words heard so far show after the text and move as more come; the final words take their place.
#[gpui_kit::test]
fn live_words_move_and_the_final_ones_take_their_place(cx: &mut TestAppContext) {
    let (prompt, _, cx) = dictation(cx);
    cx.simulate_input("fix");
    cx.update(|window, cx| prompt.update(cx, |p, cx| p.set_live_transcript("the tool", window, cx)));
    assert_eq!(written(&prompt, cx), "fix the tool");
    cx.update(|window, cx| prompt.update(cx, |p, cx| p.set_live_transcript("the tool cards.", window, cx)));
    assert_eq!(written(&prompt, cx), "fix the tool cards.");
    cx.update(|window, cx| prompt.update(cx, |p, cx| p.insert_transcript("the tool cards", window, cx)));
    assert_eq!(written(&prompt, cx), "fix the tool cards");
    cx.update(|window, cx| prompt.update(cx, |p, cx| p.insert_transcript("and the badges", window, cx)));
    assert_eq!(written(&prompt, cx), "fix the tool cards and the badges", "a later transcript adds, it does not replace");
}

/// A press that ends without words takes its live words out again.
#[gpui_kit::test]
fn live_words_go_when_the_press_ends_without_words(cx: &mut TestAppContext) {
    let (prompt, _, cx) = dictation(cx);
    cx.simulate_input("fix");
    cx.update(|window, cx| prompt.update(cx, |p, cx| p.set_live_transcript("uh", window, cx)));
    cx.update(|window, cx| prompt.update(cx, |p, cx| p.end_live_transcript(window, cx)));
    assert_eq!(written(&prompt, cx), "fix");
}

/// Once the person types, the live words stop moving, and nothing they wrote is lost.
#[gpui_kit::test]
fn typing_meanwhile_leaves_the_live_words_where_they_are(cx: &mut TestAppContext) {
    let (prompt, _, cx) = dictation(cx);
    cx.update(|window, cx| prompt.update(cx, |p, cx| p.set_live_transcript("the tool", window, cx)));
    cx.simulate_input("!");
    cx.update(|window, cx| prompt.update(cx, |p, cx| p.set_live_transcript("the tool cards", window, cx)));
    assert_eq!(written(&prompt, cx), "the tool!");
    cx.update(|window, cx| prompt.update(cx, |p, cx| p.insert_transcript("the tool cards", window, cx)));
    assert_eq!(written(&prompt, cx), "the tool! the tool cards", "the final words are added, never dropped");
}

fn microphones(prompt: &Entity<PromptInput>, cx: &mut VisualTestContext) {
    cx.update(|_, cx| {
        prompt.update(cx, |p, cx| {
            p.set_voice_devices(vec![VoiceDevice::new("usb", "USB Microphone"), VoiceDevice::new("built-in", "MacBook Pro Microphone")], None, cx)
        })
    });
}

/// The arrow beside the microphone asks the owner for a fresh list, then lists the microphones and the hold switch.
#[gpui_kit::test]
fn the_arrow_opens_the_microphones_and_the_hold_switch(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = dictation(cx);
    microphones(&prompt, cx);
    cx.run_until_parked();
    assert!(cx.debug_bounds("prompt-mic-device-usb").is_none(), "shut until the arrow is pressed");
    click(cx, "prompt-mic-menu");
    assert_eq!(count(&heard, PromptInputEvent::DictationDevices), 1);
    cx.run_until_parked();
    for row in ["prompt-mic-device-usb", "prompt-mic-device-built-in", "prompt-mic-hold"] {
        assert!(cx.debug_bounds(row).is_some(), "{row} is listed");
    }
}

/// Choosing a microphone tells the owner which, and choosing the hold row flips the switch.
#[gpui_kit::test]
fn a_microphone_and_the_hold_switch_are_chosen_from_the_menu(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = dictation(cx);
    microphones(&prompt, cx);
    click(cx, "prompt-mic-menu");
    cx.run_until_parked();
    click(cx, "prompt-mic-device-usb");
    assert_eq!(count(&heard, PromptInputEvent::DictationDevice(Some("usb".into()))), 1);
    click(cx, "prompt-mic-menu");
    cx.run_until_parked();
    click(cx, "prompt-mic-hold");
    assert_eq!(count(&heard, PromptInputEvent::DictationHold(true)), 1);
}

/// The microphone and its arrow are one group: as tall as each other, with only the seam between them.
#[gpui_kit::test]
fn the_microphone_and_its_arrow_are_one_group(cx: &mut TestAppContext) {
    let (_, _, cx) = dictation(cx);
    let (mic, arrow) = (cx.debug_bounds("prompt-mic").expect("mic"), cx.debug_bounds("prompt-mic-menu").expect("arrow"));
    assert_eq!(mic.size.height, arrow.size.height);
    assert_eq!(mic.top(), arrow.top());
    assert_eq!(arrow.left() - mic.right(), gpui_kit::px(crate::button_group::SEAM));
}

/// The list is as dense as the model select's: 28px rows. The hold row stays open when pressed, so its switch is seen to move.
#[gpui_kit::test]
fn the_menu_has_the_select_rows_and_the_hold_row_stays_open(cx: &mut TestAppContext) {
    let (prompt, _, cx) = dictation(cx);
    microphones(&prompt, cx);
    click(cx, "prompt-mic-menu");
    cx.run_until_parked();
    let row = cx.debug_bounds("prompt-mic-device-usb").expect("a device row");
    assert_eq!(row.size.height, gpui_kit::px(28.));
    click(cx, "prompt-mic-hold");
    assert!(cx.debug_bounds("prompt-mic-hold").is_some(), "the menu stays open");
}

/// The arrow waits while the microphone listens.
#[gpui_kit::test]
fn the_arrow_waits_while_it_listens(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = dictation(cx);
    cx.update(|_, cx| prompt.update(cx, |p, cx| p.set_voice_listening(cx)));
    cx.run_until_parked();
    click(cx, "prompt-mic-menu");
    assert_eq!(count(&heard, PromptInputEvent::DictationDevices), 0);
}

fn press(cx: &mut VisualTestContext, down: bool) {
    let at = cx.debug_bounds("prompt-mic").expect("the microphone is drawn").center();
    if down {
        cx.simulate_mouse_down(at, gpui_kit::MouseButton::Left, gpui_kit::Modifiers::default());
    } else {
        cx.simulate_mouse_up(at, gpui_kit::MouseButton::Left, gpui_kit::Modifiers::default());
    }
    cx.run_until_parked();
}

/// In hold mode the microphone records while it is down, and a click does nothing on its own.
#[gpui_kit::test]
fn held_down_it_starts_and_let_go_it_stops(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = dictation(cx);
    cx.update(|_, cx| prompt.update(cx, |p, cx| p.set_voice_hold(true, cx)));
    cx.run_until_parked();
    press(cx, true);
    assert_eq!(count(&heard, PromptInputEvent::DictationStart), 1);
    cx.update(|_, cx| prompt.update(cx, |p, cx| p.set_voice_listening(cx)));
    cx.run_until_parked();
    press(cx, false);
    assert_eq!(count(&heard, PromptInputEvent::DictationStop), 1);
}

/// In hold mode a quick tap is a whole press: down starts, up stops, with nothing to wait for.
#[gpui_kit::test]
fn in_hold_mode_a_tap_starts_and_stops(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = dictation(cx);
    cx.update(|_, cx| prompt.update(cx, |p, cx| p.set_voice_hold(true, cx)));
    cx.run_until_parked();
    press(cx, true);
    press(cx, false);
    assert_eq!(count(&heard, PromptInputEvent::DictationStart), 1);
    assert_eq!(count(&heard, PromptInputEvent::DictationStop), 1);
}

/// Without hold mode, a press is a click: it starts once and the release adds no stop.
#[gpui_kit::test]
fn without_hold_a_release_stops_nothing(cx: &mut TestAppContext) {
    let (_, heard, cx) = dictation(cx);
    press(cx, true);
    press(cx, false);
    assert_eq!(count(&heard, PromptInputEvent::DictationStart), 1);
    assert_eq!(count(&heard, PromptInputEvent::DictationStop), 0);
}

/// The context meter shows once the owner tells how full the context is, beside Send.
#[gpui_kit::test]
fn the_context_meter_shows_once_told(cx: &mut TestAppContext) {
    let (prompt, _, cx) = open(cx);
    cx.run_until_parked();
    assert!(cx.debug_bounds("context-meter").is_none(), "nothing is known yet");
    cx.update(|_, cx| prompt.update(cx, |p, cx| p.set_context(84_000, 200_000, cx)));
    cx.run_until_parked();
    let meter = cx.debug_bounds("context-meter").expect("the meter is drawn");
    let frame = cx.debug_bounds("prompt-frame").expect("the frame is drawn");
    assert!(meter.origin.x > frame.center().x, "it sits on the right, by Send");
}

fn heard_since(heard: &Rc<RefCell<Vec<PromptInputEvent>>>, from: usize) -> Vec<PromptInputEvent> {
    heard.borrow()[from..].iter().filter(|e| !matches!(e, PromptInputEvent::DictationDevices)).cloned().collect()
}

/// While a turn runs, Enter sends into it and ⌘↵ holds the message for after it; idle, ⌘↵ sends as ever.
#[gpui_kit::test]
fn while_running_enter_steers_and_command_enter_queues(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = open(cx);
    cx.update(|_, cx| prompt.update(cx, |p, cx| p.set_running(true, cx)));
    cx.simulate_input("steer");
    cx.simulate_keystrokes("enter");
    cx.simulate_input("later");
    cx.simulate_keystrokes("secondary-enter");
    assert_eq!(heard_since(&heard, 0), [PromptInputEvent::Submit("steer".into()), PromptInputEvent::Queue("later".into())]);
    assert_eq!(cx.update(|_, cx| prompt.read(cx).text(cx).to_string()), "", "the box empties both times");
}

/// While a turn runs, the button stops it when the box is empty and sends into it when the box has text.
#[gpui_kit::test]
fn while_running_the_button_stops_when_empty_and_steers_with_text(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = open(cx);
    cx.update(|_, cx| prompt.update(cx, |p, cx| p.set_running(true, cx)));
    cx.run_until_parked();
    click(cx, "prompt-send");
    cx.simulate_input("look at the tests too");
    cx.run_until_parked();
    click(cx, "prompt-send");
    assert_eq!(heard_since(&heard, 0), [PromptInputEvent::Stop, PromptInputEvent::Submit("look at the tests too".into())]);
}

/// The queue the owner keeps shows a row each, whose buttons ask to send one now or take it out.
#[gpui_kit::test]
fn queued_rows_send_now_or_come_out(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = open(cx);
    cx.update(|_, cx| prompt.update(cx, |p, cx| p.set_queued(vec!["first".into(), "second\nline".into()], cx)));
    cx.run_until_parked();
    assert!(cx.debug_bounds("queued-row-0").is_some() && cx.debug_bounds("queued-row-1").is_some());
    click(cx, "queued-remove-1");
    click(cx, "queued-send-0");
    assert_eq!(heard_since(&heard, 0), [PromptInputEvent::Unqueue(1), PromptInputEvent::SendQueued(0)]);
    cx.update(|_, cx| prompt.update(cx, |p, cx| p.set_queued(Vec::new(), cx)));
    cx.run_until_parked();
    assert!(cx.debug_bounds("queued-row-0").is_none(), "an empty queue shows nothing");
}
