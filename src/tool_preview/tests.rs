use super::*;

fn kinds(rows: &[DiffLine]) -> String {
    rows.iter().map(|r| match r.kind { DiffLineKind::Context => ' ', DiffLineKind::Added => '+', DiffLineKind::Removed => '-', DiffLineKind::Hunk => '@' }).collect()
}

#[test]
fn a_changed_line_is_one_removed_and_one_added_between_kept_lines() {
    let rows = line_diff("fn a() {\n    1\n}\n", "fn a() {\n    2\n}\n", 1);
    assert_eq!(kinds(&rows), " -+ ");
    assert_eq!(rows[1].text.as_ref(), "    1");
    assert_eq!(rows[2].text.as_ref(), "    2");
}

#[test]
fn lines_are_numbered_from_where_the_edit_starts() {
    let rows = line_diff("a\nb\nc", "a\nB\nc", 40);
    let numbers: Vec<_> = rows.iter().map(|r| (r.old_line, r.new_line)).collect();
    assert_eq!(numbers, [(Some(40), Some(40)), (Some(41), None), (None, Some(41)), (Some(42), Some(42))]);
}

#[test]
fn an_insert_a_delete_and_a_move_are_told_apart() {
    assert_eq!(kinds(&line_diff("a\nc", "a\nb\nc", 1)), " + ");
    assert_eq!(kinds(&line_diff("a\nb\nc", "a\nc", 1)), " - ");
    assert_eq!(kinds(&line_diff("a", "", 1)), "-");
    assert_eq!(kinds(&line_diff("", "x\ny", 1)), "++", "a new file is all added");
    assert!(line_diff("", "", 1).is_empty());
    assert_eq!(kinds(&line_diff("same\n", "same", 1)), " ", "a final newline is not a line");
}

#[test]
fn a_very_large_pair_falls_back_to_all_removed_then_all_added() {
    let a = "x\n".repeat(2500);
    let b = "y\n".repeat(2500);
    let rows = line_diff(&a, &b, 1);
    assert_eq!(rows.len(), 5000);
    assert!(rows[..2500].iter().all(|r| r.kind == DiffLineKind::Removed) && rows[2500..].iter().all(|r| r.kind == DiffLineKind::Added));
}

#[test]
fn several_edits_get_a_header_each_and_one_edit_gets_none() {
    let one = ToolPreview::edit("src/a.rs", "1", "2");
    assert_eq!(kinds(&one.rows()), "-+");
    let two = ToolPreview::edits("src/a.rs", vec![TextEdit::new("1", "2"), TextEdit::new("x", "x\ny")]);
    let rows = two.rows();
    assert_eq!(kinds(&rows), "@-+@ +");
    assert_eq!(rows[0].text.as_ref(), "@@ change 1 of 2 @@");
    assert_eq!(rows[3].text.as_ref(), "@@ change 2 of 2 @@");
}

#[test]
fn a_written_file_is_all_new_lines_and_a_command_has_no_rows() {
    let written = ToolPreview::written("README.md", "# Title\n\nWords\n");
    assert_eq!(kinds(&written.rows()), "+++");
    assert_eq!(written.path().map(|p| p.as_ref()), Some("README.md"));
    let command = ToolPreview::command("cargo test -p beui");
    assert!(command.rows().is_empty() && command.path().is_none());
}

#[test]
fn a_path_inside_the_project_shows_relative_and_one_outside_shows_whole() {
    assert_eq!(relative_path("/tmp/atelier-ux/scratch/src/main.rs", "/tmp/atelier-ux/scratch").as_ref(), "src/main.rs");
    assert_eq!(relative_path("/tmp/atelier-ux/scratch/src/main.rs", "/tmp/atelier-ux/scratch/").as_ref(), "src/main.rs", "a trailing slash on the root");
    assert_eq!(relative_path("/etc/hosts", "/tmp/atelier-ux/scratch").as_ref(), "/etc/hosts");
    assert_eq!(relative_path("/tmp/atelier-ux/scratch2/a.rs", "/tmp/atelier-ux/scratch").as_ref(), "/tmp/atelier-ux/scratch2/a.rs", "a sibling that shares a prefix");
    assert_eq!(relative_path("src/main.rs", "/tmp/x").as_ref(), "src/main.rs", "already relative");
    assert_eq!(relative_path("/a", "").as_ref(), "/a");
}
