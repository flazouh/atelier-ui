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
        PromptInputEvent::Submit(m) => Some(m.text.to_string()),
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

/// A command that takes words waits for them as a chip in front of the box; Enter then runs it with what was written.
#[gpui_kit::test]
fn a_command_with_arguments_waits_for_them(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = open(cx);
    prompt.update(cx, |p, cx| p.set_commands(commands(), cx));
    cx.simulate_input("/goa");
    cx.run_until_parked();
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert_eq!(cx.update(|_, cx| prompt.read(cx).text(cx).to_string()), "", "the name is a chip, not text");
    assert!(cx.debug_bounds("chip-command").is_some(), "the command waits as a chip");
    cx.simulate_input("ship it");
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert_eq!(commands_of(&heard), [("goal".to_string(), "ship it".to_string())]);
}

/// A skill picked from the list waits as a chip; Enter then runs it.
#[gpui_kit::test]
fn a_picked_skill_waits_in_the_box(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = open(cx);
    prompt.update(cx, |p, cx| p.set_commands(commands(), cx));
    cx.simulate_input("/tid");
    cx.run_until_parked();
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert!(commands_of(&heard).is_empty(), "the pick does not run the skill");
    assert_eq!(cx.update(|_, cx| prompt.read(cx).text(cx).to_string()), "");
    assert!(cx.debug_bounds("chip-command").is_some());
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
        PromptInputEvent::Submit(m) => Some(m.text.to_string()),
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
    assert!(cx.debug_bounds("chip-src/lib.rs").is_some(), "the file shows as a chip");
    cx.simulate_input("please");
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert_eq!(sent(&heard), ["@src/lib.rs look at please"]);
    assert!(cx.debug_bounds("chip-src/lib.rs").is_none(), "a sent message takes its chips");
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
    assert_eq!(cx.update(|_, cx| prompt.read(cx).chips().len()), 2);
    let remove = cx.debug_bounds("chip-remove-src/lib.rs").expect("a chip has a remove button");
    cx.simulate_click(remove.center(), gpui_kit::Modifiers::default());
    cx.run_until_parked();
    assert!(cx.debug_bounds("chip-src/lib.rs").is_none());
    cx.simulate_input("go");
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert_eq!(sent(&heard), ["@README.md go"]);
}

/// A chip replaced stays where it was; one that was not there goes last.
#[gpui_kit::test]
fn a_replaced_chip_keeps_its_place(cx: &mut TestAppContext) {
    let (prompt, _heard, cx) = open(cx);
    prompt.update(cx, |p, cx| {
        p.add_chip(Chip::new("a", "A"), cx);
        p.add_chip(Chip::new("b", "B"), cx);
        p.replace_chip(Chip::new("a", "A again"), cx);
        p.replace_chip(Chip::new("c", "C"), cx);
    });
    let labels: Vec<String> = prompt.read_with(cx, |p, _| p.chips().iter().map(|c| c.label.to_string()).collect());
    assert_eq!(labels, ["A again", "B", "C"]);
}

