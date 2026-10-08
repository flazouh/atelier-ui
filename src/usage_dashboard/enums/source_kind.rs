/// A subscription has windows that reset; a key is paid by use.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SourceKind {
    #[default]
    Subscription,
    Key,
}
