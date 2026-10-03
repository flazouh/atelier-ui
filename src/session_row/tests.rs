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
            let data = SessionData { archived: false, in_panel: false, provider: None,
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

mod archive {
    use std::{cell::Cell, rc::Rc};

    use gpui_kit::{Context, IntoElement, ParentElement, Render, Styled, TestAppContext, Window, div, px, size};

    use crate::{
        agent_look::AgentLook,
        session_row::SessionRow,
        session_status::SessionStatus,
        sidebar_model::SessionData,
        theme::{Appearance, set_appearance},
    };

    struct Host {
        archived: Rc<Cell<u32>>,
        opened: Rc<Cell<u32>>,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let data = SessionData {
                archived: false,
                in_panel: false,
                provider: None,
                id: "s1".into(),
                title: "Add a subtract function".into(),
                look: AgentLook::neutral(&crate::theme::Theme::light()),
                status: SessionStatus::Idle,
                active_at: 0,
            };
            let (archived, opened) = (self.archived.clone(), self.opened.clone());
            div().w(px(300.)).child(
                SessionRow::new("row", data, 10)
                    .on_open(move |_, _| opened.set(opened.get() + 1))
                    .archive(false, move |_, _| archived.set(archived.get() + 1))
                    .more(false, |_, _| {}, None),
            )
        }
    }

    #[gpui_kit::test]
    fn a_press_on_archive_archives_the_session_and_does_not_open_it(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            set_appearance(Appearance::Light, cx);
            cx.set_reduce_motion(true);
        });
        let (archived, opened) = (Rc::new(Cell::new(0)), Rc::new(Cell::new(0)));
        let (_host, cx) = cx.add_window_view(|_, _| Host { archived: archived.clone(), opened: opened.clone() });
        cx.simulate_resize(size(px(400.), px(100.)));
        cx.run_until_parked();
        let row = cx.debug_bounds("row-title:Add a subtract function").expect("the row is drawn");
        cx.simulate_mouse_move(row.center(), None, Default::default());
        cx.run_until_parked();
        let archive = cx
            .debug_bounds("session-archive")
            .expect("the archive button is drawn");
        let more = cx
            .debug_bounds("session-more")
            .expect("the more button is drawn");
        assert!(
            archive.right() <= more.left(),
            "archive ({archive:?}) sits left of more ({more:?})"
        );
        let time = cx.debug_bounds("row-time").expect("the time stays while the pointer is on the row");
        assert!(more.right() <= time.left(), "the buttons stand left of the time, as in Cursor: {more:?} {time:?}");
        let title = cx.debug_bounds("row-title:Add a subtract function").unwrap();
        assert!(title.right() <= archive.left() + px(0.5), "and the title gives them room: {title:?} {archive:?}");
        cx.simulate_click(archive.center(), Default::default());
        cx.run_until_parked();
        assert_eq!((archived.get(), opened.get()), (1, 0));
    }
}

mod provider {
    use gpui_kit::{Context, IntoElement, ParentElement, Render, Styled, TestAppContext, Window, div, px, size};

    use crate::{
        agent_look::AgentLook,
        session_row::SessionRow,
        session_status::SessionStatus,
        sidebar_model::SessionData,
        theme::{Appearance, set_appearance},
    };

    const OPENROUTER: &str = "OpenRouter";

    struct Host {
        provider: Option<&'static str>,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let data = SessionData {
                archived: false,
                in_panel: false,
                provider: self.provider.map(Into::into),
                id: "s1".into(),
                title: "Try a model".into(),
                look: AgentLook::neutral(&crate::theme::Theme::light()),
                status: SessionStatus::Idle,
                active_at: 0,
            };
            div().w(px(300.)).child(SessionRow::new("row", data, 10))
        }
    }

    fn draw<'a>(provider: Option<&'static str>, cx: &'a mut TestAppContext) -> &'a mut gpui_kit::VisualTestContext {
        cx.update(|cx| {
            gpui_kit::init(cx);
            set_appearance(Appearance::Light, cx);
            cx.set_reduce_motion(true);
        });
        let (_host, cx) = cx.add_window_view(move |_, _| Host { provider });
        cx.simulate_resize(size(px(400.), px(100.)));
        cx.run_until_parked();
        cx
    }

    #[gpui_kit::test]
    fn a_provider_other_than_the_default_is_named_on_the_row(cx: &mut TestAppContext) {
        let cx = draw(Some(OPENROUTER), cx);

        assert!(cx.debug_bounds("row-provider").is_some());
    }

    #[gpui_kit::test]
    fn the_default_provider_is_not_named(cx: &mut TestAppContext) {
        let cx = draw(None, cx);

        assert!(cx.debug_bounds("row-provider").is_none());
    }
}
