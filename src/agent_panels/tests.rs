use gpui_kit::{
    AppContext, Context, Entity, Focusable, InteractiveElement, IntoElement, ParentElement, Render, Styled, TestAppContext, Window,
    component::input::{Input, InputState},
    div, px,
};

use super::AgentPanels;
use crate::{
    agent_look::AgentLook,
    panel_types::{PanelData, ProjectLabel},
    session_status::SessionStatus,
    sidebar_model::Location,
    theme::{Appearance, ActiveTheme, set_appearance},
};

/// Agent panels holding one panel whose content is an input, as a session composer is.
struct Host {
    panels: Entity<AgentPanels>,
    input: Entity<InputState>,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.panels.clone())
    }
}

/// A press on an input inside a panel leaves the focus in the input, so typing goes there and not to
/// the window bare-letter keys.
#[gpui_kit::test]
fn a_press_on_an_input_in_a_panel_keeps_the_focus_in_the_input(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
        cx.set_reduce_motion(true);
    });
    let (host, cx) = cx.add_window_view(|window, cx| {
        let input = cx.new(|cx| InputState::new(window, cx));
        let panels = cx.new(AgentPanels::new);
        let shown = input.clone();
        let look = AgentLook::neutral(cx.theme());
        let panel = PanelData {
            id: "p".into(),
            project: ProjectLabel { id: "project".into(), name: "project".into(), location: Location::Local, badge: None },
            title: "A session".into(),
            look,
            status: SessionStatus::Idle,
            content: crate::panel_types::content_from(move |_, _| {
                div().size_full().child(div().debug_selector(|| "composer".into()).h(px(40.)).child(Input::new(&shown))).into_any_element()
            }, cx),
        };
        panels.update(cx, |p, cx| p.set_panels(vec![panel], vec!["project".into()], cx));
        Host { panels, input }
    });
    cx.run_until_parked();
    let at = cx.debug_bounds("composer").expect("the panel draws its input");
    cx.simulate_click(at.center(), gpui_kit::Modifiers::default());
    cx.run_until_parked();
    let focused = cx.update(|window, cx| host.read(cx).input.read(cx).focus_handle(cx).is_focused(window));
    assert!(focused, "the input kept the focus");
}
/// When the owner's column narrows, the owner hands the panels the new width in the same frame, and
/// the panel fits it in that frame, not one frame later.
#[gpui_kit::test]
fn a_panel_fits_its_column_in_the_frame_the_column_narrows(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
        cx.set_reduce_motion(true);
    });
    let cx = cx.add_empty_window();
    let panels = cx.update(|_, cx| cx.new(AgentPanels::new));
    let look = cx.update(|_, cx| AgentLook::neutral(cx.theme()));
    let panel = PanelData {
        id: "p".into(),
        project: ProjectLabel { id: "project".into(), name: "project".into(), location: Location::Local, badge: None },
        title: "A session".into(),
        look,
        status: SessionStatus::Idle,
        content: cx.update(|_, cx| crate::panel_types::content_from(|_, _| div().size_full().debug_selector(|| "panel".into()).into_any_element(), cx)),
    };
    cx.update(|_, cx| panels.update(cx, |p, cx| p.set_panels(vec![panel], vec!["project".into()], cx)));
    let frame = |cx: &mut gpui_kit::VisualTestContext, width: f32| {
        let panels = panels.clone();
        cx.update(|_, cx| panels.update(cx, |p, cx| p.fit_to(width - 16., cx)));
        cx.draw(gpui_kit::point(px(0.), px(0.)), gpui_kit::size(px(width), px(800.)), move |_, _| div().size_full().child(panels));
    };
    frame(cx, 620.);
    frame(cx, 620.);
    for width in [320., 380., 440., 460., 520., 560., 620., 1000.] {
        frame(cx, width);
        frame(cx, width);
        let at = cx.debug_bounds("panel").expect("the panel draws");
        assert!(f32::from(at.right()) <= width - 8. + 0.5, "{width}: the panel ends at {:?}, past its column", at.right());
    }
}