/// A press on a chip reports it by its id; the ✕ takes the chip off and reports nothing.
#[gpui_kit::test]
fn a_chip_press_is_reported_and_its_cross_is_not(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = open(cx);
    prompt.update(cx, |p, cx| {
        p.add_chip(Chip::new("quote-1", "The build fails"), cx);
        p.add_chip(Chip::new("quote-2", "Second"), cx);
    });
    cx.run_until_parked();
    let chip = cx.debug_bounds("chip-quote-1").expect("the chip is drawn");
    cx.simulate_click(chip.origin + gpui_kit::point(gpui_kit::px(10.), chip.size.height / 2.), gpui_kit::Modifiers::default());
    cx.run_until_parked();
    let pressed = |heard: &Rc<RefCell<Vec<PromptInputEvent>>>| {
        heard.borrow().iter().filter_map(|e| match e { PromptInputEvent::ChipPressed(id) => Some(id.to_string()), _ => None }).collect::<Vec<_>>()
    };
    assert_eq!(pressed(&heard), ["quote-1"]);
    let cross = cx.debug_bounds("chip-remove-quote-2").expect("a chip has a cross");
    cx.simulate_click(cross.center(), gpui_kit::Modifiers::default());
    cx.run_until_parked();
    assert_eq!(pressed(&heard), ["quote-1"], "the cross is not a press on the chip");
    assert!(cx.debug_bounds("chip-quote-2").is_none());
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
    assert!(heard.borrow().iter().any(|e| matches!(e, PromptInputEvent::Submit(m) if m.text == "typed meanwhile")));
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
    assert!(heard.borrow().iter().any(|e| matches!(e, PromptInputEvent::Submit(m) if m.text == "hello")));
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

/// A press on the ring opens the panel with the parts the owner told; the cross closes it, and the ring opens it again.
#[gpui_kit::test]
fn a_press_on_the_ring_opens_the_usage_panel_and_the_cross_closes_it(cx: &mut TestAppContext) {
    use crate::context_usage::ContextPart;
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
    cx.update(|_, cx| {
        prompt.update(cx, |p, cx| {
            p.set_context(106_300, 300_000, cx);
            p.set_context_parts(vec![ContextPart::new("System prompt", 4_100), ContextPart::new("Conversation", 69_700)], cx);
        })
    });
    cx.run_until_parked();
    assert!(cx.debug_bounds("context-usage").is_none(), "closed until pressed");
    click(cx, "context-meter");
    assert!(cx.debug_bounds("context-usage").is_some(), "the panel opens");
    assert!(cx.debug_bounds("context-usage-part-1").is_some(), "with a row for each part");
    assert!(cx.debug_bounds("context-usage-part-2").is_none(), "and no more");
    let (panel, ring) = (cx.debug_bounds("context-usage").unwrap(), cx.debug_bounds("context-meter").unwrap());
    assert!(panel.bottom() <= ring.top(), "it opens above the ring");
    click(cx, "context-usage-close");
    assert!(cx.debug_bounds("context-usage").is_none(), "the cross closes it");
    click(cx, "context-meter");
    assert!(cx.debug_bounds("context-usage").is_some(), "the ring opens it again");
}

/// Told no parts, the panel still opens, with a row for what is in use and one for what is free.
#[gpui_kit::test]
fn the_usage_panel_without_parts_shows_what_is_in_use_and_what_is_free(cx: &mut TestAppContext) {
    let (prompt, _, cx) = open(cx);
    cx.update(|_, cx| prompt.update(cx, |p, cx| p.set_context(84_000, 200_000, cx)));
    cx.run_until_parked();
    click(cx, "context-meter");
    assert!(cx.debug_bounds("context-usage-part-0").is_some(), "what is in use");
    assert!(cx.debug_bounds("context-usage-part-1").is_some(), "and what is free");
    assert!(cx.debug_bounds("context-usage-part-2").is_none());
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
    assert_eq!(heard_since(&heard, 0), [PromptInputEvent::Submit(Message::text("steer")), PromptInputEvent::Queue(Message::text("later"))]);
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
    assert_eq!(heard_since(&heard, 0), [PromptInputEvent::Stop, PromptInputEvent::Submit(Message::text("look at the tests too"))]);
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

/// While a turn runs and the box has text, a hover on Send tells how to steer and how to queue.
#[gpui_kit::test]
fn the_steer_button_tells_its_keys_on_hover(cx: &mut TestAppContext) {
    let (prompt, _, cx) = open(cx);
    cx.update(|_, cx| prompt.update(cx, |p, cx| p.set_running(true, cx)));
    cx.simulate_input("and the docs");
    cx.run_until_parked();
    let at = cx.debug_bounds("prompt-send").expect("Send is drawn").center();
    cx.simulate_mouse_move(at, None, gpui_kit::Modifiers::default());
    cx.executor().advance_clock(std::time::Duration::from_secs(2));
    cx.run_until_parked();
    assert!(cx.debug_bounds("tooltip").is_some(), "the hint shows");
}

/// The box in a column as tall as its content, as a panel holds it: the window would stretch it otherwise.
struct Column(Entity<PromptInput>);

impl gpui_kit::Render for Column {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui_kit::IntoElement {
        gpui_kit::div().w(gpui_kit::px(600.)).child(self.0.clone())
    }
}

fn boxed(cx: &mut TestAppContext) -> &mut VisualTestContext {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
    });
    let (_, cx) = cx.add_window_view(|window, cx| {
        let prompt = cx.new(|cx| PromptInput::new("Ask", "", window, cx).modes(vec!["Ask first".into(), "Plan".into()]));
        prompt.update(cx, |p, cx| p.focus_handle(cx).focus(window, cx));
        Column(prompt)
    });
    cx.run_until_parked();
    cx
}

