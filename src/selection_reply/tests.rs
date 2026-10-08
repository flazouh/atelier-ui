use std::{cell::RefCell, rc::Rc};

use gpui_kit::{
    AppContext as _, Context, Entity, InteractiveElement, IntoElement, Modifiers, MouseButton, ParentElement, Render, Styled, TestAppContext, VisualTestContext, Window,
    base::{SelectableText, TextSelectionLayer}, div, point, px, size,
};

use super::helpers::shown;
use crate::voice_input::{VoiceInputEvent, VoiceMode};
use super::*;
use crate::{
    agent_text::{AgentText, AgentTextStatus},
    theme::{Appearance, set_appearance},
};

#[test]
fn a_long_quote_is_cut_on_one_line_with_an_ellipsis() {
    assert_eq!(shown("short", 20), "short");
    assert_eq!(shown("two\n  lines", 20), "two lines");
    assert_eq!(shown("abcdefghij", 5), "abcde…");
}

struct Host {
    reply: Entity<SelectionReply>,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .child(TextSelectionLayer)
            .child(
                div()
                    .relative()
                    .w(px(450.))
                    .h(px(100.))
                    .child(div().debug_selector(|| "words".into()).w(px(400.)).child(AgentText::new("words", "The build fails on the second run").status(AgentTextStatus::Complete)))
                    .child(div().debug_selector(|| "plain".into()).w(px(400.)).child(SelectableText::new("plain", "alpha beta gamma delta")))
                    .child(self.reply.clone()),
            )
    }
}

type Heard = Rc<RefCell<Vec<SelectionReplyEvent>>>;

fn open(cx: &mut TestAppContext) -> (Entity<Host>, Heard, &mut VisualTestContext) {
    open_with(cx, false)
}
fn open_with(cx: &mut TestAppContext, dictation: bool) -> (Entity<Host>, Heard, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::init(cx);
        set_appearance(Appearance::Dark, cx);
    });
    let heard: Heard = Rc::default();
    let log = heard.clone();
    let (host, cx) = cx.add_window_view(move |window, cx| {
        let reply = cx.new(|cx| SelectionReply::new(window, cx).dictation(dictation).presets(vec![ReplyPreset::new("Explain", "Explain this."), ReplyPreset::new("Fix", "Fix this.")]));
        cx.subscribe(&reply, move |_, _, event: &SelectionReplyEvent, _| log.borrow_mut().push(event.clone())).detach();
        Host { reply }
    });
    cx.simulate_resize(size(px(700.), px(400.)));
    settle(cx);
    (host, heard, cx)
}

/// Lets the frames come: the selection is read after one is drawn, and what it offers is drawn in the next.
fn settle(cx: &mut VisualTestContext) {
    for _ in 0..2 {
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
    }
    cx.run_until_parked();
}

/// Presses at the start of the words and lets go at their end, as a reader selecting a line does.
fn select_the_words(cx: &mut VisualTestContext) {
    let words = cx.debug_bounds("words").expect("the words are drawn");
    let y = words.top() + px(10.);
    let (from, to) = (point(words.left() + px(1.), y), point(words.left() + px(300.), y));
    cx.simulate_mouse_down(from, MouseButton::Left, Modifiers::default());
    settle(cx);
    cx.simulate_mouse_move(to, MouseButton::Left, Modifiers::default());
    settle(cx);
    cx.simulate_mouse_up(to, MouseButton::Left, Modifiers::default());
    settle(cx);
}

/// A selection that ends offers a Reply button where it ended; a press with no selection offers nothing.
#[gpui_kit::test]
fn a_selection_offers_a_reply_and_a_plain_press_does_not(cx: &mut TestAppContext) {
    let (_host, _, cx) = open(cx);
    assert!(cx.debug_bounds("selection-reply-box").is_none(), "nothing is selected yet");
    let words = cx.debug_bounds("words").unwrap();
    let at = point(words.left() + px(5.), words.top() + px(10.));
    cx.simulate_mouse_down(at, MouseButton::Left, Modifiers::default());
    cx.simulate_mouse_up(at, MouseButton::Left, Modifiers::default());
    settle(cx);
    assert!(cx.debug_bounds("selection-reply-box").is_none(), "a click selects nothing");
    select_the_words(cx);
    assert!(cx.debug_bounds("selection-reply-box").is_some(), "the selection has its Reply");
}

