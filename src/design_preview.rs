//! Design preview: which of four designs the app shows for the editor tabs and for floating panels, so Alex can pick in the real app.
//! The choice is kept in the settings (Settings, "Design preview") and applies at once.
//!
//! design preview: remove after Alex picks (this file, its use in agent_panels.rs, editor_pane.rs, settings_pane.rs,
//! the settings keys `design_tabs`, `design_elevation` and `design_strength`, and the Variants story).

mod helpers;
mod structs;
mod types;

pub use helpers::{
    elevation, init, init_elevation, init_strength, panel_edge, panel_fill, panel_shadows,
    row_tone, set_elevation, set_strength, set_tabs, strength, tab_variant, tabs,
};
pub use types::{ELEVATION_DESIGNS, TABS_DESIGNS};

#[cfg(test)]
mod tests;