fn rows(cx: &mut VisualTestContext) -> (Bounds<Pixels>, Bounds<Pixels>, Bounds<Pixels>) {
    (cx.debug_bounds("prompt-frame").unwrap(), cx.debug_bounds("prompt-text").unwrap(), cx.debug_bounds("prompt-toolbar").unwrap())
}

/// The box is two rows of one height: the text on top, the controls below, with the box's own padding round both.
#[gpui_kit::test]
fn an_empty_box_is_two_rows_of_one_height(cx: &mut TestAppContext) {
    let cx = boxed(cx);
    let (frame, text, bar) = rows(cx);
    assert_eq!(text.size.height, bar.size.height, "text {text:?} bar {bar:?}");
    assert_eq!(text.top() - frame.top(), frame.bottom() - bar.bottom(), "the same padding above and below");
    assert_eq!(bar.top(), text.bottom(), "no gap between the rows");
}

fn write_lines(cx: &mut VisualTestContext, n: usize) {
    for i in 0..n {
        if i > 0 {
            cx.simulate_keystrokes("shift-enter");
        }
        cx.simulate_input("line");
    }
    cx.run_until_parked();
}

/// Writing more lines makes the text row taller, a line at a time, and the controls stay under it.
#[gpui_kit::test]
fn more_lines_make_the_box_taller_with_the_controls_below(cx: &mut TestAppContext) {
    let cx = boxed(cx);
    let one = rows(cx).0.size.height;
    write_lines(cx, 4);
    let (frame, text, bar) = rows(cx);
    assert_eq!(frame.size.height - one, gpui_kit::px(3. * 24.), "three more lines of 24px");
    assert_eq!(bar.top(), text.bottom());
}

/// Past eight lines the box stops growing and the text scrolls inside it.
#[gpui_kit::test]
fn the_box_stops_growing_at_eight_lines(cx: &mut TestAppContext) {
    let cx = boxed(cx);
    write_lines(cx, 8);
    let eight = rows(cx).0.size.height;
    for _ in 0..12 {
        cx.simulate_keystrokes("shift-enter");
        cx.simulate_input("more");
    }
    cx.run_until_parked();
    assert_eq!(rows(cx).0.size.height, eight);
}

fn submitted(heard: &Rc<RefCell<Vec<PromptInputEvent>>>) -> Vec<Message> {
    heard.borrow().iter().filter_map(|e| if let PromptInputEvent::Submit(m) = e { Some(m.clone()) } else { None }).collect()
}

/// The owner puts a chip of its own over the text; one with a mention writes it in front, one without adds no words,
/// and the message carries both for the owner to read back by id.
#[gpui_kit::test]
fn a_chip_the_owner_adds_rides_with_the_message(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = open(cx);
    prompt.update(cx, |p, cx| {
        p.add_chip(Chip::new("note-1", "Pasted text").look(ChipLook::Icon(crate::icon::IconName::Copy)).detail("12 lines"), cx);
        p.add_chip(Chip::new("see", "Spec").mention("@spec.md"), cx);
    });
    cx.run_until_parked();
    assert!(cx.debug_bounds("chip-note-1").is_some() && cx.debug_bounds("chip-see").is_some());
    cx.simulate_input("go");
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    let sent = submitted(&heard);
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].text, "@spec.md go");
    assert_eq!(sent[0].chips.iter().map(|c| c.id.to_string()).collect::<Vec<_>>(), ["note-1", "see"]);
    assert!(prompt.read_with(cx, |p, _| p.chips().is_empty()), "a sent message takes its chips");
}