/// Reply opens the box with the words quoted; Enter adds, and the event carries the quote and the note.
#[gpui_kit::test]
fn a_reply_carries_the_quote_and_the_note(cx: &mut TestAppContext) {
    let (host, heard, cx) = open(cx);
    select_the_words(cx);
    assert!(cx.debug_bounds("selection-reply-box").is_some(), "the box is open");
    cx.simulate_input("it is the cache");
    cx.simulate_keystrokes("enter");
    settle(cx);
    let events = heard.borrow().clone();
    assert_eq!(events.len(), 1, "{events:?}");
    let SelectionReplyEvent::Reply { quote, note, key } = &events[0] else { panic!("not a reply: {events:?}") };
    assert!(key.is_none(), "a new reply has no key");
    assert!(quote.starts_with("The build fails"), "the quote is what was selected: {quote:?}");
    assert_eq!(note.as_ref(), "it is the cache");
    assert!(cx.debug_bounds("selection-reply-box").is_none(), "the box is gone");
    assert!(!host.read_with(cx, |h, cx| h.reply.read(cx).showing()));
}

/// A reply with no note is allowed: the quote alone says what the reader means.
#[gpui_kit::test]
fn a_reply_with_no_note_is_a_quote_alone(cx: &mut TestAppContext) {
    let (_host, heard, cx) = open(cx);
    select_the_words(cx);
    cx.simulate_keystrokes("enter");
    settle(cx);
    let events = heard.borrow().clone();
    assert!(matches!(events.as_slice(), [SelectionReplyEvent::Reply { note, .. }] if note.is_empty()), "{events:?}");
}

/// Escape drops the box, and the selection with it, so nothing is offered again.
#[gpui_kit::test]
fn escape_drops_the_reply(cx: &mut TestAppContext) {
    let (_host, heard, cx) = open(cx);
    select_the_words(cx);
    cx.simulate_input("never mind");
    cx.simulate_keystrokes("escape");
    settle(cx);
    assert!(cx.debug_bounds("selection-reply-box").is_none() && cx.debug_bounds("selection-reply-box").is_none());
    assert!(heard.borrow().is_empty(), "nothing was reported");
}

/// The round Add button sends the reply, as Enter does.
#[gpui_kit::test]
fn the_add_button_sends_the_reply(cx: &mut TestAppContext) {
    let (_host, heard, cx) = open(cx);
    select_the_words(cx);
    cx.simulate_input("yes");
    let add = cx.debug_bounds("selection-reply-add").expect("Add is drawn");
    cx.simulate_click(add.center(), Modifiers::default());
    settle(cx);
    let events = heard.borrow().clone();
    assert!(matches!(events.as_slice(), [SelectionReplyEvent::Reply { note, .. }] if note.as_ref() == "yes"), "{events:?}");
    assert!(cx.debug_bounds("selection-reply-box").is_none() && cx.debug_bounds("selection-reply-box").is_none());
}
/// The box is one compact row: a one-line quote over the note, with no Cancel button.
#[gpui_kit::test]
fn the_box_is_compact(cx: &mut TestAppContext) {
    let (_host, _, cx) = open(cx);
    select_the_words(cx);
    let card = cx.debug_bounds("selection-reply-box").expect("the box is open");
    assert!(f32::from(card.size.height) < 180., "a bubble, a row of badges and one note row: {:?}", card.size);
    assert!(cx.debug_bounds("selection-reply-cancel").is_none());
}
/// With the microphone on, it shows; pressing it asks the owner to listen, and the words go after the note.
#[gpui_kit::test]
fn the_microphone_asks_the_owner_to_listen_and_its_words_join_the_note(cx: &mut TestAppContext) {
    let (host, heard, cx) = open_with(cx, true);
    select_the_words(cx);
    cx.simulate_input("it is");
    let mic = cx.debug_bounds("selection-reply-mic").expect("the microphone is drawn");
    cx.simulate_click(mic.center(), Modifiers::default());
    settle(cx);
    assert_eq!(heard.borrow().as_slice(), [SelectionReplyEvent::Dictate(VoiceInputEvent::Start)]);
    host.update_in(cx, |h, window, cx| {
        h.reply.update(cx, |r, cx| {
            r.set_voice_listening(cx);
            r.insert_transcript("the cache", window, cx);
        })
    });
    settle(cx);
    assert_eq!(host.read_with(cx, |h, cx| h.reply.read(cx).voice_mode()), VoiceMode::Idle, "the words end the press");
    cx.simulate_keystrokes("enter");
    settle(cx);
    let events = heard.borrow().clone();
    assert!(matches!(events.last(), Some(SelectionReplyEvent::Reply { note, .. }) if note.as_ref() == "it is the cache"), "{events:?}");
}
/// Without the microphone switched on, none is drawn.
#[gpui_kit::test]
fn no_microphone_unless_the_owner_asks(cx: &mut TestAppContext) {
    let (_host, _, cx) = open(cx);
    select_the_words(cx);
    assert!(cx.debug_bounds("selection-reply-mic").is_none());
}
/// While it listens, Enter ends the press instead of sending, and Escape drops the press with the box.
#[gpui_kit::test]
fn enter_while_listening_stops_and_escape_cancels(cx: &mut TestAppContext) {
    let (host, heard, cx) = open_with(cx, true);
    select_the_words(cx);
    host.update(cx, |h, cx| h.reply.update(cx, |r, cx| r.set_voice_listening(cx)));
    settle(cx);
    cx.simulate_keystrokes("enter");
    settle(cx);
    assert_eq!(heard.borrow().as_slice(), [SelectionReplyEvent::Dictate(VoiceInputEvent::Stop)]);
    cx.simulate_keystrokes("escape");
    settle(cx);
    assert_eq!(heard.borrow().last(), Some(&SelectionReplyEvent::DictationCancel));
    assert!(cx.debug_bounds("selection-reply-box").is_none());
}
/// A selection that ends outside the parent is not this reply's: nothing is offered.
#[gpui_kit::test]
fn a_selection_ending_outside_the_parent_is_not_offered(cx: &mut TestAppContext) {
    let (_host, _, cx) = open(cx);
    let words = cx.debug_bounds("words").unwrap();
    let y = words.top() + px(10.);
    cx.simulate_mouse_down(point(words.left() + px(1.), y), MouseButton::Left, Modifiers::default());
    settle(cx);
    let beyond = point(px(600.), y);
    cx.simulate_mouse_move(beyond, MouseButton::Left, Modifiers::default());
    settle(cx);
    cx.simulate_mouse_up(beyond, MouseButton::Left, Modifiers::default());
    settle(cx);
    assert!(cx.debug_bounds("selection-reply-box").is_none());
}

