use super::*;

fn check(name: &str, state: CheckState) -> CheckRun {
    CheckRun {
        name: name.into(),
        summary: "".into(),
        state,
        steps: vec![],
    }
}

fn step(name: &str, state: CheckState, log: &[&str]) -> JobStep {
    JobStep {
        name: name.into(),
        state,
        seconds: None,
        log: log
            .iter()
            .map(|l| SharedString::from(l.to_string()))
            .collect(),
    }
}

#[test]
fn the_summary_names_the_worst_of_the_run() {
    use CheckState::*;
    let run = |states: &[CheckState]| {
        standing(&states.iter().map(|s| check("c", *s)).collect::<Vec<_>>()).text()
    };
    assert_eq!(
        run(&[Passed, Failed, Passed]),
        "CI is red \u{2014} 1 of 3 failing"
    );
    assert_eq!(run(&[Passed, Running, Queued]), "2 of 3 still running");
    assert_eq!(run(&[Passed, Passed]), "All 2 checks passed");
    assert_eq!(run(&[Passed]), "All 1 check passed");
    assert_eq!(run(&[Passed, Skipped]), "1 of 2 passed, none failing");
    // A tolerated failure is never the reason a run is red.
    assert_eq!(run(&[Passed, Tolerated]), "1 of 2 passed, none failing");
    assert_eq!(run(&[]), "Nothing has run yet");
}

#[test]
fn failures_come_first_then_what_was_allowed_to_fail_then_the_rest() {
    use CheckState::*;
    let checks = vec![
        check("a", Passed),
        check("b", Tolerated),
        check("c", Failed),
        check("d", Running),
    ];
    let g = groups(&checks);
    let names = |v: &[&CheckRun]| v.iter().map(|c| c.name.to_string()).collect::<Vec<_>>();
    assert_eq!(names(&g.failing), ["c"]);
    assert_eq!(names(&g.tolerated), ["b"]);
    assert_eq!(names(&g.rest), ["a", "d"]);
    assert_eq!(g.rest_text(), "1 passed, 1 other");
    assert_eq!(tolerated_text(1), "1 allowed to fail");
}

#[test]
fn the_fault_is_the_failing_step_that_did_the_work_and_the_line_that_names_why() {
    use CheckState::*;
    let mut c = check("test", Failed);
    c.steps = vec![
        step("Set up job", Passed, &[]),
        step(
            "Run cargo test",
            Failed,
            &[
                "running 23 tests",
                "test diff::line_numbers ... FAILED",
                "thread 'diff::line_numbers' panicked at src/diff.rs:40:5:",
                "assertion `left == right` failed",
                "error: test failed, to rerun pass `-p beui --lib`",
                "##[error]Process completed with exit code 101.",
            ],
        ),
        step("Post Run actions/checkout@v4", Failed, &["cleanup failed"]),
    ];
    let f = fault(&c).expect("a fault");
    assert_eq!(f.step, "Run cargo test");
    assert_eq!(
        f.line,
        "thread 'diff::line_numbers' panicked at src/diff.rs:40:5:"
    );
}

#[test]
fn an_exit_code_alone_is_the_line_only_when_nothing_better_is_said() {
    use CheckState::*;
    let mut c = check("lint", Failed);
    c.steps = vec![step(
        "Run lint",
        Failed,
        &[
            "checking 40 files",
            "##[error]Process completed with exit code 1.",
        ],
    )];
    assert_eq!(
        fault(&c).unwrap().line,
        "Process completed with exit code 1."
    );
    c.steps = vec![step("Run lint", Failed, &[])];
    assert_eq!(fault(&c).unwrap().line, "");
    assert!(fault(&check("ok", Passed)).is_none());
}

#[test]
fn a_chore_fails_only_when_no_work_did() {
    use CheckState::*;
    let mut c = check("build", Failed);
    c.steps = vec![step("Set up job", Failed, &["could not pull the image"])];
    assert_eq!(fault(&c).unwrap().step, "Set up job");
}

#[test]
fn a_log_line_loses_its_runner_markup_and_keeps_its_words() {
    let cases = [
        (
            "##[error]Process completed with exit code 1.",
            "Process completed with exit code 1.",
        ),
        ("##[warning] Node 16 is deprecated", "Node 16 is deprecated"),
        ("##[notice]Cached 3 files", "Cached 3 files"),
        ("##[debug]Evaluating condition", "Evaluating condition"),
        ("##[group]Run cargo test", "Run cargo test"),
        ("##[endgroup]", ""),
        ("::error file=src/a.rs,line=4::expected `;`", "expected `;`"),
        ("::warning::unused variable", "unused variable"),
        (
            "2026-09-28T12:00:01.1234567Z error: test failed",
            "error: test failed",
        ),
        (
            "2026-09-28T12:00:01.1234567Z ##[error]Process completed with exit code 101.",
            "Process completed with exit code 101.",
        ),
        (
            "\u{1b}[31merror\u{1b}[0m: mismatched types",
            "error: mismatched types",
        ),
        ("\u{1b}[36;1m##[error]\u{1b}[0mbuild failed", "build failed"),
        (
            "plain words ##[error] in the middle",
            "plain words ##[error] in the middle",
        ),
        ("::not a command", "::not a command"),
    ];
    for (raw, clean) in cases {
        assert_eq!(clean_line(raw), clean, "{raw:?}");
    }
}

#[test]
fn the_fault_ranks_and_shows_the_clean_line() {
    use CheckState::*;
    let mut c = check("lint", Failed);
    c.steps = vec![step(
        "Run lint",
        Failed,
        &[
            "2026-09-28T12:00:01Z checking 40 files",
            "2026-09-28T12:00:02Z ::error file=a.rs::\u{1b}[31merror\u{1b}[0m[E0308]: mismatched types",
            "2026-09-28T12:00:03Z ##[error]Process completed with exit code 1.",
        ],
    )];
    assert_eq!(fault(&c).unwrap().line, "error[E0308]: mismatched types");
    c.steps = vec![step(
        "Run tests",
        Failed,
        &["##[error]Process completed with exit code 1."],
    )];
    assert_eq!(
        fault(&c).unwrap().line,
        "Process completed with exit code 1."
    );
}

#[test]
fn a_timeout_names_the_cause_ahead_of_the_exit_code() {
    use CheckState::*;
    let mut c = check("windows-x64", Failed);
    let log = [
        "running 3 tests",
        "Timed out after 20ms waiting for the socket",
        "##[error]Process completed with exit code 1.",
    ];
    c.steps = vec![step("Run tests", Failed, &log)];
    assert_eq!(
        fault(&c).unwrap().line,
        "Timed out after 20ms waiting for the socket"
    );
    let mut d = check("linux", Failed);
    d.steps = vec![step(
        "Run tests",
        Failed,
        &[
            "test watch ... timeout (60s)",
            "Process completed with exit code 1.",
        ],
    )];
    assert_eq!(fault(&d).unwrap().line, "test watch ... timeout (60s)");
}
