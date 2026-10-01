use super::trailing;
use crate::{
    session_status::{Need, SessionStatus},
    theme::Theme,
};

fn theme() -> Theme {
    Theme::dark()
}

#[test]
fn a_session_that_owes_the_reader_shows_its_words_in_the_warning_tone_and_a_failure_in_the_danger_tone() {
    let theme = theme();
    let (words, tone) = trailing(&SessionStatus::NeedsYou(Need::Approval), 1000, 900, &theme);
    assert_eq!((words.as_ref(), tone), ("Needs approval", theme.warning));
    let (words, tone) = trailing(&SessionStatus::Failed("exit code 3".into()), 1000, 900, &theme);
    assert_eq!((words.as_ref(), tone), ("Stopped: exit code 3", theme.danger));
}

#[test]
fn every_other_session_shows_the_time_since_it_last_did_anything() {
    let theme = theme();
    for status in [SessionStatus::Working, SessionStatus::Finished, SessionStatus::Idle] {
        assert_eq!(trailing(&status, 1000 + 120, 1000, &theme).0.as_ref(), "2m");
    }
}

mod open {
    use gpui_kit::{Context, IntoElement, ParentElement, Render, Styled, TestAppContext, Window, div, px, size};

    use crate::{
        agent_look::AgentLook,
        session_row::SessionRow,
        session_status::SessionStatus,
        sidebar_model::SessionData,
        theme::{Appearance, set_appearance},
    };

    struct Host {
        open: bool,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let data = SessionData { archived: false, in_panel: false,
                id: "s1".into(),
                title: "Add a subtract function".into(),
                look: AgentLook::neutral(&crate::theme::Theme::light()),
                status: SessionStatus::Idle,
                active_at: 0,
            };
            div().w(px(300.)).child(SessionRow::new("row", data, 10).open(self.open))
        }
    }

    #[gpui_kit::test]
    fn the_open_session_wears_a_bar_and_the_others_do_not(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            set_appearance(Appearance::Light, cx);
            cx.set_reduce_motion(true);
        });
        let (host, cx) = cx.add_window_view(|_, _| Host { open: false });
        cx.simulate_resize(size(px(400.), px(100.)));
        cx.run_until_parked();
        assert!(cx.debug_bounds("session-row-open").is_none());
        host.update(cx, |h, cx| {
            h.open = true;
            cx.notify();
        });
        cx.run_until_parked();
        cx.run_until_parked();
        let bar = cx.debug_bounds("session-row-open").expect("the open row has a bar");
        assert_eq!((f32::from(bar.size.width), f32::from(bar.size.height)), (3., 16.));
    }
}
