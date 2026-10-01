use gpui_kit::{Entity, Modifiers, TestAppContext, VisualTestContext, size};

use super::*;

fn names(list: &[&str]) -> Vec<SharedString> {
    list.iter().map(|n| SharedString::from(n.to_string())).collect()
}

#[test]
fn a_path_splits_into_the_directory_to_list_and_the_name_being_typed() {
    assert_eq!(split_path("/home/al"), ("/home/".to_string(), "al".to_string()));
    assert_eq!(split_path("/home/user/"), ("/home/user/".to_string(), String::new()));
    assert_eq!(split_path("~/sr"), ("~/".to_string(), "sr".to_string()));
    assert_eq!(split_path("src"), ("~/".to_string(), "src".to_string()), "a bare name is under home");
    assert_eq!(split_path(""), ("~/".to_string(), String::new()));
    assert_eq!(split_path("/"), ("/".to_string(), String::new()));
}

#[test]
fn matches_start_with_the_typed_name_without_case_and_dot_folders_wait_for_a_dot() {
    let all = names(&[".cache", ".config", "Code", "code-old", "src"]);
    let show = |p: &str| matches(&all, p).into_iter().map(|s| s.to_string()).collect::<Vec<_>>();
    assert_eq!(show(""), ["Code", "code-old", "src"], "no dot folders until asked");
    assert_eq!(show("co"), ["Code", "code-old"]);
    assert_eq!(show("."), [".cache", ".config"]);
    assert_eq!(show(".ca"), [".cache"]);
    assert!(show("zzz").is_empty());
}

#[test]
fn tab_completes_one_match_whole_and_several_as_far_as_they_agree() {
    let all = names(&["Documents", "Downloads", "dotfiles", "src"]);
    let refs = |p: &str| matches(&all, p);
    assert_eq!(tab_complete("/home/a/s", &refs("s")), "/home/a/src/");
    assert_eq!(tab_complete("/home/a/do", &refs("do")), "/home/a/do", "Documents, Downloads and dotfiles agree on nothing past do");
    assert_eq!(tab_complete("/home/a/Do", &refs("Do")), "/home/a/Do", "two names that agree only on Do");
    assert_eq!(tab_complete("/home/a/doc", &refs("doc")), "/home/a/Documents/");
    let alike = names(&["project-a", "project-b", "project-c"]);
    let found: Vec<&SharedString> = alike.iter().collect();
    assert_eq!(tab_complete("/w/pr", &found), "/w/project-", "as far as the names agree");
    assert_eq!(tab_complete("/w/x", &[]), "/w/x", "none leaves the text");
}

#[test]
fn the_folder_to_open_has_no_trailing_slash_except_the_root() {
    assert_eq!(folder_of("/home/user/"), "/home/user");
    assert_eq!(folder_of("  /srv  "), "/srv");
    assert_eq!(folder_of("/"), "/");
    assert_eq!(folder_of("~/"), "~");
    assert_eq!(folder_of(""), "~");
}

struct Heard {
    events: std::rc::Rc<std::cell::RefCell<Vec<String>>>,
}

fn open<'a>(start: &str, cx: &'a mut TestAppContext) -> (Entity<FolderPicker>, Heard, &'a mut VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::theme::set_appearance(crate::theme::Appearance::Light, cx);
        cx.set_reduce_motion(true);
    });
    let start = start.to_string();
    let (picker, cx) = cx.add_window_view(move |window, cx| FolderPicker::new(&start, window, cx));
    let events = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let log = events.clone();
    let sub = cx.update(|_, cx| {
        cx.subscribe(&picker, move |_, event: &FolderPickerEvent, _| {
            log.borrow_mut().push(match event {
                FolderPickerEvent::Want(dir) => format!("want {dir}"),
                FolderPickerEvent::Choose(path) => format!("choose {path}"),
                FolderPickerEvent::Cancel => "cancel".into(),
            })
        })
    });
    std::mem::forget(sub);
    cx.simulate_resize(size(gpui_kit::px(800.), gpui_kit::px(600.)));
    picker.update(cx, |p, cx| p.ask(cx));
    cx.run_until_parked();
    (picker, Heard { events }, cx)
}

fn frames(picker: &Entity<FolderPicker>, cx: &mut VisualTestContext) {
    for _ in 0..3 {
        cx.run_until_parked();
        picker.update(cx, |_, cx| cx.notify());
    }
    cx.run_until_parked();
}

fn text(picker: &Entity<FolderPicker>, cx: &mut VisualTestContext) -> String {
    picker.read_with(cx, |p, cx| p.text(cx))
}

