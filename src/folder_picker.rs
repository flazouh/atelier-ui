//! A folder picker inside the app, for a reader whose system has no picker (Linux with no desktop portal) and for a
//! folder on another host. The reader types a path and the folders under the last `/` complete it: Tab completes as
//! far as the names agree, the arrow keys and a click choose among them, Enter goes into the chosen folder or, on a
//! folder already typed, opens it. The listing is the owner's job (it knows the host): the picker asks for a folder
//! with [`FolderPickerEvent::Want`] and shows what [`FolderPicker::show`] gives back.

mod helpers;
mod structs;
mod types;

pub use helpers::{error_words, folder_of, matches, split_path, tab_complete};
pub use structs::FolderPicker;
pub use types::{FolderError, FolderPickerEvent};

#[cfg(test)]
use structs::Listing;

#[cfg(test)]
use gpui_kit::SharedString;

#[cfg(test)]
mod tests;