/// A chip with an id already there is not added twice, and the owner can take one off by id.
#[gpui_kit::test]
fn a_chip_id_is_added_once_and_removed_by_id(cx: &mut TestAppContext) {
    let (prompt, _, cx) = open(cx);
    prompt.update(cx, |p, cx| {
        p.add_chip(Chip::new("a", "A"), cx);
        p.add_chip(Chip::new("a", "A again"), cx);
        p.add_chip(Chip::new("b", "B"), cx);
    });
    assert_eq!(prompt.read_with(cx, |p, _| p.chips().len()), 2);
    prompt.update(cx, |p, cx| p.remove_chip("a", cx));
    assert_eq!(prompt.read_with(cx, |p, _| p.chips().iter().map(|c| c.id.to_string()).collect::<Vec<_>>()), ["b"]);
}

/// A chip alone is enough to send: its words are the message, and without a mention there are none.
#[gpui_kit::test]
fn a_chip_alone_can_be_sent(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = open(cx);
    prompt.update(cx, |p, cx| p.add_chip(Chip::new("img", "Image"), cx));
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    let sent = submitted(&heard);
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].text, "");
    assert_eq!(sent[0].chips.len(), 1);
}

fn pastes(heard: &Rc<RefCell<Vec<PromptInputEvent>>>) -> Vec<Pasted> {
    heard.borrow().iter().filter_map(|e| if let PromptInputEvent::Paste(p) = e { Some(p.clone()) } else { None }).collect()
}

fn paste(cx: &mut VisualTestContext, item: gpui_kit::ClipboardItem) {
    cx.update(|_, cx| cx.write_to_clipboard(item));
    cx.simulate_keystrokes("secondary-v");
    cx.run_until_parked();
}

/// By default a paste is text in the box and nothing more; the owner hears nothing.
#[gpui_kit::test]
fn a_paste_is_text_in_the_box_unless_the_owner_takes_pastes(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = open(cx);
    paste(cx, gpui_kit::ClipboardItem::new_string("some words".into()));
    assert_eq!(cx.update(|_, cx| prompt.read(cx).text(cx).to_string()), "some words");
    assert!(pastes(&heard).is_empty());
}

/// Even when the owner takes pastes, a short line is words for the box: a link, a path, a name.
#[gpui_kit::test]
fn a_short_line_pastes_into_the_box_even_when_the_owner_takes_pastes(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = open(cx);
    prompt.update(cx, |p, _| p.set_paste_chips(true));
    paste(cx, gpui_kit::ClipboardItem::new_string("https://example.com/a?b=c".into()));
    assert_eq!(cx.update(|_, cx| prompt.read(cx).text(cx).to_string()), "https://example.com/a?b=c");
    assert!(pastes(&heard).is_empty());
    assert!(cx.update(|_, cx| prompt.read(cx).chips().is_empty()));
}

/// A line copied with its line break is still one line; a line past 200 characters, or two lines, is a chip.
#[test]
fn only_one_short_line_is_an_inline_paste() {
    use super::helpers::is_inline_paste;
    assert!(is_inline_paste("word"));
    assert!(is_inline_paste("a line copied from a terminal\n"));
    assert!(is_inline_paste("windows line\r\n"));
    assert!(is_inline_paste(&"x".repeat(200)));
    assert!(!is_inline_paste(&"x".repeat(201)));
    assert!(!is_inline_paste("two\nlines"));
    assert!(!is_inline_paste("two\nlines\n"));
    assert!(!is_inline_paste("a\r\nb"));
}

