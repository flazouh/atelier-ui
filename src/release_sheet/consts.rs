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
/// The corner of the picture: the modal panel's, since gpui clips a box to its rectangle and not to its corners. The picture is
/// stretched over the whole sheet, so that its own corners are the sheet's: a picture that covers the sheet and is cut by it
/// would have its rounded corners outside the visible part.
pub(super) const CORNER: f32 = crate::modal::CORNER;
/// How tall the notes may be before they scroll, when earlier versions are listed under them.
pub(super) const HISTORY_MAX: f32 = 340.;
/// The close button at the top right: its size and its distance from the corner.
pub(super) const CLOSE_SIZE: f32 = 30.;
pub(super) const CLOSE_INSET: f32 = 14.;
/// The tile behind a note's icon: its size, corner and icon, and how strong its colour is at the top, at the bottom, and in its ring.
pub(super) const TILE_SIZE: f32 = 36.;
pub(super) const TILE_CORNER: f32 = 10.;
pub(super) const TILE_ICON: f32 = 18.;
pub(super) const TILE_TOP_ALPHA: f32 = 0.32;
pub(super) const TILE_BOTTOM_ALPHA: f32 = 0.10;
pub(super) const TILE_RING_ALPHA: f32 = 0.42;
