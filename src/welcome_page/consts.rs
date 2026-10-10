/// Where the pictures are served from: the grain gradient rendered for a whole window, and the take that breathes
/// over it.
pub const HERO_PATH: &str = "welcome-hero.jpg";
pub const HERO_B_PATH: &str = "welcome-hero-b.jpg";
/// The pictures' ratio; gpui gives an image the ratio of its file unless one is set.
pub(super) const HERO_RATIO: f32 = 1680. / 1050.;
/// How dark the foot of the page is made, so the words stand on the picture whatever it shows there.
pub(super) const SCRIM_ALPHA: f32 = 0.38;
/// The mark, the line and the button, and the gaps between them, in design pixels.
pub(super) const MARK_SIZE: f32 = 72.;
pub(super) const LINE_SIZE: f32 = 19.;
pub(super) const LINE_TOP: f32 = 28.;
pub(super) const LINE_ALPHA: f32 = 0.72;
pub(super) const ACTION_TOP: f32 = 40.;
/// How far a part rises as it comes in.
pub(super) const RISE: f32 = 18.;
/// The timeline, in seconds from the first frame: when each part starts, and how long it takes.
pub(super) const PICTURE_SECONDS: f32 = 1.8;
pub(super) const ZOOM_FROM: f32 = 1.10;
pub(super) const ZOOM_SECONDS: f32 = 14.;
/// The slow breath that never ends: the second picture's peak strength and one full in-and-out, and the first
/// picture's size swing over one in-and-out.
pub(super) const BREATH_PEAK: f32 = 0.65;
pub(super) const BREATH_SECONDS: f32 = 11.;
pub(super) const SWING: f32 = 0.012;
pub(super) const SWING_SECONDS: f32 = 22.;
pub(super) const MARK_AT: f32 = 0.5;
pub(super) const MARK_SECONDS: f32 = 1.0;
pub(super) const LINE_AT: f32 = 1.1;
pub(super) const LINE_SECONDS: f32 = 1.0;
pub(super) const ACTION_AT: f32 = 1.8;
pub(super) const ACTION_SECONDS: f32 = 0.9;
/// When the opening is over: after this, only the breath moves.
pub(super) const INTRO_SECONDS: f32 = 3.5;
