use gpui_kit::SharedString;

use crate::menu::Lead;

use super::super::{
    enums::{GaugeState, Pressure},
    structs::{Gauge, ProviderGauge},
};

impl ProviderGauge {
    pub fn new(name: impl Into<SharedString>, lead: Lead) -> Self {
        Self {
            name: name.into(),
            lead,
            gauges: Vec::new(),
            note: None,
            state: GaugeState::Live,
        }
    }

    pub fn gauge(mut self, gauge: Gauge) -> Self {
        self.gauges.push(gauge);
        self
    }

    pub fn note(mut self, note: impl Into<SharedString>) -> Self {
        self.note = Some(note.into());
        self
    }

    pub fn state(mut self, state: GaugeState) -> Self {
        self.state = state;
        self
    }

    /// The window nearest its end: the one that stops the reader first.
    pub fn tightest(&self) -> Option<&Gauge> {
        self.gauges.iter().max_by(|a, b| a.fraction().total_cmp(&b.fraction()))
    }

    pub fn pressure(&self) -> Pressure {
        self.tightest().map_or(Pressure::Calm, Gauge::pressure)
    }

    /// What a hover tells: the name, each window, the note, and why there are no numbers when there are none.
    pub fn tooltip(&self) -> String {
        let mut lines = vec![self.name.to_string()];
        if let GaugeState::Unavailable(why) = &self.state {
            lines.push(why.to_string());
        }
        lines.extend(self.gauges.iter().map(Gauge::words));
        lines.extend(self.note.iter().map(|note| note.to_string()));
        if self.state == GaugeState::Stale {
            lines.push("Showing the last numbers read".into());
        }
        lines.join("\n")
    }
}
