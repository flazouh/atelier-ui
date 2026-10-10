/// How tall the picture band is, in design pixels.
pub(super) const BAND_HEIGHT: f32 = 420.;
/// The width the picture is stretched for; gpui gives an image the ratio of its file unless one is set.
pub(super) const BAND_WIDTH: f32 = 1280.;
/// The space at the left and right of the page.
pub(super) const SIDE: f32 = 64.;
/// The Atelier mark: its size and its distance from the top of the band.
pub(super) const MARK_SIZE: f32 = 44.;
pub(super) const MARK_TOP: f32 = 64.;
/// The title: its size, and its distance from the bottom of the band.
pub(super) const TITLE_SIZE: f32 = 56.;
pub(super) const TITLE_BOTTOM: f32 = 40.;
/// The sentence under the band: its size and the space over it.
pub(super) const TEXT_SIZE: f32 = 18.;
pub(super) const TEXT_TOP: f32 = 32.;
/// The space between the sentence and the button.
pub(super) const ACTION_TOP: f32 = 32.;
/// The progress bars: the width and height of one, the gap between two, and the distance from the foot.
pub(super) const BAR_WIDTH: f32 = 28.;
pub(super) const BAR_HEIGHT: f32 = 3.;
pub(super) const BAR_GAP: f32 = 6.;
pub(super) const BARS_BOTTOM: f32 = 48.;
/// How strong a bar not yet reached is, as a share of the foreground.
pub(super) const BAR_REST_ALPHA: f32 = 0.14;
