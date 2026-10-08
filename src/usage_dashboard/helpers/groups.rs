use super::super::structs::{SourceGroup, UsageSource};

/// The runs of sources with the same group caption, in order.
pub fn groups(sources: &[UsageSource]) -> Vec<SourceGroup> {
    let mut out: Vec<SourceGroup> = Vec::new();
    for (at, source) in sources.iter().enumerate() {
        match out.last_mut() {
            Some(group) if group.caption == source.group => group.len += 1,
            _ => out.push(SourceGroup { caption: source.group.clone(), first: at, len: 1 }),
        }
    }
    out
}
