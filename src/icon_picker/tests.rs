use gpui_kit::AppContext;
use std::{cell::RefCell, rc::Rc};
use gpui_kit::{Context, IntoElement, ParentElement, Render, Styled, TestAppContext, Window, div, px};
use super::*;
use crate::theme::{Appearance, set_appearance};

struct Host {
    picker: gpui_kit::Entity<IconPicker>,
}
impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(520.)).p(px(16.)).child(self.picker.clone())
    }
}

fn open<'a>(
    paths: &[&str],
    cx: &'a mut TestAppContext,
) -> (gpui_kit::Entity<IconPicker>, Rc<RefCell<Vec<String>>>, &'a mut gpui_kit::VisualTestContext) {
    open_at(paths, None, cx)
}

fn open_at<'a>(
    paths: &[&str],
    root: Option<std::path::PathBuf>,
    cx: &'a mut TestAppContext,
) -> (gpui_kit::Entity<IconPicker>, Rc<RefCell<Vec<String>>>, &'a mut gpui_kit::VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(true);
    });
    let events = Rc::new(RefCell::new(Vec::new()));
    let log = events.clone();
    let paths: Vec<String> = paths.iter().map(|p| p.to_string()).collect();
    let (host, cx) = cx.add_window_view(move |window, cx| {
        let picker = cx.new(|cx| IconPicker::new(paths, root, window, cx));
        cx.subscribe(&picker, move |_, _, event: &IconPickerEvent, _| {
            log.borrow_mut().push(match event {
                IconPickerEvent::Choose(path) => format!("choose {path}"),
                IconPickerEvent::Clear => "clear".to_string(),
                IconPickerEvent::Cancel => "cancel".to_string(),
            })
        })
        .detach();
        Host { picker }
    });
    let picker = host.read_with(cx, |h, _| h.picker.clone());
    let focus = cx.update(|_, cx| gpui_kit::Focusable::focus_handle(&picker, cx));
    cx.update(|window, cx| focus.focus(window, cx));
    cx.run_until_parked();
    (picker, events, cx)
}

fn rows(cx: &mut gpui_kit::VisualTestContext) -> usize {
    const ROWS: [&str; 8] = ["icon-row-0", "icon-row-1", "icon-row-2", "icon-row-3", "icon-row-4", "icon-row-5", "icon-row-6", "icon-row-7"];
    ROWS.iter().take_while(|name| cx.debug_bounds(name).is_some()).count()
}

#[gpui_kit::test]
fn it_lists_the_images_most_likely_first_and_enter_chooses_the_first(cx: &mut TestAppContext) {
    let (_, events, cx) = open(&["docs/shots/home.png", "assets/logo.svg", "favicon.ico"], cx);
    assert_eq!(rows(cx), 3);
    cx.simulate_keystrokes("enter");
    assert_eq!(events.borrow().as_slice(), ["choose favicon.ico"]);
}

#[gpui_kit::test]
fn typing_narrows_the_list_and_the_arrows_move_the_choice(cx: &mut TestAppContext) {
    let (_, events, cx) = open(&["assets/logo.svg", "public/logo.png", "public/hero.png"], cx);
    cx.simulate_input("logo");
    cx.run_until_parked();
    assert_eq!(rows(cx), 2, "two paths hold logo");
    cx.simulate_keystrokes("down enter");
    assert_eq!(events.borrow().as_slice(), ["choose public/logo.png"]);
}

#[gpui_kit::test]
fn escape_cancels(cx: &mut TestAppContext) {
    let (_, events, cx) = open(&["logo.svg"], cx);
    cx.simulate_keystrokes("escape");
    assert_eq!(events.borrow().as_slice(), ["cancel"]);
}

#[gpui_kit::test]
fn a_project_with_no_images_says_so(cx: &mut TestAppContext) {
    let (_, events, cx) = open(&[], cx);
    assert_eq!(rows(cx), 0);
    assert!(cx.debug_bounds("icon-empty").is_some());
    cx.simulate_keystrokes("enter");
    assert!(events.borrow().is_empty(), "nothing to choose");
}

#[gpui_kit::test]
fn a_press_on_use_the_letter_clears_the_image(cx: &mut TestAppContext) {
    let (_, events, cx) = open(&["logo.svg"], cx);
    let at = cx.debug_bounds("icon-clear").expect("the button is drawn").center();
    cx.simulate_click(at, gpui_kit::Modifiers::default());
    assert_eq!(events.borrow().as_slice(), ["clear"]);
}

/// A local project shows each image itself, 20px, in its row; one on another host shows the file mark.
#[gpui_kit::test]
fn a_local_project_shows_a_thumbnail_of_each_image(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("logo.svg"), r##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><circle cx="32" cy="32" r="30" fill="#d53f8c"/></svg>"##).unwrap();
    let (_, _, cx) = open_at(&["logo.svg"], Some(dir.path().to_path_buf()), cx);
    let thumb = cx.debug_bounds("icon-thumb-0").expect("the image is drawn in its row");
    assert_eq!((f32::from(thumb.size.width), f32::from(thumb.size.height)), (20., 20.));
}

#[gpui_kit::test]
fn a_remote_project_shows_the_file_mark_instead(cx: &mut TestAppContext) {
    let (_, _, cx) = open_at(&["logo.svg"], None, cx);
    assert!(cx.debug_bounds("icon-thumb-0").is_none());
    assert_eq!(rows(cx), 1);
}
