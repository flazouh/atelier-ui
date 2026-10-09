use gpui_kit::{
    Bounds, Context, IntoElement, ParentElement, Pixels, Render, Styled, TestAppContext, Window,
    div, px, size,
};

use crate::{
    menu::Lead,
    status_bar::{
        Gauge, ProviderGauge, StatusBar, StatusCard, StatusRow, SystemLoad, Work, load_parts,
        usage_part, version_part, work_part,
    },
    theme::{Appearance, set_appearance},
};

/// A bar drawn with `StatusBar`, or the same bar put together from cards and parts.
struct Bar {
    cards: bool,
}

const PARTS: [&str; 8] = [
    "status-bar",
    "status-card-version",
    "status-card-main",
    "status-card-load",
    "status-version",
    "status-usage",
    "status-work",
    "status-cpu",
];

fn load() -> SystemLoad {
    SystemLoad {
        cpu: 0.3,
        cpu_history: vec![0.1, 0.5, 0.3],
        memory_used: 8 << 30,
        memory_total: 32 << 30,
        app_memory: None,
    }
}

fn providers() -> Vec<ProviderGauge> {
    vec![ProviderGauge::new("Claude", Lead::Monogram).gauge(Gauge::new("5h", 0.4, None))]
}

impl Render for Bar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let press = std::rc::Rc::new(|_: &mut Window, _: &mut gpui_kit::App| {});
        let bar = if self.cards {
            let id = gpui_kit::ElementId::from("bar");
            let version = version_part(&id, "1.2.3".into(), Some(press.clone()), window, cx);
            let work = work_part(Work::new(1, 2), cx);
            let usage = usage_part(&id, &providers(), Some(press), window, cx);
            let machine = load_parts(&id, &load(), cx);
            StatusRow::new("bar")
                .children([
                    StatusCard::new("status-card-version")
                        .width(Some(120.))
                        .children([version]),
                    StatusCard::new("status-card-main").children(work.into_iter().chain(usage)),
                    StatusCard::new("status-card-load")
                        .width(Some(260.))
                        .children(machine),
                ])
                .into_any_element()
        } else {
            StatusBar::new("bar")
                .load(Some(load()))
                .work(Work::new(1, 2))
                .providers(providers())
                .version(Some("1.2.3".into()))
                .on_version(|_, _| {})
                .on_usage(|_, _| {})
                .columns(Some(120.), Some(260.))
                .into_any_element()
        };
        div().w(px(900.)).child(bar)
    }
}

fn layout(cards: bool, cx: &mut TestAppContext) -> Vec<Option<Bounds<Pixels>>> {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
    });
    let (_, cx) = cx.add_window_view(move |_, _| Bar { cards });
    cx.simulate_resize(size(px(900.), px(100.)));
    cx.run_until_parked();
    PARTS.iter().map(|part| cx.debug_bounds(part)).collect()
}

/// A bar an app puts together from cards and parts stands exactly where the bar that lays itself out stands, part for part.
#[gpui_kit::test]
fn cards_and_parts_make_the_same_bar_as_the_status_bar(cx: &mut TestAppContext) {
    let own = layout(false, cx);
    let composed = layout(true, cx);
    assert!(
        own.iter().all(Option::is_some),
        "the bar draws every part: {own:?}"
    );
    assert_eq!(own, composed);
}