/// When the owner takes pastes, text goes to it and the box stays as it was.
#[gpui_kit::test]
fn a_pasted_text_goes_to_the_owner_when_it_takes_pastes(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = open(cx);
    prompt.update(cx, |p, _| p.set_paste_chips(true));
    paste(cx, gpui_kit::ClipboardItem::new_string("a long\nlog".into()));
    assert_eq!(cx.update(|_, cx| prompt.read(cx).text(cx).to_string()), "");
    assert_eq!(pastes(&heard), [Pasted::Text("a long\nlog".into())]);
}

/// An image on the clipboard is told apart from the text beside it, and wins: a browser puts both.
#[gpui_kit::test]
fn a_pasted_image_comes_before_the_text_beside_it(cx: &mut TestAppContext) {
    use gpui_kit::{ClipboardEntry, ClipboardString, Image, ImageFormat};
    let (prompt, heard, cx) = open(cx);
    prompt.update(cx, |p, _| p.set_paste_chips(true));
    let image = Image::from_bytes(ImageFormat::Png, vec![1, 2, 3]);
    paste(cx, gpui_kit::ClipboardItem { entries: vec![ClipboardEntry::String(ClipboardString::new("<img>".into())), ClipboardEntry::Image(image.clone())] });
    assert_eq!(pastes(&heard), [Pasted::Image(std::sync::Arc::new(image))]);
}

/// Copied files arrive as files.
#[gpui_kit::test]
fn pasted_files_arrive_as_paths(cx: &mut TestAppContext) {
    use gpui_kit::{ClipboardEntry, ExternalPaths};
    let (prompt, heard, cx) = open(cx);
    prompt.update(cx, |p, _| p.set_paste_chips(true));
    paste(cx, gpui_kit::ClipboardItem { entries: vec![ClipboardEntry::ExternalPaths(ExternalPaths(["/tmp/a.png".into(), "/tmp/b.rs".into()].into_iter().collect()))] });
    assert_eq!(pastes(&heard), [Pasted::Files(vec!["/tmp/a.png".into(), "/tmp/b.rs".into()])]);
}

/// An image chip shows its thumbnail where a file chip shows its icon.
#[gpui_kit::test]
fn an_image_chip_is_drawn(cx: &mut TestAppContext) {
    use gpui_kit::{Image, ImageFormat};
    let (prompt, _, cx) = open(cx);
    let image = std::sync::Arc::new(Image::from_bytes(ImageFormat::Png, vec![1, 2, 3]));
    prompt.update(cx, |p, cx| p.add_chip(Chip::new("img-1", "Image").look(ChipLook::Image(image)), cx));
    cx.run_until_parked();
    assert!(cx.debug_bounds("chip-img-1").is_some());
}

/// The ✕ on the command chip lets go of the command: the next message is a plain message.
#[gpui_kit::test]
fn the_command_chips_cross_lets_go_of_the_command(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = open(cx);
    prompt.update(cx, |p, cx| p.set_commands(commands(), cx));
    cx.simulate_input("/tid");
    cx.run_until_parked();
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    let cross = cx.debug_bounds("chip-remove-command").expect("the chip has a cross");
    cx.simulate_click(cross.center(), gpui_kit::Modifiers::default());
    cx.run_until_parked();
    assert!(cx.debug_bounds("chip-command").is_none());
    cx.simulate_input("hello");
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert!(commands_of(&heard).is_empty());
    assert_eq!(sent(&heard), ["hello"]);
}

