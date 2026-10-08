use super::super::{enums::Selection, structs::UsageSource};

/// Whether `source` is in what `selection` picks.
pub fn is_selected(selection: &Selection, source: &UsageSource) -> bool {
    match selection {
        Selection::All => false,
        Selection::Group(group) => *group == source.group,
        Selection::Source(id) => *id == source.id,
    }
}
