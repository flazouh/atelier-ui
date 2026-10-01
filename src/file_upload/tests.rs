use std::{cell::RefCell, rc::Rc};

use gpui_kit::{Entity, ExternalPaths, FileDropEvent, Modifiers, TestAppContext, VisualTestContext, point, size};

use super::*;
use crate::theme::{Appearance, set_appearance};

#[test]
fn a_size_is_written_in_the_fewest_digits() {
    assert_eq!(format_bytes(0), "0 B");
    assert_eq!(format_bytes(512), "512 B");
    assert_eq!(format_bytes(1024), "1.0 KB");
    assert_eq!(format_bytes(1536), "1.5 KB");
    assert_eq!(format_bytes(18_400_000), "18 MB");
    assert_eq!(format_bytes(2_800_000), "2.7 MB");
    assert_eq!(format_bytes(84_200_000), "80 MB");
    assert_eq!(format_bytes(5 * 1024 * 1024 * 1024 * 1024), "5.0 TB");
    assert_eq!(format_bytes(u64::MAX), "16777216 TB", "no unit past a terabyte");
}

#[test]
fn a_kind_is_the_extension_else_the_media_subtype_else_file() {
    let item = |name: &str| UploadItem::new("x", name.to_string(), 1);
    assert_eq!(kind_of(&item("brand-assets.zip")), "ZIP");
    assert_eq!(kind_of(&item("archive.tar.gz")), "GZ");
    assert_eq!(kind_of(&item("Makefile")), "FILE");
    assert_eq!(kind_of(&item("Makefile").mime("text/x-makefile")), "X-MAKEFILE");
    assert_eq!(kind_of(&item("noext.").mime("video/quicktime")), "QUICKTIME", "an empty extension counts for nothing");
    assert_eq!(kind_of(&item(".gitignore")), "GITIGNORE", "as the web reads it");
}

#[test]
fn an_icon_follows_the_media_type_then_the_extension() {
    let item = |name: &str, mime: Option<&str>| {
        let i = UploadItem::new("x", name.to_string(), 1);
        match mime {
            Some(m) => i.mime(m.to_string()),
            None => i,
        }
    };
    assert_eq!(icon_of(&item("a.bin", Some("image/png"))), IconName::Image);
    assert_eq!(icon_of(&item("a.mov", Some("video/quicktime"))), IconName::Movie);
    assert_eq!(icon_of(&item("a.zip", None)), IconName::FolderZip);
    assert_eq!(icon_of(&item("a.csv", None)), IconName::TableChart);
    assert_eq!(icon_of(&item("a.pdf", None)), IconName::Description);
    assert_eq!(icon_of(&item("a.rs", None)), IconName::Code);
    assert_eq!(icon_of(&item("a.mp3", None)), IconName::AudioFile);
    assert_eq!(icon_of(&item("a.unknown", None)), IconName::Draft);
}

#[test]
fn progress_is_kept_within_0_and_100_and_a_file_that_arrived_is_100() {
    assert_eq!(clamp_progress(42., UploadStatus::Uploading), 42.);
    assert_eq!(clamp_progress(-5., UploadStatus::Uploading), 0.);
    assert_eq!(clamp_progress(140., UploadStatus::Error), 100.);
    assert_eq!(clamp_progress(f32::NAN, UploadStatus::Queued), 0.);
    assert_eq!(clamp_progress(10., UploadStatus::Success), 100.);
}

fn files(names: &[&str]) -> (tempfile::TempDir, Vec<PathBuf>) {
    let dir = tempfile::tempdir().unwrap();
    let paths: Vec<PathBuf> = names.iter().map(|n| {
        let path = dir.path().join(n);
        std::fs::write(&path, vec![b'x'; 2048]).unwrap();
        path
    }).collect();
    (dir, paths)
}

#[test]
fn the_queue_takes_the_accepted_kinds_within_the_room_and_one_when_not_multiple() {
    let (dir, mut paths) = files(&["a.pdf", "b.png", "c.PDF", "d.zip"]);
    paths.push(dir.path().join("a-folder"));
    std::fs::create_dir(dir.path().join("a-folder")).unwrap();
    let names = |taken: Vec<PathBuf>| taken.into_iter().map(|p| p.file_name().unwrap().to_string_lossy().to_string()).collect::<Vec<_>>();
    assert_eq!(names(take_paths(&paths, &[], None, true)), ["a.pdf", "b.png", "c.PDF", "d.zip"], "a folder is not a file");
    assert_eq!(names(take_paths(&paths, &["pdf".to_string()], None, true)), ["a.pdf", "c.PDF"], "an extension counts without regard to case");
    assert_eq!(names(take_paths(&paths, &[], Some(2), true)), ["a.pdf", "b.png"]);
    assert_eq!(names(take_paths(&paths, &[], Some(3), false)), ["a.pdf"], "one when not multiple");
    assert!(take_paths(&paths, &[], Some(0), true).is_empty());
    assert_eq!(names(take_paths(&paths, &[], None, false)), ["a.pdf"]);
}

