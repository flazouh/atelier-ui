use super::*;

fn open() -> MergeFacts {
    MergeFacts::default()
}

fn choice(facts: &MergeFacts) -> Choice {
    first_choice(facts, None)
}

fn label(facts: &MergeFacts) -> String {
    button(facts, &choice(facts)).unwrap().label
}

#[test]
fn the_blockers_come_in_their_order_with_their_words() {
    let facts = MergeFacts {
        draft: true,
        conflicts: vec!["src/a.rs".into()],
        behind: Some(vec![UpdateWay::Rebase, UpdateWay::Merge]),
        checks_failing: 1,
        checks_running: 2,
        review: ReviewNeed::ChangesAsked(vec!["Ada".into()]),
        queue: Some(Queue::default()),
        rights: Rights::Cannot,
        ..open()
    };
    let names: Vec<String> = blockers(&facts).iter().map(Blocker::name).collect();
    assert_eq!(
        names,
        [
            "This pull request is still a draft",
            "This branch has conflicts that must be resolved",
            "This branch is out of date with the base branch",
            "1 required check failing",
            "2 required checks still running",
            "Changes asked by Ada",
            "This repository merges through a queue",
            "You cannot merge into this repository",
        ]
    );
    assert_eq!(standing(&facts), "Blocked: still a draft");
}

#[test]
fn the_standing_line_reads_as_the_brief_writes_it() {
    assert_eq!(standing(&open()), "Ready to merge");
    let reader = MergeFacts {
        rights: Rights::Cannot,
        ..open()
    };
    assert_eq!(
        standing(&reader),
        "Ready to merge, by someone with write access"
    );
    assert_eq!(
        standing(&MergeFacts {
            checks_failing: 1,
            ..reader
        }),
        "Blocked: 1 check failing",
        "a real blocker still leads"
    );
    assert_eq!(
        standing(&MergeFacts {
            checks_running: 2,
            ..open()
        }),
        "2 checks still running"
    );
    assert_eq!(
        standing(&MergeFacts {
            review: ReviewNeed::ChangesAsked(vec!["Ada".into()]),
            ..open()
        }),
        "Blocked: changes asked by Ada"
    );
    assert_eq!(
        standing(&MergeFacts {
            review: ReviewNeed::ChangesAsked(vec!["Ada".into(), "Rui".into(), "Kai".into()]),
            ..open()
        }),
        "Blocked: changes asked by Ada, Rui and Kai"
    );
    assert_eq!(
        standing(&MergeFacts {
            state: PullState::Merged,
            ..open()
        }),
        "Merged"
    );
    assert_eq!(
        standing(&MergeFacts {
            auto_merge: Some(true),
            checks_running: 1,
            ..open()
        }),
        "Merges when ready"
    );
    assert_eq!(
        standing(&MergeFacts {
            queue: Some(Queue {
                queued: true,
                position: Some(3)
            }),
            ..open()
        }),
        "In the merge queue, number 3"
    );
}

#[test]
fn the_menu_offers_the_allowed_methods_default_first() {
    let facts = MergeFacts {
        methods: vec![MergeMethod::Merge, MergeMethod::Rebase],
        default_method: MergeMethod::Rebase,
        ..open()
    };
    assert_eq!(offered(&facts), [MergeMethod::Rebase, MergeMethod::Merge]);
    let all = MergeFacts {
        default_method: MergeMethod::Squash,
        ..open()
    };
    assert_eq!(
        offered(&all),
        [MergeMethod::Squash, MergeMethod::Merge, MergeMethod::Rebase]
    );
}

#[test]
fn the_remembered_method_wins_while_the_repository_allows_it() {
    let facts = MergeFacts {
        default_method: MergeMethod::Squash,
        ..open()
    };
    assert_eq!(
        pick_method(&facts, Some(MergeMethod::Rebase)),
        MergeMethod::Rebase
    );
    assert_eq!(
        pick_method(&facts, None),
        MergeMethod::Squash,
        "nothing remembered: the default"
    );
    let squash_only = MergeFacts {
        methods: vec![MergeMethod::Squash],
        default_method: MergeMethod::Squash,
        ..open()
    };
    assert_eq!(
        pick_method(&squash_only, Some(MergeMethod::Rebase)),
        MergeMethod::Squash,
        "no longer allowed: the default"
    );
}