/// The single view leaves the room the strip leaves: its insets at the left and at the foot, 8 at the right.
#[gpui_kit::test]
fn the_single_view_sits_at_the_strips_inset(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
        cx.set_reduce_motion(true);
    });
    let cx = cx.add_empty_window();
    let panels = cx.update(|_, cx| cx.new(AgentPanels::new));
    let look = cx.update(|_, cx| AgentLook::neutral(cx.theme()));
    let panel = PanelData {
        id: "p".into(),
        project: ProjectLabel { id: "project".into(), name: "project".into(), location: Location::Local, badge: None },
        title: "A session".into(),
        look,
        status: SessionStatus::Idle,
        content: cx.update(|_, cx| crate::panel_types::content_from(|_, _| div().size_full().debug_selector(|| "panel".into()).into_any_element(), cx)),
    };
    cx.update(|_, cx| panels.update(cx, |p, cx| {
        p.set_panels(vec![panel], vec!["project".into()], cx);
        p.set_inset_left(2., cx);
        p.set_inset_bottom(2., cx);
        p.set_layout(crate::panel_types::Layout::Single, cx);
    }));
    let frame = |cx: &mut gpui_kit::VisualTestContext, width: f32| {
        let panels = panels.clone();
        cx.update(|_, cx| panels.update(cx, |p, cx| p.fit_to(width - 16., cx)));
        cx.draw(gpui_kit::point(px(0.), px(0.)), gpui_kit::size(px(width), px(800.)), move |_, _| div().size_full().child(panels));
    };
    frame(cx, 620.);
    frame(cx, 620.);
    frame(cx, 800.);
    frame(cx, 800.);
    let at = cx.debug_bounds("panel").expect("the panel draws");
    assert!((f32::from(at.left()) - 2.).abs() < 0.5, "the panel starts at {:?}, not at the strip's inset", at.left());
    assert!(f32::from(at.right()) <= 800. - 8. + 0.5, "the panel ends at {:?}, past the right room", at.right());
    assert!((f32::from(at.bottom()) - (800. - 2.)).abs() < 0.5, "the panel ends at {:?}, not the foot inset above the foot", at.bottom());
}
struct Strip {
    panels: Entity<AgentPanels>,
}
impl Render for Strip {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.panels.clone())
    }
}

/// A column cut at the strip's edge fades there, and only there: an edge with nothing beyond it stays sharp.
#[gpui_kit::test]
fn the_strip_fades_each_edge_only_while_more_lies_beyond_it(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
        cx.set_reduce_motion(true);
    });
    let (host, cx) = cx.add_window_view(|_, cx| {
        let panels = cx.new(AgentPanels::new);
        let look = AgentLook::neutral(cx.theme());
        let list: Vec<PanelData> = (0..4)
            .map(|i| PanelData {
                id: format!("p{i}").into(),
                project: ProjectLabel { id: "project".into(), name: "project".into(), location: Location::Local, badge: None },
                title: format!("Session {i}").into(),
                look: look.clone(),
                status: SessionStatus::Idle,
                content: crate::panel_types::content_from(|_, _| div().size_full().into_any_element(), cx),
            })
            .collect();
        panels.update(cx, |p, cx| p.set_panels(list, vec!["project".into()], cx));
        Strip { panels }
    });
    cx.simulate_resize(gpui_kit::size(px(700.), px(600.)));
    cx.run_until_parked();
    let panels = host.read_with(cx, |h, _| h.panels.clone());
    let fades = |cx: &mut gpui_kit::VisualTestContext| {
        (cx.debug_bounds("strip-fade-left").is_some(), cx.debug_bounds("strip-fade-right").is_some())
    };
    panels.update(cx, |p, cx| p.scroll_to(0., cx));
    cx.run_until_parked();
    assert_eq!(fades(cx), (false, true), "at the start: nothing before it, more after");
    panels.update(cx, |p, cx| p.scroll_to(300., cx));
    cx.run_until_parked();
    assert_eq!(fades(cx), (true, true), "in the middle: both");
    panels.update(cx, |p, cx| p.scroll_to(100000., cx));
    cx.run_until_parked();
    assert_eq!(fades(cx), (true, false), "at the end: more before it, nothing after");
}
