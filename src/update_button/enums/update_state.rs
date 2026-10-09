/// Which of its three looks the button wears.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UpdateState {
    /// The fraction downloaded, 0 to 1.
    Downloading(f32),
    Ready,
    Restarting,
}

impl UpdateState {
    /// A download at `fraction`, held to 0..=1.
    pub fn downloading(fraction: f32) -> Self {
        Self::Downloading(fraction.clamp(0., 1.))
    }

    /// The name a test finds this state by.
    pub fn selector(self) -> &'static str {
        match self {
            Self::Downloading(_) => "update-button-downloading",
            Self::Ready => "update-button-ready",
            Self::Restarting => "update-button-restarting",
        }
    }

    /// Only a ready update can be pressed.
    pub fn pressable(self) -> bool {
        matches!(self, Self::Ready)
    }
}
