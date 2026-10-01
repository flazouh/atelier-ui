use gpui_kit::SharedString;

use crate::{
    sidebar_model::{ProjectData},
};
use super::types::SessionFilter;

/// The projects with only the sessions that pass `filter`. A project stays even with none left.
pub fn narrow(projects: &[ProjectData], filter: SessionFilter) -> Vec<ProjectData> {
    projects
        .iter()
        .map(|project| {
            let mut project = project.clone();
            project.sessions.retain(|s| filter.keeps(s));
            project
        })
        .collect()
}

/// How many sessions of `projects` the filter hides, for a line such as "3 archived".
pub fn hidden_by(projects: &[ProjectData], filter: SessionFilter) -> usize {
    projects.iter().flat_map(|p| p.sessions.iter()).filter(|s| !filter.keeps(s)).count()
}

/// The words for the filter button's tooltip.
pub fn describe(filter: SessionFilter) -> SharedString {
    if filter.is_default() { "Filter sessions".into() } else { format!("Showing: {}", filter.words()).into() }
}
