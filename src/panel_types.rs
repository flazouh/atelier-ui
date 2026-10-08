//! The data the agent panels take. The app owns what goes in a panel; atelier-ui lays panels out and draws their
//! tabs, their group headers and their fold.

mod helpers;
mod structs;
mod types;

pub(crate) use helpers::element_id;
pub use helpers::{content_from, draw_content};
pub use structs::{DraggedEdge, DraggedTab, PanelData, PanelsState, ProjectLabel};
pub use types::{Layout, PanelContent, PanelsEvent};