/// Backspace in an empty box takes the last chip off, then the command; with words in the box it is the text's own.
#[gpui_kit::test]
fn backspace_in_an_empty_box_takes_chips_off_from_the_end(cx: &mut TestAppContext) {
    let (prompt, _, cx) = open(cx);
    prompt.update(cx, |p, cx| {
        p.set_commands(commands(), cx);
        p.add_chip(Chip::new("a", "A"), cx);
        p.add_chip(Chip::new("b", "B"), cx);
    });
    cx.simulate_input("/tid");
    cx.run_until_parked();
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    cx.simulate_input("xy");
    cx.simulate_keystrokes("backspace");
    assert_eq!(prompt.read_with(cx, |p, cx| p.text(cx).to_string()), "x", "with words, backspace is the text's");
    assert_eq!(prompt.read_with(cx, |p, _| p.chips().len()), 2);
    cx.simulate_keystrokes("backspace");
    cx.simulate_keystrokes("backspace");
    assert_eq!(prompt.read_with(cx, |p, _| p.chips().iter().map(|c| c.id.to_string()).collect::<Vec<_>>()), ["a"], "the last chip goes first");
    cx.simulate_keystrokes("backspace");
    assert!(cx.debug_bounds("chip-command").is_some(), "the command stays until the chips are gone");
    cx.simulate_keystrokes("backspace");
    assert!(cx.debug_bounds("chip-command").is_none());
}

/// The owner can say which mode the agent runs in: the picker shows it, a word it does not offer changes nothing, and
/// nobody is told (a mode the owner set is not one the reader chose).
#[gpui_kit::test]
fn the_owner_sets_the_mode_the_picker_shows(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = open(cx);
    assert_eq!(cx.update(|_, cx| prompt.read(cx).mode().cloned()).as_deref(), Some("Ask first"));
    cx.update(|_, cx| prompt.update(cx, |p, cx| p.set_mode("Plan", cx)));
    assert_eq!(cx.update(|_, cx| prompt.read(cx).mode().cloned()).as_deref(), Some("Plan"));
    cx.update(|_, cx| prompt.update(cx, |p, cx| p.set_mode("Bypass", cx)));
    assert_eq!(cx.update(|_, cx| prompt.read(cx).mode().cloned()).as_deref(), Some("Plan"));
    assert!(heard.borrow().iter().all(|e| !matches!(e, PromptInputEvent::ModeChanged(_))));
}

#[test]
fn a_chips_ink_reads_on_its_fill_in_the_light_and_the_dark_theme() {
    for theme in [crate::theme::Theme::light(), crate::theme::Theme::dark()] {
        let ink = super::helpers::chip_ink(&theme);
        let seen = crate::theme::contrast(ink, theme.chip_rest);
        assert!(seen >= 4.5, "{:?}: the ink on a chip has contrast {seen}", theme.appearance);
    }
}

/// The caret of an empty box stands where the content of the first control starts (its edge and its own padding), so the text
/// and the controls' marks share one line down the box's left side.
#[gpui_kit::test]
fn the_caret_starts_where_the_first_controls_content_starts(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
    });
    let (column, cx) = cx.add_window_view(|window, cx| {
        let prompt = cx.new(|cx| {
            PromptInput::new("Ask", "", window, cx)
                .models(vec![PromptModel::new("a", "Model A"), PromptModel::new("b", "Model B")])
                .modes(vec!["Ask first".into(), "Plan".into()])
        });
        prompt.update(cx, |p, cx| p.focus_handle(cx).focus(window, cx));
        Column(prompt)
    });
    cx.simulate_resize(gpui_kit::size(px(900.), px(600.)));
    run_for(100, cx);
    let model = cx.debug_bounds("prompt-model-select").expect("the model select is drawn");
    let caret = cx.update(|_, cx| {
        let text = column.read(cx).0.read(cx).text.clone();
        text.read(cx).range_to_bounds(&(0..0)).map(|b| b.left())
    });
    let caret = caret.expect("the caret has a place");
    assert!((f32::from(caret) - f32::from(model.left()) - 8.).abs() < 0.6, "the caret is at {caret:?}, the model select at {:?} with 8 px of padding", model.left());
}

