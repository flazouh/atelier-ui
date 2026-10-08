use super::{MENU, MENU_ORIGIN, MenuChoice, connection_words};
use crate::sidebar_model::Connection;

#[test]
fn a_connected_project_says_nothing_and_the_others_say_where_they_are() {
    assert_eq!(connection_words(Connection::Connected), None);
    assert_eq!(
        connection_words(Connection::Connecting),
        Some("Connecting…")
    );
    assert_eq!(
        connection_words(Connection::Reconnecting),
        Some("Reconnecting…")
    );
    assert_eq!(connection_words(Connection::Offline), Some("Offline"));
}

#[test]
fn the_menu_offers_close_reveal_and_copy_in_that_order_and_each_choice_has_its_words() {
    assert_eq!(
        MENU,
        [
            "Pull requests",
            "Tasks",
            "Worktrees",
            "Choose an icon…",
            "Close project",
            "Files",
            "Copy path"
        ]
    );
    let words: Vec<_> = MenuChoice::ALL.iter().map(|c| c.words()).collect();
    assert_eq!(words, MENU);
}

/// With no forge remote, "Pull requests" stays in the menu but cannot be chosen, and says why.
#[test]
fn pull_requests_are_off_with_their_reason_when_the_project_has_no_forge_remote() {
    let why = "No GitHub remote for this project";
    assert_eq!(
        super::unavailable(MenuChoice::PullRequests, Some(why)),
        Some(why)
    );
    assert_eq!(super::unavailable(MenuChoice::PullRequests, None), None);
    assert_eq!(
        super::unavailable(MenuChoice::Close, Some(why)),
        None,
        "the other entries stay"
    );
}

/// The menu grows out from under its `⋯` button, which is at the panel's right.
#[test]
fn the_menu_unfolds_from_the_corner_under_its_button() {
    // The origin keeps 12px from the panel's edges, so the corner is 12px in from the top.
    assert_eq!(MENU_ORIGIN.point((250., 130.)), (238., 12.));
}

mod narrow {
    use gpui_kit::{
        Context, IntoElement, ParentElement, Render, Styled, TestAppContext, Window, div,
    };

    use crate::{
        project_section::ProjectSection,
        scale::px,
        sidebar_model::{Badge, Connection, Location, ProjectData},
        theme::{Appearance, set_appearance},
    };

    struct Host {
        name: &'static str,
        host: Option<&'static str>,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let location = self
                .host
                .map_or(Location::Local, |host| Location::Ssh { host: host.into() });
            let project = ProjectData {
                id: "p".into(),
                name: self.name.into(),
                location,
                connection: Connection::Connected,
                sessions: Vec::new(),
                pulls_unavailable: None,
                badge: Badge {
                    label: "F".into(),
                    color: 1,
                    icon: None,
                },
            };
            div().w(px(240.)).child(
                ProjectSection::new("project", project)
                    .expanded(true)
                    .on_new_session(|_, _| {})
                    .on_menu(|_, _| {})
                    .on_toggle(|_, _| {}),
            )
        }
    }

    /// A long name on a remote host leaves the row's buttons in the row: the name gives way, not the buttons.
    #[gpui_kit::test]
    fn a_long_name_and_a_host_leave_the_buttons_in_the_row(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            set_appearance(Appearance::Dark, cx);
            cx.set_reduce_motion(true);
        });
        let host = Host {
            name: "fluentai-pro-with-a-name-that-is-long-indeed",
            host: Some("hp-agent-on-the-other-side"),
        };
        let (_, cx) = cx.add_window_view(move |_, _| host);
        cx.run_until_parked();
        let more = cx.debug_bounds("project-more").expect("the ⋯ is drawn");
        let new = cx
            .debug_bounds("project-new-session")
            .expect("the + is drawn");
        let row = px(240.);
        assert!(
            more.right() <= row,
            "the ⋯ is inside the row: {more:?} in {row:?}"
        );
        assert!(
            new.right() <= more.left(),
            "the + is before it: {new:?} {more:?}"
        );
    }
}
