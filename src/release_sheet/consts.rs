/// Where the hero picture is served from.
pub const HERO_PATH: &str = "release-hero.jpg";
/// How tall the picture's own part is, where the kicker and the version stand.
pub(super) const HERO_HEIGHT: f32 = 232.;
/// The space round the words, the gap round the dark panel, and its corner.
pub(super) const SIDE: f32 = 34.;
pub(super) const PANEL_GAP: f32 = 12.;
pub(super) const PANEL_CORNER: f32 = 14.;
/// How much of the panel's fill shows: the picture shows through a little.
pub(super) const PANEL_ALPHA: f32 = 0.92;
/// The corner of the picture: the modal panel's, since gpui clips a box to its rectangle and not to its corners.
pub(super) const CORNER: f32 = crate::modal::CORNER;
