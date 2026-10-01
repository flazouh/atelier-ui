use super::*;

/// "Auto" puts the badge on a row only where no heading says the project: the priority list.
#[test]
fn the_badge_shows_by_mode_unless_the_reader_decided() {
    let by = |mode, project_badge| SidebarLayout { mode, project_badge, ..Default::default() };
    assert!(!by(ListMode::Projects, BadgeShow::Auto).badge_on_rows());
    assert!(by(ListMode::Priority, BadgeShow::Auto).badge_on_rows());
    assert!(by(ListMode::Projects, BadgeShow::Always).badge_on_rows());
    assert!(!by(ListMode::Priority, BadgeShow::Never).badge_on_rows());
}

#[test]
fn rows_are_flush_only_in_the_priority_list() {
    assert!(!SidebarLayout::default().rows_flush());
    assert!(SidebarLayout { mode: ListMode::Priority, ..Default::default() }.rows_flush());
}

#[test]
fn a_badge_choice_is_kept_by_its_key_and_a_stranger_is_auto() {
    for badge in BadgeShow::ALL {
        assert_eq!(BadgeShow::from_key(Some(badge.key())), badge);
    }
    assert_eq!(BadgeShow::from_key(Some("whatever")), BadgeShow::Auto);
    assert_eq!(BadgeShow::from_key(None), BadgeShow::Auto);
}

/// The defaults are the numbers the sidebar always had.
#[test]
fn the_defaults_are_what_the_sidebar_had() {
    let d = SidebarLayout::default();
    assert_eq!((d.fold_after, d.earlier_shown, d.show_time, d.show_agent_icon), (5, 8, true, true));
    assert!(FOLD_CHOICES.contains(&d.fold_after) && EARLIER_CHOICES.contains(&d.earlier_shown));
}

/// The Settings page changes the look and leaves the head's two choices alone.
#[test]
fn the_look_of_another_layout_keeps_the_heads_choices() {
    let head = SidebarLayout { mode: ListMode::Priority, filter: SessionFilter::Archived, ..Default::default() };
    let settings = SidebarLayout { project_badge: BadgeShow::Never, show_time: false, fold_after: 12, ..Default::default() };
    let merged = head.with_look_of(&settings);
    assert_eq!((merged.mode, merged.filter), (ListMode::Priority, SessionFilter::Archived));
    assert_eq!((merged.project_badge, merged.show_time, merged.fold_after), (BadgeShow::Never, false, 12));
}
