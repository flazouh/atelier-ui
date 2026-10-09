/// Where the hero picture is served from.
pub const HERO_PATH: &str = "release-hero.jpg";

/// How tall the picture band is, in design pixels.
pub(super) const BAND_HEIGHT: f32 = 132.;
/// The width the picture is cut for, the width of the modal the sheet stands in. The picture is a strip of this shape,
/// stretched to the band, so its own top corners are the sheet's; gpui gives an image the ratio of its file unless one
/// is set. At another width the strip only stretches a little.
pub(super) const BAND_WIDTH: f32 = 860.;
/// The corner of the picture: the modal panel's, since gpui clips a box to its rectangle and not to its corners. The
/// picture is stretched over the band, so that its own top corners are the sheet's.
pub(super) const CORNER: f32 = crate::modal::CORNER;
/// The space at the left and right of the title and of the releases.
pub(super) const SIDE: f32 = 44.;
/// The title: its size, and its distance from the bottom of the band.
pub(super) const TITLE_SIZE: f32 = 34.;
pub(super) const TITLE_BOTTOM: f32 = 22.;
/// The close button at the top right: its size and its distance from the corner.
pub(super) const CLOSE_SIZE: f32 = 32.;
pub(super) const CLOSE_INSET: f32 = 18.;
/// How strong the close button's hover tone is, as a share of the light foreground.
pub(super) const CLOSE_HOVER_ALPHA: f32 = 0.18;
/// A release: the space over and under it, the width of its left column, and the gap to the notes.
pub(super) const ROW_PAD: f32 = 34.;
pub(super) const META_WIDTH: f32 = 150.;
pub(super) const COLUMN_GAP: f32 = 20.;
/// The left column: the version and the date under it.
pub(super) const VERSION_SIZE: f32 = 22.;
pub(super) const DATE_SIZE: f32 = 13.;
pub(super) const DATE_GAP: f32 = 4.;
/// A note: the lead, the text, the gap between them and the space under a note.
pub(super) const LEAD_SIZE: f32 = 15.;
pub(super) const TEXT_SIZE: f32 = 14.;
pub(super) const TEXT_LINE: f32 = 21.7;
pub(super) const LEAD_GAP: f32 = 6.;
pub(super) const NOTE_GAP: f32 = 24.;
