//! A project's badge in the sidebar: its first letter (two when names collide) on a colour from a fixed palette, or
//! the project's own icon in its place. The palette is acepe's twelve (`project-color-options.ts`). A project has a
//! colour of its own by where it lives, so it keeps it from run to run; one the reader picked beats that.

mod helpers;
mod structs;
mod types;

pub use helpers::{color_of, fallback_color, fill, ink_on, labels, palette};
pub use structs::{ProjectBadge, Swatch};
pub use types::{COUNT, SIZE};

#[cfg(test)]
mod tests;
