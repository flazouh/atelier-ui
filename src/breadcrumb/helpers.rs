use gpui_kit::ElementId;

use super::types::MIN_SHOWN;

/// What is shown of a path of `count` parts under `max_items`: the indices in order, with `None` for the
/// ellipsis. A path that fits shows every part.
pub fn shown(count: usize, max_items: usize) -> Vec<Option<usize>> {
    let limit = max_items.max(MIN_SHOWN);
    if count <= limit {
        return (0..count).map(Some).collect();
    }
    let tail = limit - 2;
    let mut out = vec![Some(0), None];
    out.extend((count - tail..count).map(Some));
    out
}

/// The parts the ellipsis holds: every one between the first and the tail.
pub fn hidden(count: usize, max_items: usize) -> std::ops::Range<usize> {
    let limit = max_items.max(MIN_SHOWN);
    if count <= limit { 0..0 } else { 1..count - (limit - 2) }
}

pub(super) fn key_id(id: &ElementId, at: usize) -> ElementId {
    ElementId::NamedChild(std::sync::Arc::new(id.clone()), format!("press-{at}").into())
}
