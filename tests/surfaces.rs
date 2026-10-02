//! Two rules that keep parts visible on the surface they really sit on. Like `no_colour_literals.rs`,
//! this reads the sources, code only (comments dropped).
//!
//! - A box in the agent panel fills `card_strong`: the panel is a `card`, so a `card` box in it does not
//!   part from it (the prompt input's rule).
//! - Muted is never faded with an opacity: a quiet mark takes `Theme::faint`, which holds 3:1 on every
//!   surface, and quiet text takes `muted_foreground` whole.

use std::path::Path;

/// The parts drawn inside an agent panel.
const IN_THE_PANEL: &[&str] = &[
    "tool_call",
    "file_diff",
    "changed_files",
    "todo_list",
    "subagent_card",
    "subagent_row",
    "tool_approval",
    "pr_card",
    "agent_text",
];

/// Fades of muted that mark nothing: the editor's indent guides and whitespace dots, a control hidden
/// until its row is hovered, and the model badge's shimmer, which sweeps to the ink.
const DECORATIVE: &[&str] = &[
    "border: muted.opacity(0.14)",
    "editor_invisible: Some(muted.opacity(0.25))",
    "color(muted.opacity(0.0))",
    "mix(muted.opacity(0.5), theme.foreground, p)",
];

fn sources(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if path.is_dir() {
            sources(&path, out);
        } else if name.ends_with(".rs") && name != "tests.rs" {
            out.push(path);
        }
    }
}

fn code_lines(file: &Path) -> Vec<(usize, String)> {
    std::fs::read_to_string(file)
        .unwrap()
        .lines()
        .enumerate()
        .map(|(n, line)| (n + 1, line.split("//").next().unwrap_or("").to_string()))
        .collect()
}

fn in_the_panel(file: &Path, src: &Path) -> bool {
    let module = file
        .strip_prefix(src)
        .unwrap()
        .components()
        .next()
        .unwrap()
        .as_os_str()
        .to_string_lossy()
        .into_owned();
    IN_THE_PANEL.contains(&module.trim_end_matches(".rs"))
}

#[test]
fn a_box_in_the_agent_panel_parts_from_the_panel() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    sources(&src, &mut files);
    let mut found = Vec::new();
    for file in files.iter().filter(|f| in_the_panel(f, &src)) {
        for (n, code) in code_lines(file) {
            if code.contains(".bg(theme.card)") || code.contains("s.bg(theme.card)") {
                found.push(format!("{}:{n}: {}", file.display(), code.trim()));
            }
        }
    }
    assert!(
        found.is_empty(),
        "a card box on the card panel, which does not show:\n{}",
        found.join("\n")
    );
}

#[test]
fn muted_is_never_faded() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    sources(&src, &mut files);
    let mut found = Vec::new();
    for file in &files {
        for (n, code) in code_lines(file) {
            let faded =
                code.contains("muted.opacity(") || code.contains("muted_foreground.opacity(");
            if faded && !DECORATIVE.iter().any(|d| code.contains(d)) {
                found.push(format!("{}:{n}: {}", file.display(), code.trim()));
            }
        }
    }
    assert!(
        found.is_empty(),
        "muted faded below what reads; use Theme::faint or muted whole:\n{}",
        found.join("\n")
    );
}