fn set_text(picker: &Entity<FolderPicker>, text: &str, cx: &mut VisualTestContext) {
    let text = text.to_string();
    cx.update(|window, cx| picker.update(cx, |p, cx| p.set_text(text, window, cx)));
    frames(picker, cx);
}

fn focus(picker: &Entity<FolderPicker>, cx: &mut VisualTestContext) {
    let handle = picker.read_with(cx, |p, cx| p.focus_handle(cx));
    cx.update(|window, cx| handle.focus(window, cx));
}

fn listing(picker: &Entity<FolderPicker>, dir: &str, cx: &mut VisualTestContext) {
    let dir = dir.to_string();
    cx.update(|window, cx| picker.update(cx, |p, cx| p.show(&dir, Ok(vec![("code".into(), true), ("notes.txt".into(), false), ("scripts".into(), true), ("src".into(), true)]), window, cx)));
    frames(picker, cx);
}

#[gpui_kit::test]
fn the_picker_asks_for_its_directory_and_again_when_the_typed_directory_changes_and_lists_folders_only(cx: &mut TestAppContext) {
    let (picker, heard, cx) = open("/home/user/", cx);
    assert_eq!(*heard.events.borrow(), ["want /home/user/"]);
    listing(&picker, "/home/user/", cx);
    assert!(cx.debug_bounds("folder-row-2").is_some(), "three folders: code, scripts, src");
    assert!(cx.debug_bounds("folder-row-3").is_none(), "the file is left out");
    set_text(&picker, "/home/user/src/", cx);
    assert_eq!(heard.events.borrow().last().map(String::as_str), Some("want /home/user/src/"));
    assert!(cx.debug_bounds("folder-row-0").is_none(), "the old listing is not shown for another directory");
    set_text(&picker, "/home/user/src/x", cx);
    assert_eq!(heard.events.borrow().iter().filter(|e| e.starts_with("want /home/user/src/")).count(), 1, "a longer name in the same directory asks nothing more");
}

#[gpui_kit::test]
fn tab_completes_the_name_and_the_arrow_keys_and_enter_go_into_a_folder(cx: &mut TestAppContext) {
    let (picker, heard, cx) = open("/home/user/", cx);
    listing(&picker, "/home/user/", cx);
    focus(&picker, cx);
    set_text(&picker, "/home/user/sc", cx);
    cx.simulate_keystrokes("tab");
    assert_eq!(text(&picker, cx), "/home/user/scripts/", "one match completes whole");
    set_text(&picker, "/home/user/s", cx);
    cx.simulate_keystrokes("tab");
    assert_eq!(text(&picker, cx), "/home/user/s", "src and scripts agree only on s");
    cx.simulate_keystrokes("down");
    cx.simulate_keystrokes("right");
    assert_eq!(text(&picker, cx), "/home/user/src/", "down chose the second match, and the right arrow went into it");
    assert!(heard.events.borrow().iter().all(|e| !e.starts_with("choose")), "going into a folder does not open it");
}

#[gpui_kit::test]
fn enter_on_a_typed_folder_opens_it_and_escape_and_the_buttons_answer(cx: &mut TestAppContext) {
    let (picker, heard, cx) = open("/home/user/", cx);
    listing(&picker, "/home/user/", cx);
    focus(&picker, cx);
    cx.simulate_keystrokes("enter");
    assert_eq!(heard.events.borrow().last().map(String::as_str), Some("choose /home/user"), "no name being typed: the folder itself");
    cx.simulate_keystrokes("escape");
    assert_eq!(heard.events.borrow().last().map(String::as_str), Some("cancel"));
    let open_button = cx.debug_bounds("folder-open");
    let _ = open_button;
}

#[gpui_kit::test]
fn a_click_on_a_folder_goes_into_it_and_a_folder_that_cannot_be_read_says_why(cx: &mut TestAppContext) {
    let (picker, heard, cx) = open("/home/user/", cx);
    listing(&picker, "/home/user/", cx);
    let at = cx.debug_bounds("folder-row-1").expect("drawn").center();
    cx.simulate_click(at, Modifiers::default());
    frames(&picker, cx);
    assert_eq!(text(&picker, cx), "/home/user/scripts/");
    assert_eq!(heard.events.borrow().last().map(String::as_str), Some("want /home/user/scripts/"));
    cx.update(|window, cx| picker.update(cx, |p, cx| p.show("/home/user/scripts/", Err(FolderError::Denied), window, cx)));
    frames(&picker, cx);
    assert!(cx.debug_bounds("folder-row-0").is_none());
    let unreadable = picker.read_with(cx, |p, _| matches!(&p.listing, Some(Listing { folders: Err(FolderError::Denied), .. })));
    assert!(unreadable);
}

