//! The mark of atelier: a disc in the ink with an "A" cut out of it in the page's tone, so on the page it reads as
//! the primary button does, the page inverted. It has no colour of its own: a theme gives it both tones. The "A"
//! is `assets/atelier-mark.svg`, the shape tools/mac/make-icon.sh draws the app icon from.

mod helpers;
mod structs;
mod types;

pub(crate) use helpers::bytes;
pub use structs::AtelierMark;
pub use types::PATH;
