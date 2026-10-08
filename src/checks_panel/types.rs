use gpui_kit::SharedString;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckState {
    Passed,
    Failed,
    /// Failed in a run that succeeded: `continue-on-error`.
    Tolerated,
    Running,
    Queued,
    Skipped,
}

/// How the whole run stands, worst first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Standing {
    Empty,
    Red { failed: usize, total: usize },
    Running { waiting: usize, total: usize },
    Passed { total: usize },
    Stopped { green: usize, total: usize },
}

impl Standing {
    pub fn text(self) -> SharedString {
        match self {
            Self::Empty => "Nothing has run yet".into(),
            Self::Red { failed, total } => {
                format!("CI is red \u{2014} {failed} of {total} failing").into()
            }
            Self::Running { waiting, total } => {
                format!("{waiting} of {total} still running").into()
            }
            Self::Passed { total: 1 } => "All 1 check passed".into(),
            Self::Passed { total } => format!("All {total} checks passed").into(),
            Self::Stopped { green, total } => {
                format!("{green} of {total} passed, none failing").into()
            }
        }
    }
}
