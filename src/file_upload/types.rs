use super::structs::UploadItem;

pub(super) const ROW_TIME: f32 = 0.22;

pub(super) const SWAP_TIME: f32 = 0.16;

pub(super) const BAR_TIME: f32 = 0.28;

pub(super) const ROW_RISE: f32 = 8.;

pub(super) const ROW_LEAVE: f32 = 6.;

pub(super) const SWAP_SHIFT: f32 = 4.;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UploadStatus {
    Queued,
    Uploading,
    Success,
    Error,
}

impl UploadStatus {
    pub fn word(self) -> &'static str {
        match self {
            UploadStatus::Queued => "Queued",
            UploadStatus::Uploading => "Uploading",
            UploadStatus::Success => "Uploaded",
            UploadStatus::Error => "Failed",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UploadVariant {
    /// A row: the tile, the words, and Browse.
    #[default]
    Row,
    /// A column, roomier.
    Centered,
}

/// How many cells the bar of an upload has, across the card.
pub(super) const UPLOAD_CELLS: usize = 40;

pub enum FileUploadEvent {
    /// Files were dropped or picked, and are in the queue as uploading.
    Added(Vec<UploadItem>),
    Removed(UploadItem),
    /// A failed file was asked to try again: it is uploading from 0.
    Retried(UploadItem),
}
