use gpui_kit::SharedString;

use super::types::{ChecksSummary, PrState, ReviewState};

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
    /// What the forge said beyond the title, once it said it.
    pub facts: Option<PrFacts>,
}

impl PrChipData {
    /// `#3344`.
    pub fn label(&self) -> SharedString {
        format!("#{}", self.number).into()
    }

    /// The title without a conventional-commit head: `chore(ui): Faster chips` is `Faster chips`.
    pub fn short_title(&self) -> &str {
        let title = self.title.as_ref();
        let Some((head, rest)) = title.split_once(": ") else { return title };
        let kind = head.split('(').next().unwrap_or(head).trim_end_matches('!');
        let conventional = !kind.is_empty() && kind.chars().all(|c| c.is_ascii_lowercase()) && !rest.is_empty();
        if conventional { rest } else { title }
    }
}

/// A pull request's author, size, talk, review and checks, as a list row knows them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PrFacts {
    pub author: SharedString,
    pub added: u32,
    pub removed: u32,
    pub comments: u32,
    pub review: ReviewState,
    /// `None` until the checks are known.
    pub checks: Option<Checks>,
    /// Unix seconds; 0 when the forge did not say.
    pub updated_at: u64,
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
