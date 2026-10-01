use crate::{
    theme::{Theme},
    };

/// An imported theme, the tokens it had to derive, and the colours moved for contrast.
pub struct Imported {
    pub theme: Theme,
    pub derived: Vec<&'static str>,
    pub raised: Vec<&'static str>,
}