#[test]
fn the_button_says_what_a_press_does_in_each_state() {
    let squash = MergeFacts {
        default_method: MergeMethod::Squash,
        ..open()
    };
    let cases: Vec<(&str, MergeFacts, &str, bool)> = vec![
        ("ready", squash.clone(), "Squash and merge", true),
        ("ready, merge commit", open(), "Merge", true),
        (
            "ready, rebase",
            MergeFacts {
                default_method: MergeMethod::Rebase,
                ..open()
            },
            "Rebase and merge",
            true,
        ),
        (
            "a draft",
            MergeFacts {
                draft: true,
                ..squash.clone()
            },
            "Ready for review",
            true,
        ),
        (
            "behind",
            MergeFacts {
                behind: Some(vec![UpdateWay::Merge]),
                ..squash.clone()
            },
            "Update branch",
            true,
        ),
        (
            "conflicts",
            MergeFacts {
                conflicts: vec!["a".into()],
                ..squash.clone()
            },
            "Squash and merge",
            false,
        ),
        (
            "checks running",
            MergeFacts {
                checks_running: 2,
                ..squash.clone()
            },
            "Squash and merge",
            false,
        ),
        (
            "review missing",
            MergeFacts {
                review: ReviewNeed::Missing,
                ..squash.clone()
            },
            "Squash and merge",
            false,
        ),
        (
            "a queue, ready",
            MergeFacts {
                queue: Some(Queue::default()),
                ..squash.clone()
            },
            "Add to merge queue",
            true,
        ),
        (
            "in the queue",
            MergeFacts {
                queue: Some(Queue {
                    queued: true,
                    position: Some(2),
                }),
                ..squash.clone()
            },
            "Remove from the queue",
            true,
        ),
        (
            "merge when ready on",
            MergeFacts {
                auto_merge: Some(true),
                checks_running: 1,
                ..squash.clone()
            },
            "Cancel merge when ready",
            true,
        ),
        (
            "an admin past a failing check",
            MergeFacts {
                checks_failing: 1,
                rights: Rights::Bypass,
                ..squash.clone()
            },
            "Bypass rules and merge",
            true,
        ),
        (
            "an admin cannot pass a conflict",
            MergeFacts {
                conflicts: vec!["a".into()],
                rights: Rights::Bypass,
                ..squash.clone()
            },
            "Squash and merge",
            false,
        ),
        (
            "no rights",
            MergeFacts {
                rights: Rights::Cannot,
                ..squash.clone()
            },
            "Squash and merge",
            false,
        ),
    ];
    for (what, facts, want, enabled) in cases {
        let state = button(&facts, &choice(&facts)).unwrap();
        assert_eq!(state.label, want, "{what}");
        assert_eq!(state.action.is_some(), enabled, "{what}");
        assert_eq!(
            state.reason.is_some(),
            !enabled,
            "{what}: a disabled button says why"
        );
    }
    assert_eq!(
        label(&MergeFacts {
            conflicts: vec!["a".into()],
            ..open()
        }),
        "Merge"
    );
}

#[test]
fn a_disabled_button_gives_the_first_reason() {
    let facts = MergeFacts {
        checks_running: 1,
        review: ReviewNeed::Missing,
        ..open()
    };
    let state = button(&facts, &choice(&facts)).unwrap();
    assert_eq!(
        state.reason.as_deref(),
        Some("1 required check still running")
    );
}

#[test]
fn merge_when_ready_waits_out_checks_and_reviews_only() {
    let waiting = MergeFacts {
        checks_running: 1,
        review: ReviewNeed::Missing,
        auto_merge: Some(false),
        ..open()
    };
    let auto = Choice {
        auto: true,
        ..choice(&waiting)
    };
    assert_eq!(
        button(&waiting, &auto).unwrap().action,
        Some(Action::MergeWhenReady(MergeMethod::Merge))
    );
    let conflict = MergeFacts {
        conflicts: vec!["a".into()],
        auto_merge: Some(false),
        ..open()
    };
    let auto = Choice {
        auto: true,
        ..choice(&conflict)
    };
    assert_eq!(
        button(&conflict, &auto).unwrap().action,
        None,
        "a conflict does not pass by itself"
    );
    let not_allowed = MergeFacts {
        checks_running: 1,
        ..open()
    };
    let auto = Choice {
        auto: true,
        ..choice(&not_allowed)
    };
    assert_eq!(
        button(&not_allowed, &auto).unwrap().action,
        None,
        "the repository does not allow it"
    );
}

#[test]
fn a_behind_branch_updates_the_default_way() {
    let facts = MergeFacts {
        behind: Some(vec![UpdateWay::Rebase, UpdateWay::Merge]),
        ..open()
    };
    assert_eq!(
        button(&facts, &choice(&facts)).unwrap().action,
        Some(Action::UpdateBranch(UpdateWay::Rebase))
    );
}

#[test]
fn a_merged_or_closed_pull_request_has_no_button() {
    for state in [PullState::Merged, PullState::Closed] {
        let facts = MergeFacts { state, ..open() };
        assert_eq!(button(&facts, &choice(&facts)), None);
    }
}

#[test]
fn the_branch_setting_starts_as_the_repository_sets_it() {
    assert!(
        first_choice(
            &MergeFacts {
                delete_branch: true,
                ..open()
            },
            None
        )
        .delete_branch
    );
    assert!(!first_choice(&open(), None).delete_branch);
}
