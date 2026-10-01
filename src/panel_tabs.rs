//! The single view: one panel fills the area and horizontal tabs at the top pick it. A tab shows the status
//! mark (the agent's own mark, or the tone that replaces it when the session needs you, has finished
//! unseen, or stopped) and the title. Tabs scroll sideways when they overflow, reorder by drag, close with
//! their `×` or ⌘W, and ⌃Tab moves between them. Grouped, they gather by project under a small name.

mod impls;
mod structs;
mod types;

pub use types::TAB_HEIGHT;
