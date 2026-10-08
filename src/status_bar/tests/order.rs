use std::{cell::RefCell, rc::Rc};

use gpui_kit::{
    IntoElement, Modifiers, ParentElement, Render, Styled, TestAppContext, VisualTestContext,
    Window, div, px, size,
};

use crate::{
    menu::Lead,
    status_bar::{Gauge, ProviderGauge, StatusBar, SystemLoad, Work},
    theme::{Appearance, set_appearance},
};

type Log = Rc<RefCell<Vec<&'static str>>>;

struct Bar {
    width: f32,
    lead: Option<f32>,
    tail: Option<f32>,
    version: Option<&'static str>,
    log: Log,
}

impl Render for Bar {
    fn render(&mut self, _: &mut Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        let load = SystemLoad {
            cpu: 0.3,
            cpu_history: vec![0.3; 4],
            memory_used: 8 << 30,
            memory_total: 16 << 30,
            app_memory: None,
        };
        let claude = ProviderGauge::new("Claude", Lead::Monogram)
            .gauge(Gauge::new("5h", 0.43, Some(600)))
            .gauge(Gauge::new("7d", 0.24, Some(86_400)));
        let (on_version, on_usage) = (self.log.clone(), self.log.clone());
        div().w(px(self.width)).child(
            StatusBar::new("bar")
                .load(Some(load))
                .work(Work::new(1, 1))
                .providers(vec![claude])
                .version(self.version.map(Into::into))
                .on_version(move |_, _| on_version.borrow_mut().push("version"))
                .on_usage(move |_, _| on_usage.borrow_mut().push("usage"))
                .columns(self.lead, self.tail),
        )
    }
}

fn open<'a>(
    width: f32,
    lead: Option<f32>,
    tail: Option<f32>,
    version: Option<&'static str>,
    cx: &'a mut TestAppContext,
) -> (&'a mut VisualTestContext, Log) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
    });
    let log = Log::default();
    let host = Bar {
        width,
        lead,
        tail,
        version,
        log: log.clone(),
    };
    let (_host, cx) = cx.add_window_view(move |_, _| host);
    cx.simulate_resize(size(px(width), px(200.)));
    cx.run_until_parked();
    (cx, log)
}

fn inside(
    cx: &mut VisualTestContext,
    part: &'static str,
    card: gpui_kit::Bounds<gpui_kit::Pixels>,
) {
    let b = cx
        .debug_bounds(part)
        .unwrap_or_else(|| panic!("{part} drawn"));
    assert!(
        b.left() >= card.left() && b.right() <= card.right(),
        "{part} stands in its card"
    );
}

/// The version stands under the sidebar, the work and the usage between, the processor and the memory under the right pane.
#[gpui_kit::test]
fn the_cards_run_version_then_work_and_usage_then_the_machine(cx: &mut TestAppContext) {
    let (cx, _) = open(1000., Some(240.), Some(340.), Some("0.4.2"), cx);
    let (first, middle, last) = (
        cx.debug_bounds("status-card-version").unwrap(),
        cx.debug_bounds("status-card-main").unwrap(),
        cx.debug_bounds("status-card-load").unwrap(),
    );
    assert!(
        (f32::from(first.size.width) - 240.).abs() < 0.6,
        "the version card is the sidebar width"
    );
    assert!(
        (f32::from(last.size.width) - 340.).abs() < 0.6,
        "the load card is the right pane width"
    );
    assert!(
        first.right() <= middle.left() && middle.right() <= last.left(),
        "in that order"
    );
    inside(cx, "status-version", first);
    inside(cx, "status-work", middle);
    inside(cx, "status-usage", middle);
    inside(cx, "status-cpu", last);
    inside(cx, "status-memory", last);
    assert!(
        cx.debug_bounds("status-card-providers").is_none()
            && cx.debug_bounds("status-card-cpu").is_none()
    );
}

#[gpui_kit::test]
fn the_version_label_is_shown_and_none_draws_no_label(cx: &mut TestAppContext) {
    let (cx, _) = open(1000., Some(240.), Some(260.), Some("0.4.2"), cx);
    assert!(cx.debug_bounds("status-version").is_some());
}

#[gpui_kit::test]
fn no_version_draws_no_label_but_the_card_keeps_the_column(cx: &mut TestAppContext) {
    let (cx, _) = open(1000., Some(240.), Some(260.), None, cx);
    assert!(
        cx.debug_bounds("status-version").is_none(),
        "no version, no label"
    );
    assert!(
        cx.debug_bounds("status-card-version").is_some(),
        "the card keeps the column"
    );
}

#[gpui_kit::test]
fn a_press_on_the_version_and_on_the_usage_fires_each_callback(cx: &mut TestAppContext) {
    let (cx, log) = open(1000., Some(240.), Some(260.), Some("0.4.2"), cx);
    let version = cx.debug_bounds("status-version").unwrap();
    cx.simulate_click(version.center(), Modifiers::default());
    assert_eq!(*log.borrow(), vec!["version"]);
    let usage = cx.debug_bounds("status-usage").unwrap();
    cx.simulate_click(usage.center(), Modifiers::default());
    assert_eq!(*log.borrow(), vec!["version", "usage"]);
    // A press on one provider chip is a press on the usage.
    let chip = cx.debug_bounds("status-provider-Claude").unwrap();
    cx.simulate_click(chip.center(), Modifiers::default());
    assert_eq!(*log.borrow(), vec!["version", "usage", "usage"]);
}

#[gpui_kit::test]
fn a_narrow_bar_cuts_text_and_the_cards_never_overlap(cx: &mut TestAppContext) {
    let (cx, _) = open(
        360.,
        Some(110.),
        Some(120.),
        Some("0.4.2-beta.17+build.99"),
        cx,
    );
    let (first, middle, last) = (
        cx.debug_bounds("status-card-version").unwrap(),
        cx.debug_bounds("status-card-main").unwrap(),
        cx.debug_bounds("status-card-load").unwrap(),
    );
    assert!(first.right() <= middle.left() && middle.right() <= last.left());
    assert!(last.right() <= cx.debug_bounds("status-bar").unwrap().right());
    let version = cx.debug_bounds("status-version").unwrap();
    assert!(
        version.right() <= first.right(),
        "the version is cut, not spilled"
    );
}

#[gpui_kit::test]
fn with_no_lead_the_version_stands_first_in_the_middle_card(cx: &mut TestAppContext) {
    let (cx, _) = open(1000., None, Some(260.), Some("0.4.2"), cx);
    assert!(cx.debug_bounds("status-card-version").is_none());
    let main = cx.debug_bounds("status-card-main").unwrap();
    let version = cx.debug_bounds("status-version").unwrap();
    let work = cx.debug_bounds("status-work").unwrap();
    assert!(
        version.left() >= main.left() && version.right() <= work.left(),
        "the version leads the middle card"
    );
}

#[gpui_kit::test]
fn with_no_tail_the_machine_stands_last_in_the_middle_card(cx: &mut TestAppContext) {
    let (cx, _) = open(1000., Some(240.), None, Some("0.4.2"), cx);
    assert!(cx.debug_bounds("status-card-load").is_none());
    let main = cx.debug_bounds("status-card-main").unwrap();
    let usage = cx.debug_bounds("status-usage").unwrap();
    let cpu = cx.debug_bounds("status-cpu").unwrap();
    assert!(
        usage.right() <= cpu.left() && cpu.right() <= main.right(),
        "the machine follows the usage, inside the card"
    );
}
