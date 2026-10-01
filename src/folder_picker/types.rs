use gpui_kit::SharedString;

pub(super) const ROW: f32 = 32.;

pub enum FolderPickerEvent {
    /// The picker needs the folders of this directory (`~/...` or absolute).
    Want(SharedString),
    /// The reader chose this folder.
    Choose(SharedString),
    Cancel,
}

/// Why a folder could not be listed or opened, in the terms the owner can tell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FolderError {
    /// There is nothing at that path.
    Missing,
    /// The reader may not look in it.
    Denied,
    /// It is a file.
    NotAFolder,
    /// Any other reason, in the system's words.
    Other(SharedString),
}

impl FolderError {
    /// A folder that is not there is only news, not trouble: it shows in the muted tone.
    pub fn is_quiet(&self) -> bool {
        matches!(self, FolderError::Missing | FolderError::NotAFolder)
    }
}
