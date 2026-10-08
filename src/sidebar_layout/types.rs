/// When a row wears its project's badge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeShow {
    /// Where no project heading says which project it is: in the priority list.
    #[default]
    Auto,
    Always,
    Never,
}

impl BadgeShow {
    pub const ALL: [BadgeShow; 3] = [Self::Auto, Self::Always, Self::Never];

    pub fn words(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Always => "Always",
            Self::Never => "Never",
        }
    }

    /// As the settings keep it.
    pub fn key(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Always => "always",
            Self::Never => "never",
        }
    }

    pub fn from_key(key: Option<&str>) -> Self {
        Self::ALL
            .into_iter()
            .find(|b| Some(b.key()) == key)
            .unwrap_or_default()
    }
}

/// A project shows this many sessions before the rest fold into "Show N older", unless one that needs the reader
/// would fall beyond them.
pub const FOLD_AFTER: usize = 5;

/// The priority list shows this many earlier sessions before "Show more".
pub const EARLIER_SHOWN: usize = 8;

/// The counts the Settings page offers for those two.
pub const FOLD_CHOICES: [usize; 4] = [3, 5, 8, 12];

pub const EARLIER_CHOICES: [usize; 4] = [5, 8, 12, 20];
