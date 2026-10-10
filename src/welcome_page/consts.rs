/// Where the picture is served from: the grain gradient rendered for a whole window.
pub const HERO_PATH: &str = "welcome-hero.jpg";
/// The picture's ratio; gpui gives an image the ratio of its file unless one is set.
pub(super) const HERO_RATIO: f32 = 1680. / 1050.;
/// The space at the left and right of the page, and over the mark and under the button, in design pixels.
pub(super) const SIDE: f32 = 64.;
pub(super) const EDGE: f32 = 56.;
/// The mark at the top left.
pub(super) const MARK_SIZE: f32 = 40.;
/// The title: its size, how far down the page it starts as a share of the page's height, and how far left its
/// first letter is pulled so its stem lines up with the mark.
pub(super) const TITLE_SIZE: f32 = 92.;
pub(super) const TITLE_TOP: f32 = 0.235;
pub(super) const TITLE_PULL: f32 = 5.;
/// The line under the title: its size, its line, the space over it, how wide it may run and how strong its ink is.
pub(super) const TEXT_SIZE: f32 = 24.;
pub(super) const TEXT_LINE: f32 = 34.;
pub(super) const TEXT_TOP: f32 = 22.;
pub(super) const TEXT_WIDTH: f32 = 600.;
pub(super) const TEXT_ALPHA: f32 = 0.88;
/// The timeline, in seconds from the first frame: when the first word comes, how long between two words, how long
/// after the last word the button comes, and how long it takes to come.
pub(super) const WORDS_AT: f32 = 0.6;
pub(super) const WORD_STEP: f32 = 0.085;
pub(super) const ACTION_WAIT: f32 = 0.35;
pub(super) const ACTION_SECONDS: f32 = 0.45;
