use crate::icon::IconName;

/// One piece of a hint: a key symbol drawn as an icon, or plain text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyPart {
    Symbol(IconName),
    Text(String),
}
