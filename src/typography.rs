//! Geist and Geist Mono, embedded so the app never depends on installed fonts.

mod helpers;
mod types;

pub(crate) use helpers::load_fonts;
#[cfg(not(target_os = "macos"))]
pub(crate) use helpers::disable_font_smoothing;
#[cfg(target_os = "macos")]
pub(crate) use helpers::disable_font_smoothing;
pub use types::{FONT_FAMILY, MONO_FONT_FAMILY, SEGMENT_GAP, TextSize};
