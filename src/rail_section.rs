//! A card on the left rail of the pull request view, after GitQuiet's `Section.tsx`: a head with the
//! part's name and a short summary in the tone of its news ("Checks  CI is red — 1 of 3 failing"), and
//! the part under it. Borderless: the card is a `card` fill on the page.

mod structs;
mod types;

pub use structs::RailSection;
pub use types::SectionTone;
