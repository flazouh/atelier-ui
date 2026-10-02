use super::types::Node;

/// A folder whose only child is a folder merges with it.
pub(super) fn compact(name: String, children: Vec<Node>) -> Node {
    match <[Node; 1]>::try_from(children) {
        Ok([Node::Folder { name: inner, children }]) => Node::Folder { name: format!("{name}/{inner}"), children },
        Ok([only]) => Node::Folder { name, children: vec![only] },
        Err(children) => Node::Folder { name, children },
    }
}

/// Visits each node before its children, with its full path and depth. `visit` returns whether to go
/// into a folder.
pub(super) fn walk(nodes: &[Node], parent: &str, depth: usize, visit: &mut impl FnMut(&Node, &str, usize) -> bool) {
    for node in nodes {
        let name = match node {
            Node::Folder { name, .. } | Node::File { name, .. } => name,
        };
        let path = if parent.is_empty() { name.clone() } else { format!("{parent}/{name}") };
        let open = visit(node, &path, depth);
        if let (Node::Folder { children, .. }, true) = (node, open) {
            walk(children, &path, depth + 1, visit);
        }
    }
}
