use gpui_kit::{IntoElement, ParentElement, Render, Styled, TestAppContext, Window, div};

use crate::{
    menu::Lead,
    status_bar::{Gauge, GaugeState, ProviderGauge, StatusBar, SystemLoad, Work},
    theme::{Appearance, set_appearance},
};

struct Host {
    load: Option<SystemLoad>,
    work: Work,
    providers: Vec<ProviderGauge>,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        div().w(gpui_kit::px(900.)).child(
            StatusBar::new("bar")
                .load(self.load.clone())
                .work(self.work)
                .providers(self.providers.clone()),
        )
    }
}

fn shown(host: Host, cx: &mut TestAppContext) -> &mut gpui_kit::VisualTestContext {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(Appearance::Dark, cx);
        cx.set_reduce_motion(true);
    });
    let (_, cx) = cx.add_window_view(move |_, _| host);
    cx.run_until_parked();
    cx
}

fn load() -> SystemLoad {
    SystemLoad {
        cpu: 0.3,
        cpu_history: vec![0.1, 0.5, 0.3],
        memory_used: 8 << 30,
        memory_total: 32 << 30,
        app_memory: None,
    }
}

#[gpui_kit::test]
fn the_bar_shows_the_machine_the_work_and_each_provider(cx: &mut TestAppContext) {
    let providers = vec![
        ProviderGauge::new("Claude", Lead::Monogram).gauge(Gauge::new("5h", 0.4, None)),
        ProviderGauge::new("Codex", Lead::Monogram).gauge(Gauge::new("30d", 0.25, None)),
    ];
    let cx = shown(
        Host {
            load: Some(load()),
            work: Work::new(2, 1),
            providers,
        },
        cx,
    );
    for part in [
        "status-bar",
        "status-cpu",
        "status-memory",
        "status-work",
        "status-provider-Claude",
        "status-provider-Codex",
    ] {
        assert!(cx.debug_bounds(part).is_some(), "{part}");
    }
}

#[gpui_kit::test]
fn a_bar_with_nothing_to_tell_is_empty_but_for_itself(cx: &mut TestAppContext) {
    let cx = shown(
        Host {
            load: None,
            work: Work::default(),
            providers: Vec::new(),
        },
        cx,
    );
    assert!(cx.debug_bounds("status-bar").is_some());
    for part in ["status-cpu", "status-memory", "status-work"] {
        assert!(cx.debug_bounds(part).is_none(), "{part}");
    }
}

#[gpui_kit::test]
fn a_provider_with_no_numbers_still_shows_with_a_dash(cx: &mut TestAppContext) {
    let provider = ProviderGauge::new("Codex", Lead::Monogram).state(GaugeState::Unavailable("not signed in".into()));
    let cx = shown(
        Host {
            load: None,
            work: Work::default(),
            providers: vec![provider],
        },
        cx,
    );
    assert!(cx.debug_bounds("status-provider-Codex").is_some());
}

#[gpui_kit::test]
fn the_bar_is_as_tall_as_it_says(cx: &mut TestAppContext) {
    let cx = shown(
        Host {
            load: Some(load()),
            work: Work::default(),
            providers: Vec::new(),
        },
        cx,
    );
    let bar = cx.debug_bounds("status-bar").expect("drawn");
    assert_eq!(
        f32::from(bar.size.height),
        crate::status_bar::HEIGHT * crate::scale::zoom()
    );
}