/// A mode with an icon shows it before its words, in the picker: the picker is wider by the icon and its gap.
#[gpui_kit::test]
fn a_mode_icon_draws_before_the_words_in_the_picker(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
    });
    let width = |icons: bool, cx: &mut TestAppContext| {
        let (_prompt, cx) = cx.add_window_view(|window, cx| {
            let prompt = PromptInput::new("Ask", "", window, cx).modes(vec!["Ask first".into(), "Plan".into()]);
            if icons {
                prompt.mode_icons(vec![(crate::IconName::Help, gpui_kit::hsla(0.5, 0.8, 0.6, 1.)), (crate::IconName::Checklist, gpui_kit::hsla(0.7, 0.8, 0.6, 1.))])
            } else {
                prompt
            }
        });
        cx.simulate_resize(gpui_kit::size(px(900.), px(600.)));
        run_for(100, cx);
        f32::from(cx.debug_bounds("prompt-mode-select").expect("the mode picker is drawn").size.width)
    };
    let (plain, with_icon) = (width(false, cx), width(true, cx));
    assert!(with_icon >= plain + 14., "the picker is {with_icon} wide with an icon and {plain} without");
}

mod manage {
    use std::{cell::RefCell, rc::Rc};
    use gpui_kit::{AppContext, Entity, SharedString, TestAppContext, VisualTestContext, px};
    use super::{run_for, set_appearance, Appearance};
    use crate::prompt_input::{PromptInput, PromptInputEvent, PromptModel};

    fn open(cx: &mut TestAppContext) -> (Rc<RefCell<Vec<String>>>, Entity<PromptInput>, &mut VisualTestContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            set_appearance(Appearance::Dark, cx);
        });
        let (prompt, cx) = cx.add_window_view(|window, cx| {
            PromptInput::new("Ask", "", window, cx)
                .models(vec![PromptModel::new("a", "Model A"), PromptModel::new("b", "Model B"), PromptModel::new("c", "Model C")])
                .manage_models(true)
        });
        prompt.update(cx, |p, cx| p.set_models(p.models.clone(), Some("a".into()), cx));
        let heard = Rc::new(RefCell::new(Vec::new()));
        let log = heard.clone();
        cx.update(|_, cx| {
            cx.subscribe(&prompt, move |_, event: &PromptInputEvent, _| match event {
                PromptInputEvent::ModelChanged(v) => log.borrow_mut().push(format!("picked {v}")),
                PromptInputEvent::ModelStarred(v) => log.borrow_mut().push(format!("star {v}")),
                PromptInputEvent::ModelHidden(v) => log.borrow_mut().push(format!("hide {v}")),
                PromptInputEvent::ModelsMoved(order) => {
                    log.borrow_mut().push(format!("moved {}", order.iter().map(SharedString::to_string).collect::<Vec<_>>().join(",")))
                }
                _ => {}
            })
            .detach()
        });
        cx.simulate_resize(gpui_kit::size(px(900.), px(600.)));
        run_for(100, cx);
        let at = cx.debug_bounds("prompt-model-select").expect("the picker").center();
        cx.simulate_click(at, gpui_kit::Modifiers::default());
        run_for(700, cx);
        (heard, prompt, cx)
    }

    /// In a managed picker the row under the pointer shows a star and an eye: the star makes the model the default, the eye hides it,
    /// and neither picks the model.
    #[gpui_kit::test]
    fn the_star_and_the_eye_of_a_row_report_the_model(cx: &mut TestAppContext) {
        let (heard, _prompt, cx) = open(cx);
        let row = cx.debug_bounds("select-option-2").expect("the third row");
        cx.simulate_mouse_move(row.center(), None, gpui_kit::Modifiers::default());
        run_for(300, cx);
        let star = cx.debug_bounds("select-star-2").expect("the star shows on the row under the pointer");
        cx.simulate_click(star.center(), gpui_kit::Modifiers::default());
        run_for(100, cx);
        let eye = cx.debug_bounds("select-eye-2").expect("and the eye");
        cx.simulate_click(eye.center(), gpui_kit::Modifiers::default());
        run_for(100, cx);
        assert_eq!(*heard.borrow(), ["star c", "hide c"], "each reports the model and none picked it");
    }

    /// The default keeps its filled star, and it has no eye.
    #[gpui_kit::test]
    fn the_default_row_has_a_star_and_no_eye(cx: &mut TestAppContext) {
        let (_heard, _prompt, cx) = open(cx);
        assert!(cx.debug_bounds("select-star-0").is_some(), "the default's star is always there");
        assert!(cx.debug_bounds("select-eye-0").is_none(), "but it cannot be hidden");
    }

    /// A row dragged by its grip onto another reports the models in the new order.
    #[gpui_kit::test]
    fn a_row_dragged_onto_another_reports_the_new_order(cx: &mut TestAppContext) {
        let (heard, _prompt, cx) = open(cx);
        let (from, onto) = (cx.debug_bounds("select-option-2").unwrap(), cx.debug_bounds("select-option-0").unwrap());
        let none = gpui_kit::Modifiers::default();
        let (start, end) = (gpui_kit::point(from.left() + px(14.), from.center().y), gpui_kit::point(onto.left() + px(80.), onto.center().y));
        cx.simulate_mouse_move(start, None, none);
        cx.simulate_mouse_down(start, gpui_kit::MouseButton::Left, none);
        for step in 1..=8 {
            let at = gpui_kit::point(start.x + (end.x - start.x) * (step as f32 / 8.), start.y + (end.y - start.y) * (step as f32 / 8.));
            cx.simulate_mouse_move(at, Some(gpui_kit::MouseButton::Left), none);
            cx.run_until_parked();
        }
        cx.simulate_mouse_up(end, gpui_kit::MouseButton::Left, none);
        run_for(100, cx);
        assert_eq!(*heard.borrow(), ["moved c,a,b"]);
    }
}

