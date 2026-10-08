use std::collections::{BTreeMap, HashSet};

use gpui_kit::SharedString;

use super::helpers::{compact, walk};
use super::types::Node;
use crate::changed_files::ChangedFile;

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
    pub(super) folders: BTreeMap<String, Builder>,
    pub(super) files: BTreeMap<String, ChangedFile>,
}

impl Builder {
    fn insert(&mut self, parts: &[&str], file: &ChangedFile) {
        match parts {
            [] => {}
            [name] => {
                self.files.insert(name.to_string(), file.clone());
            }
            [folder, rest @ ..] => self
                .folders
                .entry(folder.to_string())
                .or_default()
                .insert(rest, file),
        }
    }

    pub(super) fn build(self) -> Vec<Node> {
        let mut folders: Vec<_> = self.folders.into_iter().collect();
        let mut files: Vec<_> = self.files.into_iter().collect();
        // By name without regard to case, as VS Code lists them; the map's own order breaks ties.
        folders.sort_by_key(|(name, _)| name.to_lowercase());
        files.sort_by_key(|(name, _)| name.to_lowercase());
        let folders = folders
            .into_iter()
            .map(|(name, b)| compact(name, b.build()));
        let files = files
            .into_iter()
            .map(|(name, file)| Node::File { name, file });
        folders.chain(files).collect()
    }
}

impl FileTree {
    pub fn new(files: &[ChangedFile]) -> Self {
        let mut root = Builder::default();
        for file in files {
            let parts: Vec<&str> = file.path.split('/').filter(|p| !p.is_empty()).collect();
            root.insert(&parts, file);
        }
        Self {
            roots: root.build(),
        }
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
            out.push(TreeRow {
                depth,
                name: name.into(),
                path: path.to_string().into(),
                added,
                removed,
                file,
                folded: is_folded,
            });
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