/// `edit` opens the box on an earlier reply, with its note in it; adding reports the quote and the changed note.
#[gpui_kit::test]
fn edit_opens_the_box_on_an_earlier_reply(cx: &mut TestAppContext) {
    let (host, heard, cx) = open(cx);
    host.update_in(cx, |h, window, cx| h.reply.update(cx, |r, cx| r.edit(point(px(100.), px(100.)), "The build fails", "old note", "quote-1", window, cx)));
    settle(cx);
    assert!(cx.debug_bounds("selection-reply-box").is_some(), "the box is open");
    cx.simulate_input(" more");
    cx.simulate_keystrokes("enter");
    settle(cx);
    let events = heard.borrow().clone();
    let [SelectionReplyEvent::Reply { quote, note, key }] = events.as_slice() else { panic!("{events:?}") };
    assert_eq!(quote.as_ref(), "The build fails");
    assert_eq!(note.as_ref(), "old note more");
    assert_eq!(key.as_deref(), Some("quote-1"));
}

/// Plain selectable text works out its selected words as it paints: the quote is the words selected, not the whole run.
#[gpui_kit::test]
fn the_quote_of_plain_text_is_the_words_selected(cx: &mut TestAppContext) {
    let (_host, heard, cx) = open(cx);
    let plain = cx.debug_bounds("plain").expect("the plain text is drawn");
    let y = plain.top() + px(10.);
    cx.simulate_mouse_down(point(plain.left() + px(1.), y), MouseButton::Left, Modifiers::default());
    settle(cx);
    let to = point(plain.left() + px(40.), y);
    cx.simulate_mouse_move(to, MouseButton::Left, Modifiers::default());
    settle(cx);
    cx.simulate_mouse_up(to, MouseButton::Left, Modifiers::default());
    settle(cx);
    cx.simulate_keystrokes("enter");
    settle(cx);
    let events = heard.borrow().clone();
    let [SelectionReplyEvent::Reply { quote, .. }] = events.as_slice() else { panic!("{events:?}") };
    assert!(quote.starts_with("al") && !quote.contains("delta"), "only the selected words: {quote:?}");
}

