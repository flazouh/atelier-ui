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

mod cards {
    use gpui_kit::{IntoElement, ParentElement, Styled, TestAppContext, div, px, size};
    use crate::{
        menu::Lead,
        panel_layout::GAP,
        status_bar::{Gauge, HEIGHT, ProviderGauge, StatusBar, SystemLoad},
        theme::{Appearance, set_appearance},
    };
    struct Bar(Option<f32>, Option<f32>);
    impl gpui_kit::Render for Bar {
        fn render(&mut self, _: &mut gpui_kit::Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
            let load = SystemLoad { cpu: 0.3, cpu_history: vec![0.3; 4], memory_used: 8 << 30, memory_total: 16 << 30, app_memory: None };
            let claude = ProviderGauge::new("Claude", Lead::Monogram)
                .gauge(Gauge::new("5h", 0.43, Some(600)))
                .gauge(Gauge::new("7d", 0.24, Some(86_400)));
            let codex = ProviderGauge::new("Codex", Lead::Monogram).gauge(Gauge::new("30d", 1.0, None));
            div().w(px(1000.)).child(StatusBar::new("bar").load(Some(load)).providers(vec![claude, codex]).columns(self.0, self.1))
        }
    }
    fn open(lead: Option<f32>, tail: Option<f32>, cx: &mut TestAppContext) -> &mut gpui_kit::VisualTestContext {
        cx.update(|cx| {
            gpui_kit::init(cx);
            set_appearance(Appearance::Dark, cx);
        });
        let (_host, cx) = cx.add_window_view(|_, _| Bar(lead, tail));
        cx.simulate_resize(size(px(1000.), px(200.)));
        cx.run_until_parked();
        cx
    }
    /// The bar's cards stand under the columns above it: the machine's under the sidebar's width, the providers' under the right
    /// pane's, the one between takes the rest, each a panels' gap from the next and the bar's whole height.
    #[gpui_kit::test]
    fn the_cards_stand_under_the_columns_a_panels_gap_apart(cx: &mut TestAppContext) {
        let cx = open(Some(300.), Some(260.), cx);
        let bar = cx.debug_bounds("status-bar").unwrap();
        let (machine, main, providers) = (
            cx.debug_bounds("status-card-cpu").unwrap(),
            cx.debug_bounds("status-card-main").unwrap(),
            cx.debug_bounds("status-card-providers").unwrap(),
        );
        let near = |a: gpui_kit::Pixels, b: f32| (f32::from(a) - b).abs() < 0.6;
        assert!(near(machine.size.width, 300.), "the first card is the sidebar's width: {:?}", machine.size.width);
        assert!(near(providers.size.width, 260.), "the last card is the right pane's width: {:?}", providers.size.width);
        assert!(near(main.left() - machine.right(), GAP), "a panels' gap before the middle card");
        assert!(near(providers.left() - main.right(), GAP), "and after it");
        assert!(near(bar.right() - providers.right(), 0.), "the last card ends at the bar's edge");
        for card in [machine, main, providers] {
            assert!(near(card.size.height, HEIGHT) && near(card.top() - bar.top(), 0.), "each card takes the bar's whole height");
        }
    }
    /// The processor, with its whole history, fits the card under a sidebar of a usual width with room to spare, and the memory
    /// stands in the card beside it.
    #[gpui_kit::test]
    fn the_processor_has_room_in_the_card_under_the_sidebar(cx: &mut TestAppContext) {
        let cx = open(Some(256.), Some(260.), cx);
        let (card, cpu) = (cx.debug_bounds("status-card-cpu").unwrap(), cx.debug_bounds("status-cpu").unwrap());
        let spare = f32::from(card.size.width) - f32::from(cpu.size.width);
        assert!(spare >= 20. + 30., "the processor takes {:?} of {:?}: less than 30 px to spare", cpu.size.width, card.size.width);
        let (main, memory) = (cx.debug_bounds("status-card-main").unwrap(), cx.debug_bounds("status-memory").unwrap());
        assert!(memory.left() >= main.left() && memory.right() <= main.right(), "the memory is in the middle card");
    }
    /// With no column to stand under, the load and the providers share one card.
    #[gpui_kit::test]
    fn with_no_columns_there_is_one_card(cx: &mut TestAppContext) {
        let cx = open(None, None, cx);
        assert!(cx.debug_bounds("status-card-cpu").is_none() && cx.debug_bounds("status-card-providers").is_none());
        assert!(cx.debug_bounds("status-card-main").is_some() && cx.debug_bounds("status-cpu").is_some());
        assert!(cx.debug_bounds("status-provider-Claude").is_some());
    }
    /// A provider with a short and a long window shows both, compactly; one with a single window shows that one.
    #[gpui_kit::test]
    fn a_chip_shows_the_five_hour_and_the_weekly_window(cx: &mut TestAppContext) {
        let cx = open(Some(300.), Some(260.), cx);
        let (five, week) = (cx.debug_bounds("status-window-Claude-5h"), cx.debug_bounds("status-window-Claude-7d"));
        assert!(five.is_some() && week.is_some(), "both windows show");
        assert!(five.unwrap().right() <= week.unwrap().left(), "the short one first");
        assert!(cx.debug_bounds("status-window-Codex-30d").is_some(), "a single window shows alone");
    }
}