#[test]
fn a_missing_folder_says_which_name_is_not_in_which_folder_in_plain_words() {
    assert_eq!(error_words("/tmp/atelier-ux/sc~/", &FolderError::Missing), "No folder named sc~ in /tmp/atelier-ux");
    assert_eq!(error_words("/nowhere", &FolderError::Missing), "No folder named nowhere in /");
    assert_eq!(error_words("~/x", &FolderError::Missing), "No folder named x in ~");
    assert_eq!(error_words("/", &FolderError::Missing), "There is no such folder.");
    assert_eq!(error_words("/root/", &FolderError::Denied), "You may not look in /root");
    assert_eq!(error_words("/etc/hosts", &FolderError::NotAFolder), "hosts is a file, not a folder");
    assert_eq!(error_words("/srv/x", &FolderError::Other("Input/output error (os error 5)".into())), "/srv/x could not be read: Input/output error");
    assert!(FolderError::Missing.is_quiet() && FolderError::NotAFolder.is_quiet() && !FolderError::Denied.is_quiet());
}

#[gpui_kit::test]
fn typing_a_whole_path_replaces_the_prefill_and_a_refused_open_keeps_the_picker_and_the_path(cx: &mut TestAppContext) {
    let (picker, heard, cx) = open("~/", cx);
    focus(&picker, cx);
    cx.simulate_input("/tmp/atelier-ux/sc");
    frames(&picker, cx);
    assert_eq!(text(&picker, cx), "/tmp/atelier-ux/sc", "the prefill was selected, so the path replaced it");
    assert!(heard.events.borrow().iter().any(|e| e == "want /tmp/atelier-ux/"));
    cx.simulate_input("~");
    cx.simulate_keystrokes("enter");
    let last = heard.events.borrow().last().cloned().unwrap_or_default();
    assert!(last.starts_with("want /tmp/atelier-ux/sc~") || last == "choose /tmp/atelier-ux/sc~", "{last}");
    picker.update(cx, |p, cx| p.refuse(FolderError::Missing, cx));
    frames(&picker, cx);
    assert_eq!(text(&picker, cx), "/tmp/atelier-ux/sc~", "the path stays as typed");
    assert!(picker.read_with(cx, |p, _| p.refused.is_some()));
    cx.simulate_input("x");
    assert!(picker.read_with(cx, |p, _| p.refused.is_none()), "typing again clears the reason");
}

#[gpui_kit::test]
fn a_name_added_to_a_completed_path_goes_after_it(cx: &mut TestAppContext) {
    let (picker, _, cx) = open("/home/user/", cx);
    listing(&picker, "/home/user/", cx);
    focus(&picker, cx);
    set_text(&picker, "/home/user/sc", cx);
    cx.simulate_keystrokes("tab");
    cx.simulate_input("x");
    assert_eq!(text(&picker, cx), "/home/user/scripts/x", "the caret was at the end after Tab");
}

#[gpui_kit::test]
fn enter_opens_the_folder_the_field_names_or_the_row_the_arrows_chose(cx: &mut TestAppContext) {
    let (picker, heard, cx) = open("/home/user/", cx);
    listing(&picker, "/home/user/", cx);
    focus(&picker, cx);
    set_text(&picker, "/home/user/sc", cx);
    cx.simulate_keystrokes("enter");
    assert_eq!(heard.events.borrow().last().map(String::as_str), Some("choose /home/user/sc"), "the field names it, so it is what opens, even if it is only part of a name");
    set_text(&picker, "/home/user/s", cx);
    cx.simulate_keystrokes("down");
    cx.simulate_keystrokes("enter");
    assert_eq!(heard.events.borrow().last().map(String::as_str), Some("choose /home/user/src"), "the arrows chose src");
    set_text(&picker, "/home/user/s", cx);
    cx.simulate_keystrokes("enter");
    assert_eq!(heard.events.borrow().last().map(String::as_str), Some("choose /home/user/s"), "typing cleared the choice");
}

#[gpui_kit::test]
fn tab_before_the_listing_arrives_completes_when_it_does(cx: &mut TestAppContext) {
    let (picker, _, cx) = open("/home/user/", cx);
    focus(&picker, cx);
    set_text(&picker, "/home/user/sc", cx);
    cx.simulate_keystrokes("tab");
    assert_eq!(text(&picker, cx), "/home/user/sc", "nothing to complete from yet");
    listing(&picker, "/home/user/", cx);
    assert_eq!(text(&picker, cx), "/home/user/scripts/", "the listing came, and Tab finished");
}
