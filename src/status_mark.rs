//! beui's todo status marks (`TodoStatusIcon` in `components/agents/todo-list.tsx`), drawn as the same
//! SVG geometry on a 24-unit grid: a circle of radius 9, dashed `2 3` when pending, a progress arc when in
//! progress, and a check or cross stroke that draws itself in.

mod helpers;
mod structs;
mod types;

pub(crate) use helpers::{arc_points, at, partial, stroke};
pub use structs::{Mark, StatusMark};
#[cfg(test)]
pub(crate) use types::SLASH;

#[cfg(test)]
use gpui_kit::Hsla;

#[cfg(test)]
mod tests;
