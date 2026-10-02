use crate::{icon::IconName, keys::Command};

pub(super) const GAP: f32 = 8.;

pub(super) const PADDING: f32 = 10.;

/// The file icon and its gap.
pub(super) const ICON: f32 = 14. + GAP;

/// The lookups a language server answers, in the order GitQuiet's keyboard sheet lists them.
pub(super) const LOOKUPS: [(Command, IconName); 5] = [
    (Command::Uses, IconName::Link),
    (Command::FileNames, IconName::FormatListBulleted),
    (Command::GoToName, IconName::DataObject),
    (Command::GoToFile, IconName::Description),
    // A comment on the line under the caret: the same as the `c` key, and a Tab stop for a reader without one.
    (Command::Comment, IconName::ChatBubble),
];
