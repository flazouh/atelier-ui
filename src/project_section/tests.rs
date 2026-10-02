use super::{MENU, MENU_ORIGIN, MenuChoice, connection_words};
use crate::sidebar_model::Connection;

#[test]
fn a_connected_project_says_nothing_and_the_others_say_where_they_are() {
    assert_eq!(connection_words(Connection::Connected), None);
    assert_eq!(connection_words(Connection::Connecting), Some("Connecting…"));
    assert_eq!(connection_words(Connection::Reconnecting), Some("Reconnecting…"));
    assert_eq!(connection_words(Connection::Offline), Some("Offline"));
}

#[test]
fn the_menu_offers_close_reveal_and_copy_in_that_order_and_each_choice_has_its_words() {
    assert_eq!(MENU, ["Pull requests", "Tasks", "Worktrees", "Choose an icon…", "Close project", "Files", "Copy path"]);
    let words: Vec<_> = MenuChoice::ALL.iter().map(|c| c.words()).collect();
    assert_eq!(words, MENU);
}

/// With no forge remote, "Pull requests" stays in the menu but cannot be chosen, and says why.
#[test]
fn pull_requests_are_off_with_their_reason_when_the_project_has_no_forge_remote() {
    let why = "No GitHub remote for this project";
    assert_eq!(super::unavailable(MenuChoice::PullRequests, Some(why)), Some(why));
    assert_eq!(super::unavailable(MenuChoice::PullRequests, None), None);
    assert_eq!(super::unavailable(MenuChoice::Close, Some(why)), None, "the other entries stay");
}

/// The menu grows out from under its `⋯` button, which is at the panel's right.
#[test]
fn the_menu_unfolds_from_the_corner_under_its_button() {
    // The origin keeps 12px from the panel's edges, so the corner is 12px in from the top.
    assert_eq!(MENU_ORIGIN.point((250., 130.)), (238., 12.));
}
