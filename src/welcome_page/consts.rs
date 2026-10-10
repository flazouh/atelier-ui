/// Where the picture is served from: the grain gradient rendered for a whole window.
pub const HERO_PATH: &str = "welcome-hero.jpg";
/// The picture's ratio; gpui gives an image the ratio of its file unless one is set.
pub(super) const HERO_RATIO: f32 = 1680. / 1050.;
/// The card: its width, its corner (the modal's) and the space inside it. It is opaque, so a word not in yet,
/// drawn in the card's own tone, is not there at all.
pub(super) const CARD_WIDTH: f32 = 440.;
pub(super) const CARD_CORNER: f32 = crate::modal::CORNER;
pub(super) const CARD_PAD: f32 = 32.;
/// The mark, the words and the button, and the gaps between them, in design pixels.
pub(super) const MARK_SIZE: f32 = 56.;
pub(super) const TEXT_SIZE: f32 = 17.;
pub(super) const TEXT_LINE: f32 = 26.;
pub(super) const TEXT_TOP: f32 = 24.;
pub(super) const ACTION_TOP: f32 = 28.;
/// The timeline, in seconds from the first frame: when the first word comes, how long between two words, how long
/// after the last word the button comes, and how long it takes to come.
pub(super) const WORDS_AT: f32 = 0.6;
pub(super) const WORD_STEP: f32 = 0.085;
pub(super) const ACTION_WAIT: f32 = 0.35;
pub(super) const ACTION_SECONDS: f32 = 0.45;
