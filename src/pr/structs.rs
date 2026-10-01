use gpui_kit::SharedString;

use super::types::{ChecksSummary, PrState};

/// Everything a chip shows about one pull request.
#[derive(Clone, Debug, PartialEq)]
pub struct PrChipData {
    pub number: u64,
    /// `owner/repo`.
    pub repo: SharedString,
    pub title: SharedString,
    pub state: PrState,
    /// Where Open and Copy link point.
    pub url: SharedString,
}

impl PrChipData {
    /// `#3344`.
    pub fn label(&self) -> SharedString {
        format!("#{}", self.number).into()
    }
}

/// How many checks of each kind a pull request has.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Checks {
    pub passed: u32,
    pub failed: u32,
    pub running: u32,
}

impl Checks {
    pub fn summary(self) -> ChecksSummary {
        if self.failed > 0 {
            ChecksSummary::Failing(self.failed)
        } else if self.running > 0 {
            ChecksSummary::Running
        } else if self.passed > 0 {
            ChecksSummary::Passed(self.passed)
        } else {
            ChecksSummary::None
        }
    }
}