/// Idle, Send is one wide button (56 by 28 design pixels): the corner of the mic beside it, no Queue.
#[gpui_kit::test]
fn idle_send_is_one_wide_button(cx: &mut TestAppContext) {
    let (_, _, cx) = open(cx);
    cx.simulate_input("hi");
    cx.run_until_parked();
    let send = cx.debug_bounds("prompt-send").expect("Send is drawn");
    assert_eq!((f32::from(send.size.width), f32::from(send.size.height)), (56., 28.));
    assert!(cx.debug_bounds("prompt-queue").is_none(), "an idle agent has nothing to queue behind");
}

/// While a turn runs and the box has text, Queue stands left of Steer; a press on each says what it is.
#[gpui_kit::test]
fn while_running_with_text_queue_and_steer_are_two_buttons(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = open(cx);
    cx.update(|_, cx| prompt.update(cx, |p, cx| p.set_running(true, cx)));
    cx.simulate_input("steer it");
    cx.run_until_parked();
    let (queue, send) = (cx.debug_bounds("prompt-queue").expect("Queue is drawn"), cx.debug_bounds("prompt-send").expect("Steer is drawn"));
    assert!(queue.right() <= send.left(), "Queue stands left of Steer");
    click(cx, "prompt-queue");
    cx.simulate_input("steer it");
    cx.run_until_parked();
    click(cx, "prompt-send");
    assert_eq!(
        heard_since(&heard, 0),
        [PromptInputEvent::Queue(Message::text("steer it")), PromptInputEvent::Submit(Message::text("steer it"))]
    );
}

/// While a turn runs and the box is empty there is Stop alone: no Queue.
#[gpui_kit::test]
fn while_running_with_an_empty_box_there_is_stop_alone(cx: &mut TestAppContext) {
    let (prompt, heard, cx) = open(cx);
    cx.update(|_, cx| prompt.update(cx, |p, cx| p.set_running(true, cx)));
    cx.run_until_parked();
    assert!(cx.debug_bounds("prompt-queue").is_none());
    click(cx, "prompt-send");
    assert_eq!(heard_since(&heard, 0), [PromptInputEvent::Stop]);
}
