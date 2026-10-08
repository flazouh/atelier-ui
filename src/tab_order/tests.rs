use gpui_kit::SharedString;

use super::{TabOrder, grouped, visual_order};

fn s(text: &str) -> SharedString {
    text.to_string().into()
}

fn tabs(ids: &[&str]) -> TabOrder {
    let mut tabs = TabOrder::new();
    ids.iter().for_each(|id| tabs.open(s(id)));
    tabs
}

fn order(tabs: &TabOrder) -> Vec<String> {
    tabs.order().iter().map(|t| t.to_string()).collect()
}

#[test]
fn opening_a_tab_adds_it_at_the_end_and_makes_it_active_and_a_second_open_only_activates() {
    let mut t = tabs(&["a", "b", "c"]);
    assert_eq!(
        (order(&t), t.active().map(|a| a.to_string())),
        (
            vec!["a".to_string(), "b".into(), "c".into()],
            Some("c".into())
        )
    );
    t.open(s("a"));
    assert_eq!(
        (order(&t), t.active().map(|a| a.to_string())),
        (
            vec!["a".to_string(), "b".into(), "c".into()],
            Some("a".into())
        )
    );
}

#[test]
fn activating_an_unknown_tab_changes_nothing() {
    let mut t = tabs(&["a", "b"]);
    t.activate("zzz");
    assert_eq!(t.active().map(|a| a.to_string()), Some("b".into()));
    t.activate("a");
    assert_eq!(t.active().map(|a| a.to_string()), Some("a".into()));
}

#[test]
fn closing_the_active_tab_activates_its_right_neighbour_else_its_left() {
    let mut t = tabs(&["a", "b", "c", "d"]);
    t.activate("b");
    assert_eq!(t.close("b").map(|a| a.to_string()), Some("c".into()));
    assert_eq!(order(&t), ["a", "c", "d"]);
    t.activate("d");
    assert_eq!(
        t.close("d").map(|a| a.to_string()),
        Some("c".into()),
        "the last has no right neighbour"
    );
    assert_eq!(t.close("c").map(|a| a.to_string()), Some("a".into()));
    assert_eq!(t.close("a"), None, "the last tab closes to nothing");
    assert!(t.order().is_empty());
}

#[test]
fn closing_a_tab_that_is_not_active_leaves_the_active_one_alone() {
    let mut t = tabs(&["a", "b", "c"]);
    t.activate("a");
    assert_eq!(t.close("c").map(|a| a.to_string()), Some("a".into()));
    assert_eq!(
        t.close("zzz").map(|a| a.to_string()),
        Some("a".into()),
        "an unknown tab closes nothing"
    );
    assert_eq!(order(&t), ["a", "b"]);
}

#[test]
fn a_tab_dragged_onto_another_sits_just_before_it_and_a_drop_on_nothing_sends_it_to_the_end() {
    let mut t = tabs(&["a", "b", "c", "d"]);
    t.move_before("d", Some("b"));
    assert_eq!(order(&t), ["a", "d", "b", "c"]);
    t.move_before("a", Some("c"));
    assert_eq!(order(&t), ["d", "b", "a", "c"]);
    t.move_before("d", None);
    assert_eq!(order(&t), ["b", "a", "c", "d"]);
    t.move_before("b", Some("b"));
    assert_eq!(order(&t), ["b", "a", "c", "d"], "onto itself is no move");
    t.move_before("zzz", Some("a"));
    t.move_before("a", Some("zzz"));
    assert_eq!(order(&t), ["b", "a", "c", "d"], "unknown tabs move nothing");
}

#[test]
fn moving_a_tab_keeps_the_active_one_active() {
    let mut t = tabs(&["a", "b", "c"]);
    t.activate("b");
    t.move_before("c", Some("a"));
    assert_eq!(t.active().map(|a| a.to_string()), Some("b".into()));
}

#[test]
fn control_tab_cycles_forward_and_back_round_the_ends() {
    let mut t = tabs(&["a", "b", "c"]);
    let seq = t.order().to_vec();
    t.activate("c");
    assert_eq!(t.cycle(&seq, true), Some(s("a")));
    t.activate("a");
    assert_eq!(t.cycle(&seq, false), Some(s("c")));
    assert_eq!(t.cycle(&seq, true), Some(s("b")));
    assert_eq!(TabOrder::new().cycle(&[], true), None);
}

#[test]
fn cycling_with_no_active_tab_starts_at_the_first_or_the_last() {
    let t = TabOrder::new();
    let seq = vec![s("a"), s("b")];
    assert_eq!(t.cycle(&seq, true), Some(s("a")));
    assert_eq!(t.cycle(&seq, false), Some(s("b")));
}

fn project_of(id: &SharedString) -> SharedString {
    s(id.split(':').next().unwrap())
}

#[test]
fn grouped_tabs_gather_by_project_in_the_sidebars_order_and_keep_their_own_order() {
    let open = vec![s("web:1"), s("api:1"), s("web:2"), s("docs:1"), s("api:2")];
    let groups = grouped(&open, project_of, &[s("api"), s("web")]);
    let shape: Vec<_> = groups
        .iter()
        .map(|g| {
            (
                g.project.to_string(),
                g.tabs.iter().map(|t| t.to_string()).collect::<Vec<_>>(),
            )
        })
        .collect();
    assert_eq!(
        shape,
        [
            (
                "api".to_string(),
                vec!["api:1".to_string(), "api:2".to_string()]
            ),
            (
                "web".to_string(),
                vec!["web:1".to_string(), "web:2".to_string()]
            ),
            ("docs".to_string(), vec!["docs:1".to_string()]),
        ]
    );
    let visual: Vec<_> = visual_order(&groups)
        .iter()
        .map(|t| t.to_string())
        .collect();
    assert_eq!(visual, ["api:1", "api:2", "web:1", "web:2", "docs:1"]);
}

#[test]
fn control_tab_follows_the_grouped_order_when_grouping_is_on() {
    let mut t = tabs(&["web:1", "api:1", "web:2"]);
    let groups = grouped(t.order(), project_of, &[s("api"), s("web")]);
    let visual = visual_order(&groups);
    t.activate("api:1");
    assert_eq!(
        t.cycle(&visual, true),
        Some(s("web:1")),
        "api:1 is followed by the first web tab, not by web:2"
    );
}

#[test]
fn no_tabs_no_groups() {
    assert!(grouped(&[], project_of, &[]).is_empty());
    assert!(visual_order(&[]).is_empty());
}
