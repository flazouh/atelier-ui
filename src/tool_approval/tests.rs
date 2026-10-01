use gpui_kit::{Context, IntoElement, Render, TestAppContext, Window, px, size};

use super::*;
use crate::theme::{Appearance, set_appearance};

struct Host {
    status: ToolApprovalStatus,
    preview: Option<ToolPreview>,
    title: Option<&'static str>,
    description: Option<&'static str>,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let mut approval = ToolApproval::new("ask", "Edit").status(self.status).parameter("file_path", "/tmp/x/src/main.rs").parameter_code("old_string", "a\nb");
        if let Some(preview) = &self.preview {
            approval = approval.preview(preview.clone());
        }
        if let Some(title) = self.title {
            approval = approval.title(title);
        }
        if let Some(description) = self.description {
            approval = approval.description(description);
        }
        div().w(px(600.)).child(approval)
    }
}

fn open(status: ToolApprovalStatus, preview: Option<ToolPreview>, cx: &mut TestAppContext) -> (gpui_kit::Entity<Host>, &mut gpui_kit::VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(true);
    });
    let (host, cx) = cx.add_window_view(move |_, _| Host { status, preview, title: None, description: None });
    cx.simulate_resize(size(px(700.), px(700.)));
    for _ in 0..3 {
        cx.run_until_parked();
        host.update(cx, |_, cx| cx.notify());
    }
    cx.run_until_parked();
    (host, cx)
}

#[gpui_kit::test]
fn a_pending_edit_shows_its_diff_and_the_raw_input_stays_behind_view_details(cx: &mut TestAppContext) {
    let (host, cx) = open(ToolApprovalStatus::Pending, Some(ToolPreview::edit("src/main.rs", "fn a() {}", "fn b() {}")), cx);
    let shown = cx.debug_bounds("approval-preview").expect("the preview is drawn while the request waits");
    assert!(f32::from(shown.size.height) > 40., "a header and rows: {shown:?}");
    host.update(cx, |h, cx| {
        h.status = ToolApprovalStatus::Approved;
        cx.notify();
    });
    for _ in 0..3 {
        cx.run_until_parked();
        host.update(cx, |_, cx| cx.notify());
    }
    assert!(cx.debug_bounds("approval-preview").is_none(), "an answered request does not repeat the change");
}

#[gpui_kit::test]
fn a_command_shows_as_a_mono_block_and_no_preview_leaves_the_card_as_it_was(cx: &mut TestAppContext) {
    let (_, cx) = open(ToolApprovalStatus::Pending, Some(ToolPreview::command("cargo test -p beui")), cx);
    assert!(cx.debug_bounds("approval-preview").is_some());
    let (_, cx) = open(ToolApprovalStatus::Pending, None, cx);
    assert!(cx.debug_bounds("approval-preview").is_none());
}

#[test]
fn a_preview_that_names_the_file_makes_the_head_one_line() {
    let (title, tool, description) = head_words("Edit", "Edit", Some("NOTES.md"), Some("NOTES.md"), None);
    assert_eq!((title.as_str(), tool, description), ("Edit NOTES.md", None, None));
    // A title the app made itself stays as it is; the tool line stays if it says something else.
    let (title, tool, description) = head_words("Change the notes", "Edit", Some("Adds a heading"), Some("NOTES.md"), None);
    assert_eq!((title.as_str(), tool.as_deref(), description.as_deref()), ("Change the notes", Some("Edit"), Some("Adds a heading")));
}

#[test]
fn with_no_preview_the_path_line_stays_and_only_a_repeated_tool_line_goes() {
    let (title, tool, description) = head_words("Edit", "Edit", Some("NOTES.md"), None, None);
    assert_eq!((title.as_str(), tool, description.as_deref()), ("Edit", None, Some("NOTES.md")), "the second line said what the first said");
    let (title, tool, description) = head_words("Allow this tool to run?", "terminal.run", None, None, None);
    assert_eq!((title.as_str(), tool.as_deref(), description), ("Allow this tool to run?", Some("terminal.run"), None));
}

#[gpui_kit::test]
fn the_head_of_an_edit_with_a_preview_has_no_tool_line_and_no_repeated_path(cx: &mut TestAppContext) {
    let (_host, cx) = open_titled("Edit", Some("NOTES.md"), Some(ToolPreview::edit("NOTES.md", "a", "b")), cx);
    assert!(cx.debug_bounds("approval-tool-line").is_none());
    assert!(cx.debug_bounds("approval-description").is_none());
    let (_host, cx) = open_titled("Edit", Some("NOTES.md"), None, cx);
    assert!(cx.debug_bounds("approval-tool-line").is_none(), "the tool line said the title again");
    assert!(cx.debug_bounds("approval-description").is_some(), "with no preview the path line stays");
}

fn open_titled<'a>(title: &'static str, description: Option<&'static str>, preview: Option<ToolPreview>, cx: &'a mut TestAppContext) -> (gpui_kit::Entity<Host>, &'a mut gpui_kit::VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(true);
    });
    let (host, cx) = cx.add_window_view(move |_, _| Host { status: ToolApprovalStatus::Pending, preview, title: Some(title), description });
    cx.simulate_resize(size(px(700.), px(700.)));
    for _ in 0..3 {
        cx.run_until_parked();
        host.update(cx, |_, cx| cx.notify());
    }
    cx.run_until_parked();
    (host, cx)
}

/// A Bash approval showed its command twice: the reason (the command, cut at an ellipsis) and then the block.
#[test]
fn a_command_preview_drops_a_description_that_only_repeats_the_command() {
    let command = "git ls-files; find /tmp/atelier-ux -name main.rs -not -path '*/target/*' 2>/dev/null; cat README.md";
    let (_, _, description) = head_words("Bash", "Bash", Some("git ls-files; find /tmp/atelier-ux -name main.rs -n…"), None, Some(command));
    assert_eq!(description, None, "the cut-off copy goes");
    let (_, _, description) = head_words("Bash", "Bash", Some(command), None, Some(command));
    assert_eq!(description, None, "the whole copy goes too");
    let (_, _, description) = head_words("Bash", "Bash", Some("Reads the readme"), None, Some(command));
    assert_eq!(description.as_deref(), Some("Reads the readme"), "a reason of its own stays");
    let (_, _, description) = head_words("Bash", "Bash", Some("git ls-files…"), None, None);
    assert_eq!(description.as_deref(), Some("git ls-files…"), "with no command block it is all there is");
}
