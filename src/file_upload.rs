//! beui's FileUpload (`components/motion/file-upload.tsx`): a dropzone that takes files, and the queue of what is being
//! sent. The dropzone is a dashed 24px card in one of two shapes (`Centered`, a column with a 64px tile; `Row`, a
//! 56px tile, the words and a Browse pill); files dropped on it, or picked with Browse, become rows of the queue. A row
//! has the file's icon in a 44px tile, its name, its kind and size, a status mark that swaps as the status changes, a
//! retry button after a failure, a remove button, and a 6px progress bar.
//!
//! Motion: a row comes in from 8px below on 220ms of the out curve and leaves 6px up on the same; the rows round it
//! glide to their places; the bar's fill moves on 280ms of the out curve; the status mark leaves up and the next one
//! comes up from below, 4px each way, 160ms each. Under Reduce Motion it all jumps.
//!
//! The component holds the queue and sends events; sending the files is the owner's job. The owner adds progress with
//! [`FileUpload::set_items`] or [`FileUpload::update`].

mod helpers;
mod structs;
mod types;

pub use helpers::{clamp_progress, format_bytes, icon_of, kind_of, take_paths};
pub use structs::{FileUpload, UploadItem};
pub use types::{FileUploadEvent, UploadStatus, UploadVariant};

#[cfg(test)]
use std::path::PathBuf;
#[cfg(test)]
use crate::icon::IconName;

#[cfg(test)]
mod tests;
