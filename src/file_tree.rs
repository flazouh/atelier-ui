//! The changed files as a tree, for [`crate::changed_file_tree::ChangedFileTree`]: folders first, then
//! files, each by name, as VS Code and GitQuiet list them. A folder that holds only one folder merges
//! with it into one row ("crates/beui/src"), and every folder sums the lines its files add and remove.
//! Pure, so the whole shape is tested without a window.

use std::collections::{BTreeMap, HashSet};

use gpui_kit::SharedString;

use crate::changed_files::ChangedFile;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Node {
    Folder { name: String, children: Vec<Node> },
    File { name: String, file: ChangedFile },
}

impl Node {
    fn sums(&self) -> (usize, usize) {
        match self {
            Node::File { file, .. } => (file.added, file.removed),
            Node::Folder { children, .. } => children.iter().map(Node::sums).fold((0, 0), |(a, r), (x, y)| (a + x, r + y)),
        }
    }
}

/// One visible row of the tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeRow {
    pub depth: usize,
    /// What the row shows: a file's name, or a folder's name with any merged folders ("beui/src").
    pub name: SharedString,
    /// The full path: the file's, or the folder's with no trailing `/`.
    pub path: SharedString,
    pub added: usize,
    pub removed: usize,
    /// Some for a file row.
    pub file: Option<ChangedFile>,
    /// A folder row whose children are hidden.
    pub folded: bool,
}

impl TreeRow {
    pub fn is_folder(&self) -> bool {
        self.file.is_none()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileTree {
    roots: Vec<Node>,
}

/// Folders before files, at every level; `build` orders each by name.
#[derive(Default)]
struct Builder {
    folders: BTreeMap<String, Builder>,
    files: BTreeMap<String, ChangedFile>,
}

impl Builder {
    fn insert(&mut self, parts: &[&str], file: &ChangedFile) {
        match parts {
            [] => {}
            [name] => {
                self.files.insert(name.to_string(), file.clone());
            }
            [folder, rest @ ..] => self.folders.entry(folder.to_string()).or_default().insert(rest, file),
        }
    }

    fn build(self) -> Vec<Node> {
        let mut folders: Vec<_> = self.folders.into_iter().collect();
        let mut files: Vec<_> = self.files.into_iter().collect();
        // By name without regard to case, as VS Code lists them; the map's own order breaks ties.
        folders.sort_by_key(|(name, _)| name.to_lowercase());
        files.sort_by_key(|(name, _)| name.to_lowercase());
        let folders = folders.into_iter().map(|(name, b)| compact(name, b.build()));
        let files = files.into_iter().map(|(name, file)| Node::File { name, file });
        folders.chain(files).collect()
    }
}

/// A folder whose only child is a folder merges with it.
fn compact(name: String, children: Vec<Node>) -> Node {
    match <[Node; 1]>::try_from(children) {
        Ok([Node::Folder { name: inner, children }]) => Node::Folder { name: format!("{name}/{inner}"), children },
        Ok([only]) => Node::Folder { name, children: vec![only] },
        Err(children) => Node::Folder { name, children },
    }
}

impl FileTree {
    pub fn new(files: &[ChangedFile]) -> Self {
        let mut root = Builder::default();
        for file in files {
            let parts: Vec<&str> = file.path.split('/').filter(|p| !p.is_empty()).collect();
            root.insert(&parts, file);
        }
        Self { roots: root.build() }
    }

    /// The rows to show, with each folder in `folded` closed. A folder is named by its full path.
    pub fn rows(&self, folded: &HashSet<SharedString>) -> Vec<TreeRow> {
        let mut out = Vec::new();
        walk(&self.roots, "", 0, &mut |node, path, depth| {
            let (added, removed) = node.sums();
            let (name, file, is_folded) = match node {
                Node::File { name, file } => (name.clone(), Some(file.clone()), false),
                Node::Folder { name, .. } => (name.clone(), None, folded.contains(path)),
            };
            out.push(TreeRow { depth, name: name.into(), path: path.to_string().into(), added, removed, file, folded: is_folded });
            !is_folded
        });
        out
    }

    /// Every file's path in the order the tree lists them, whatever is folded: the order Next and
    /// Previous walk.
    pub fn file_order(&self) -> Vec<SharedString> {
        let mut out = Vec::new();
        walk(&self.roots, "", 0, &mut |node, path, _| {
            if let Node::File { .. } = node {
                out.push(path.to_string().into());
            }
            true
        });
        out
    }
}

/// Visits each node before its children, with its full path and depth. `visit` returns whether to go
/// into a folder.
fn walk(nodes: &[Node], parent: &str, depth: usize, visit: &mut impl FnMut(&Node, &str, usize) -> bool) {
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

#[cfg(test)]
mod tests;