fn open(reduce: bool, cx: &mut TestAppContext) -> (Entity<FileUpload>, Rc<RefCell<Vec<String>>>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Light, cx);
        cx.set_reduce_motion(reduce);
        crate::motion::clock::freeze();
    });
    let (upload, cx) = cx.add_window_view(|_, _| FileUpload::new("up").max_files(3));
    cx.simulate_resize(size(gpui_kit::px(600.), gpui_kit::px(700.)));
    let heard = Rc::new(RefCell::new(Vec::new()));
    let log = heard.clone();
    let sub = cx.update(|_, cx| {
        cx.subscribe(&upload, move |_, event: &FileUploadEvent, _| {
            log.borrow_mut().push(match event {
                FileUploadEvent::Added(items) => format!("added {}", items.iter().map(|i| i.name.to_string()).collect::<Vec<_>>().join(",")),
                FileUploadEvent::Removed(item) => format!("removed {}", item.name),
                FileUploadEvent::Retried(item) => format!("retried {} {:?} {}", item.name, item.status, item.progress),
            })
        })
    });
    std::mem::forget(sub);
    settle(&upload, cx);
    (upload, heard, cx)
}

fn settle(upload: &Entity<FileUpload>, cx: &mut VisualTestContext) {
    for _ in 0..4 {
        cx.run_until_parked();
        upload.update(cx, |_, cx| cx.notify());
    }
    cx.run_until_parked();
}

fn queue() -> Vec<UploadItem> {
    vec![
        UploadItem::new("zip", "brand-assets.zip", 18_400_000).progress(100.).status(UploadStatus::Success),
        UploadItem::new("mov", "release-cut.mov", 84_200_000).progress(58.).status(UploadStatus::Uploading),
        UploadItem::new("pdf", "vendor-contract.pdf", 2_800_000).progress(32.).status(UploadStatus::Error).error("Connection lost"),
    ]
}

fn click(cx: &mut VisualTestContext, name: &str) {
    let at = cx.debug_bounds(Box::leak(name.to_string().into_boxed_str())).unwrap_or_else(|| panic!("{name} is not drawn")).center();
    cx.simulate_click(at, Modifiers::default());
    cx.run_until_parked();
}

#[gpui_kit::test]
fn each_file_is_a_row_and_only_a_failed_one_has_retry(cx: &mut TestAppContext) {
    let (upload, _, cx) = open(true, cx);
    upload.update(cx, |u, cx| u.set_items(queue(), cx));
    settle(&upload, cx);
    for id in ["zip", "mov", "pdf"] {
        assert!(cx.debug_bounds(Box::leak(format!("upload-row-{id}").into_boxed_str())).is_some(), "{id} is a row");
    }
    // Three rows, three remove buttons; one retry.
    assert!(cx.debug_bounds("upload-retry").is_some());
    let bar = |id: &str, cx: &mut VisualTestContext| cx.debug_bounds(Box::leak(format!("upload-bar-{id}").into_boxed_str())).map(|b| f32::from(b.size.width));
    assert!(bar("pdf", cx).is_none(), "a failed file shows no bar");
    let (full, part) = (bar("zip", cx).expect("bar"), bar("mov", cx).expect("bar"));
    assert!(part < full && (part / full - 0.58).abs() < 0.03, "58% of the track: {part} of {full}");
}

#[gpui_kit::test]
fn remove_sends_the_file_and_retry_starts_it_again_from_zero(cx: &mut TestAppContext) {
    let (upload, heard, cx) = open(true, cx);
    upload.update(cx, |u, cx| u.set_items(queue(), cx));
    settle(&upload, cx);
    click(cx, "upload-retry");
    settle(&upload, cx);
    assert_eq!(*heard.borrow(), ["retried vendor-contract.pdf Uploading 0"]);
    assert!(cx.debug_bounds("upload-retry").is_none(), "retrying, it is no longer failed");
    let pdf = upload.read_with(cx, |u, _| u.items().into_iter().find(|i| i.id == "pdf").unwrap());
    assert_eq!((pdf.status, pdf.progress, pdf.error), (UploadStatus::Uploading, 0., None));
    upload.update(cx, |u, cx| u.remove(&"zip".into(), cx));
    settle(&upload, cx);
    assert_eq!(heard.borrow().last().map(String::as_str), Some("removed brand-assets.zip"));
    assert_eq!(upload.read_with(cx, |u, _| u.items().len()), 2);
    assert!(cx.debug_bounds("upload-row-zip").is_none(), "with Reduce Motion the row is gone at once");
}

