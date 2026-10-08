use crate::{
    menu::Lead,
    status_bar::{Gauge, GaugeState, Pressure, ProviderGauge},
};

fn claude() -> ProviderGauge {
    ProviderGauge::new("Claude", Lead::Monogram)
        .gauge(Gauge::new("5h", 0.4, Some(600)))
        .gauge(Gauge::new("7d", 0.9, Some(86_400 * 2)))
}

#[test]
fn the_window_that_stops_the_reader_first_is_the_one_shown() {
    let provider = claude();
    assert_eq!(provider.tightest().map(|g| g.label.as_ref()), Some("7d"));
    assert_eq!(provider.pressure(), Pressure::Hot);
}

#[test]
fn a_provider_with_no_windows_is_calm_and_has_none_tightest() {
    let provider = ProviderGauge::new("OpenRouter", Lead::Monogram);
    assert!(provider.tightest().is_none());
    assert_eq!(provider.pressure(), Pressure::Calm);
}

#[test]
fn the_hover_lists_every_window_and_the_note() {
    let words = claude().note("$100.03 of $100 extra usage").tooltip();
    assert_eq!(
        words.lines().collect::<Vec<_>>(),
        [
            "Claude",
            "5h: 40% used, resets in 10 min",
            "7d: 90% used, resets in 2 d 0 h",
            "$100.03 of $100 extra usage"
        ]
    );
}

#[test]
fn the_hover_says_why_there_are_no_numbers_and_when_they_are_old() {
    let none = ProviderGauge::new("Codex", Lead::Monogram)
        .state(GaugeState::Unavailable("Codex is not signed in".into()));
    assert_eq!(
        none.tooltip().lines().collect::<Vec<_>>(),
        ["Codex", "Codex is not signed in"]
    );
    let old = claude().state(GaugeState::Stale);
    assert_eq!(
        old.tooltip().lines().last(),
        Some("Showing the last numbers read")
    );
}
