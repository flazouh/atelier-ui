/// Where the picture is served from: the grain gradient rendered for a whole window.
pub const HERO_PATH: &str = "welcome-hero.jpg";
/// The picture's ratio; gpui gives an image the ratio of its file unless one is set.
pub(super) const HERO_RATIO: f32 = 1680. / 1050.;
/// The space at the left and right of the page, and over the mark and under the button, in design pixels.
pub(super) const SIDE: f32 = 64.;
pub(super) const EDGE: f32 = 56.;
/// The mark at the top left.
pub(super) const MARK_SIZE: f32 = 40.;
/// The title and the line under it stand in the middle of the page, lifted a little: the title's size, and the
/// space under the two that lifts them.
pub(super) const TITLE_SIZE: f32 = 92.;
pub(super) const LIFT: f32 = 24.;
/// The line under the title: its size, its line, the space over it, how wide it may run and how strong its ink is.
pub(super) const TEXT_SIZE: f32 = 24.;
pub(super) const TEXT_LINE: f32 = 34.;
pub(super) const TEXT_TOP: f32 = 22.;
pub(super) const TEXT_WIDTH: f32 = 600.;
pub(super) const TEXT_ALPHA: f32 = 0.88;
/// The timeline, in seconds from the moment the picture is loaded. The picture fades in; the title comes first, a word at a time; the line follows it,
/// faster; the button comes after the last word.
pub(super) const PICTURE_SECONDS: f32 = 0.4;
pub(super) const TITLE_AT: f32 = 0.5;
pub(super) const TITLE_STEP: f32 = 0.16;
pub(super) const TEXT_WAIT: f32 = 0.3;
pub(super) const TEXT_STEP: f32 = 0.085;
pub(super) const ACTION_WAIT: f32 = 0.35;
pub(super) const ACTION_SECONDS: f32 = 0.45;