#[gpui_kit::test]
fn with_motion_a_removed_row_stays_in_place_until_its_exit_has_run(cx: &mut TestAppContext) {
    let (upload, _, cx) = open(false, cx);
    upload.update(cx, |u, cx| u.set_items(queue(), cx));
    crate::motion::clock::advance(std::time::Duration::from_millis(400));
    settle(&upload, cx);
    upload.update(cx, |u, cx| u.remove(&"mov".into(), cx));
    settle(&upload, cx);
    assert!(cx.debug_bounds("upload-row-mov").is_some(), "the leaving row is still in the queue");
    assert_eq!(upload.read_with(cx, |u, _| u.items().len()), 2, "though the queue no longer counts it");
    crate::motion::clock::advance(std::time::Duration::from_millis(300));
    settle(&upload, cx);
    assert!(cx.debug_bounds("upload-row-mov").is_none(), "and after 220 ms it is gone");
}

#[gpui_kit::test]
fn a_status_change_swaps_the_mark_and_the_bar_moves_to_the_new_number(cx: &mut TestAppContext) {
    let (upload, _, cx) = open(false, cx);
    upload.update(cx, |u, cx| u.set_items(queue(), cx));
    crate::motion::clock::advance(std::time::Duration::from_millis(500));
    settle(&upload, cx);
    upload.update(cx, |u, cx| u.update("mov", |i| { i.progress = 100.; i.status = UploadStatus::Success; }, cx));
    let (swapping, bar) = upload.read_with(cx, |u, _| { let r = u.rows.iter().find(|r| r.item.id == "mov").unwrap(); (r.swap.as_ref().map(|(old, _)| *old), r.bar.target()) });
    assert_eq!(swapping, Some(UploadStatus::Uploading), "the old mark leaves first");
    assert_eq!(bar, 1.);
    crate::motion::clock::advance(std::time::Duration::from_millis(400));
    settle(&upload, cx);
    assert!(upload.read_with(cx, |u, _| u.rows.iter().find(|r| r.item.id == "mov").unwrap().swap.is_none()));
    // The same status again is no swap.
    upload.update(cx, |u, cx| u.update("mov", |i| i.progress = 100., cx));
    assert!(upload.read_with(cx, |u, _| u.rows.iter().find(|r| r.item.id == "mov").unwrap().swap.is_none()));
}

#[gpui_kit::test]
fn files_dropped_on_the_dropzone_join_the_queue_as_uploading_up_to_the_limit(cx: &mut TestAppContext) {
    let (upload, heard, cx) = open(true, cx);
    let (_dir, paths) = files(&["one.txt", "two.txt", "three.txt", "four.txt"]);
    let over = cx.debug_bounds("upload-dropzone").expect("the dropzone is drawn").center();
    cx.simulate_event(FileDropEvent::Entered { position: over, paths: ExternalPaths(paths.iter().cloned().collect()) });
    cx.simulate_event(FileDropEvent::Submit { position: over });
    settle(&upload, cx);
    assert_eq!(*heard.borrow(), ["added one.txt,two.txt,three.txt"], "three is the limit");
    let items = upload.read_with(cx, |u, _| u.items());
    assert!(items.iter().all(|i| i.status == UploadStatus::Uploading && i.progress == 0. && i.size == 2048));
    assert!(upload.read_with(cx, |u, _| u.maxed()), "the dropzone now says the limit is reached");
    // A drop on a full queue adds nothing.
    let more = ExternalPaths(paths[3..].iter().cloned().collect());
    cx.simulate_event(FileDropEvent::Entered { position: point(over.x, over.y), paths: more });
    cx.simulate_event(FileDropEvent::Submit { position: over });
    settle(&upload, cx);
    assert_eq!(heard.borrow().len(), 1);
}

#[gpui_kit::test]
fn a_row_is_82px_tall_with_a_bar_and_70_without_as_on_the_web(cx: &mut TestAppContext) {
    let (upload, _, cx) = open(true, cx);
    upload.update(cx, |u, cx| u.set_items(queue(), cx));
    settle(&upload, cx);
    let height = |id: &str, cx: &mut VisualTestContext| f32::from(cx.debug_bounds(Box::leak(format!("upload-row-{id}").into_boxed_str())).unwrap().size.height);
    assert_eq!(height("zip", cx), 82., "12 + 56 + 12 + 2 border");
    assert_eq!(height("pdf", cx), 70., "12 + 44 + 12 + 2 border");
}

#[gpui_kit::test]
fn the_centered_dropzone_is_about_as_tall_as_the_web_s_230_and_the_row_one_about_100(cx: &mut TestAppContext) {
    let (upload, _, cx) = open(true, cx);
    let zone = |cx: &mut VisualTestContext| f32::from(cx.debug_bounds("upload-dropzone").unwrap().size.height);
    upload.update(cx, |u, cx| u.set_variant(UploadVariant::Centered, cx));
    settle(&upload, cx);
    let centered = zone(cx);
    assert!((228.0..=232.0).contains(&centered), "min-h-56 and its content: {centered}");
    upload.update(cx, |u, cx| u.set_variant(UploadVariant::Row, cx));
    settle(&upload, cx);
    let row = zone(cx);
    assert!((94.0..=98.0).contains(&row), "20 + 56 + 20 + 2: {row}");
}
