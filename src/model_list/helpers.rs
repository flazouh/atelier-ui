use gpui_kit::SharedString;

/// `order` with `id` taken out and put before `before` (at the end for none). An id not in the order, or dropped on itself, changes nothing.
pub fn moved(order: &[SharedString], id: &SharedString, before: Option<&SharedString>) -> Vec<SharedString> {
    if !order.contains(id) || before == Some(id) {
        return order.to_vec();
    }
    let mut rest: Vec<SharedString> = order.iter().filter(|o| *o != id).cloned().collect();
    let at = before.and_then(|b| rest.iter().position(|o| o == b)).unwrap_or(rest.len());
    rest.insert(at, id.clone());
    rest
}