/// A preset beside Reply adds the reply at once with its note: no box opens, and the selection is let go.
#[gpui_kit::test]
fn a_preset_adds_the_reply_with_its_note_and_opens_no_box(cx: &mut TestAppContext) {
    let (host, heard, cx) = open(cx);
    select_the_words(cx);
    let fix = cx.debug_bounds("selection-reply-preset-1").expect("the presets are drawn beside Reply");
    cx.simulate_click(fix.center(), Modifiers::default());
    settle(cx);
    let events = heard.borrow().clone();
    let [SelectionReplyEvent::Reply { quote, note, key }] = events.as_slice() else { panic!("one reply: {events:?}") };
    assert!(quote.starts_with("The build fails"), "{quote:?}");
    assert_eq!(note.as_ref(), "Fix this.");
    assert!(key.is_none());
    assert!(cx.debug_bounds("selection-reply-box").is_none(), "no box opens");
    assert!(cx.debug_bounds("selection-reply-box").is_none(), "and the offer is gone");
    assert!(!host.read_with(cx, |h, cx| h.reply.read(cx).showing()));
}
/// The presets are badges in one row inside the box, in order.
#[gpui_kit::test]
fn the_presets_are_badges_in_one_row_in_the_box(cx: &mut TestAppContext) {
    let (_host, _, cx) = open(cx);
    select_the_words(cx);
    let card = cx.debug_bounds("selection-reply-box").unwrap();
    let (first, second) = (cx.debug_bounds("selection-reply-preset-0").unwrap(), cx.debug_bounds("selection-reply-preset-1").unwrap());
    assert!(first.left() < second.left() && second.right() <= card.right());
    assert_eq!(first.top(), second.top(), "one row");
    assert!(first.bottom() < cx.debug_bounds("selection-reply-note").unwrap().top(), "the badges sit above the note");
}
/// The bar is drawn in the very next frame after a drag ends, before the selection has been read.
#[gpui_kit::test]
fn the_bar_is_up_in_the_first_frame_after_a_drag_ends(cx: &mut TestAppContext) {
    let (_host, _, cx) = open(cx);
    let words = cx.debug_bounds("words").expect("the words are drawn");
    let y = words.top() + px(10.);
    let (from, to) = (point(words.left() + px(1.), y), point(words.left() + px(300.), y));
    cx.simulate_mouse_down(from, MouseButton::Left, Modifiers::default());
    settle(cx);
    cx.simulate_mouse_move(to, MouseButton::Left, Modifiers::default());
    settle(cx);
    cx.simulate_mouse_up(to, MouseButton::Left, Modifiers::default());
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(cx.debug_bounds("selection-reply-box").is_some(), "the box comes up with the release");
}
/// A click that selects nothing never shows the bar, not even for a frame.
#[gpui_kit::test]
fn a_plain_click_never_flashes_the_bar(cx: &mut TestAppContext) {
    let (_host, _, cx) = open(cx);
    let words = cx.debug_bounds("words").unwrap();
    let at = point(words.left() + px(5.), words.top() + px(10.));
    cx.simulate_mouse_down(at, MouseButton::Left, Modifiers::default());
    cx.simulate_mouse_up(at, MouseButton::Left, Modifiers::default());
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(cx.debug_bounds("selection-reply-box").is_none());
}

/// The reader can type the moment the drag ends: the note has focus before the words are read.
#[gpui_kit::test]
fn typing_right_after_the_drag_goes_to_the_note(cx: &mut TestAppContext) {
    let (_host, heard, cx) = open(cx);
    let words = cx.debug_bounds("words").expect("the words are drawn");
    let y = words.top() + px(10.);
    let (from, to) = (point(words.left() + px(1.), y), point(words.left() + px(300.), y));
    cx.simulate_mouse_down(from, MouseButton::Left, Modifiers::default());
    settle(cx);
    cx.simulate_mouse_move(to, MouseButton::Left, Modifiers::default());
    settle(cx);
    cx.simulate_mouse_up(to, MouseButton::Left, Modifiers::default());
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.simulate_input("why");
    settle(cx);
    cx.simulate_keystrokes("enter");
    settle(cx);
    let events = heard.borrow().clone();
    assert!(matches!(events.as_slice(), [SelectionReplyEvent::Reply { note, quote, .. }] if note.as_ref() == "why" && quote.starts_with("The build")), "{events:?}");
}
/// A press outside the box closes it, and leaves the selection alone.
#[gpui_kit::test]
fn a_press_outside_closes_the_box(cx: &mut TestAppContext) {
    let (host, heard, cx) = open(cx);
    select_the_words(cx);
    assert!(cx.debug_bounds("selection-reply-box").is_some());
    let plain = cx.debug_bounds("plain").expect("the plain words are drawn");
    let at = point(plain.left() + px(5.), plain.top() + px(10.));
    cx.simulate_mouse_down(at, MouseButton::Left, Modifiers::default());
    settle(cx);
    assert!(cx.debug_bounds("selection-reply-box").is_none(), "the box is gone");
    assert!(!host.read_with(cx, |h, cx| h.reply.read(cx).showing()));
    assert!(heard.borrow().is_empty(), "nothing is sent");
}
/// A press inside the box does not close it.
#[gpui_kit::test]
fn a_press_inside_the_box_keeps_it(cx: &mut TestAppContext) {
    let (_host, _, cx) = open(cx);
    select_the_words(cx);
    let quote = cx.debug_bounds("selection-reply-quote").expect("the quote bubble is drawn");
    cx.simulate_mouse_down(quote.center(), MouseButton::Left, Modifiers::default());
    settle(cx);
    assert!(cx.debug_bounds("selection-reply-box").is_some(), "still open");
}
