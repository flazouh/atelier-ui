//! The chooser behind "Choose an icon…": the project's image files (see [`crate::icon_candidates`]), a field that filters
//! them by the words typed, and the arrow keys and Enter to choose. The owner lists the files (it knows the host) and
//! hears [`IconPickerEvent`]; a local project shows each image as a thumbnail from `root`.

mod structs;
mod types;

pub use structs::IconPicker;
pub use types::IconPickerEvent;

#[cfg(test)]
mod tests;
