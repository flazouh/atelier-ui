use crate::changed_files::ChangedFile;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Node {
    Folder { name: String, children: Vec<Node> },
    File { name: String, file: ChangedFile },
}

impl Node {
    pub(super) fn sums(&self) -> (usize, usize) {
        match self {
            Node::File { file, .. } => (file.added, file.removed),
            Node::Folder { children, .. } => children.iter().map(Node::sums).fold((0, 0), |(a, r), (x, y)| (a + x, r + y)),
        }
    }
}
