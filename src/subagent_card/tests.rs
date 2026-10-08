use super::*;

#[test]
fn a_running_card_leads_with_its_live_tool_call() {
    assert_eq!(
        lead_text(None, Some("Read crates/ui/src/theme.rs".into())).as_deref(),
        Some("Read crates/ui/src/theme.rs")
    );
    assert_eq!(lead_text(None, None), None);
}

#[test]
fn a_finished_card_leads_with_how_long_it_ran() {
    assert_eq!(
        lead_text(Some(Some(38)), None).as_deref(),
        Some("Done in 38s")
    );
    assert_eq!(
        lead_text(Some(None), Some("Read a.rs".into())).as_deref(),
        Some("Done")
    );
}

mod mark {
    use crate::{
        agent_look::AgentLook,
        subagent_card::SubagentCard,
        theme::{ActiveTheme, Appearance, set_appearance},
    };
    use gpui_kit::{
        Context, IntoElement, ParentElement, Render, Styled, TestAppContext, Window, div, px, size,
    };
    struct Host {
        done: bool,
    }
    impl Render for Host {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let card = SubagentCard::new(
                "card",
                AgentLook::neutral(cx.theme()),
                "general-purpose",
                "Count chars",
            )
            .tint(gpui_kit::hsla(0.5, 0.8, 0.6, 1.));
            div().size_full().child(if self.done {
                card.finished(Some(3))
            } else {
                card
            })
        }
    }
    fn drawn(done: bool, cx: &mut TestAppContext) -> bool {
        cx.update(|cx| {
            gpui_kit::init(cx);
            set_appearance(Appearance::Dark, cx);
            cx.set_reduce_motion(true);
        });
        let (_host, cx) = cx.add_window_view(|_, _| Host { done });
        cx.simulate_resize(size(px(500.), px(200.)));
        cx.run_until_parked();
        cx.debug_bounds("subagent-mark").is_some()
    }
    /// A card keeps its coloured mark when it is done: the logo at rest, or for an agent with no logo a filled check.
    #[gpui_kit::test]
    fn a_card_keeps_its_mark_when_it_is_done(cx: &mut TestAppContext) {
        assert!(drawn(false, cx), "running: the mark is drawn");
        assert!(drawn(true, cx), "finished: the mark stays");
    }
}
