use super::super::{enums::Pressure, structs::Gauge};

impl Gauge {
    pub fn new(
        label: impl Into<gpui_kit::SharedString>,
        used: f32,
        resets_in: Option<u64>,
    ) -> Self {
        Self {
            label: label.into(),
            used,
            resets_in,
        }
    }

    /// The fraction used, held between 0 and 1.
    pub fn fraction(&self) -> f32 {
        if self.used.is_nan() {
            0.
        } else {
            self.used.clamp(0., 1.)
        }
    }

    /// `77%`.
    pub fn percent(&self) -> String {
        format!("{}%", (self.fraction() * 100.).round() as u32)
    }

    pub fn pressure(&self) -> Pressure {
        Pressure::of(self.fraction())
    }

    /// `5h: 77% used, resets in 12 min`.
    pub fn words(&self) -> String {
        match self.resets_in {
            Some(secs) => format!(
                "{}: {} used, resets {}",
                self.label,
                self.percent(),
                when(secs)
            ),
            None => format!("{}: {} used", self.label, self.percent()),
        }
    }
}

/// `in under a minute`, `in 12 min`, `in 2 h 5 min`, `in 3 d 4 h`.
fn when(secs: u64) -> String {
    let (minutes, hours, days) = (secs / 60, secs / 3600, secs / 86_400);
    match secs {
        0..=59 => "in under a minute".into(),
        60..=3599 => format!("in {minutes} min"),
        3600..=86_399 => format!("in {hours} h {} min", minutes % 60),
        _ => format!("in {days} d {} h", hours % 24),
    }
}
