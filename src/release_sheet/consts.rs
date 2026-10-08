/// Where the hero picture is served from.
pub const HERO_PATH: &str = "release-hero.jpg";
/// How tall the picture's own part is, where the kicker and the version stand.
pub(super) const HERO_HEIGHT: f32 = 232.;
/// The picture reaches this far past every edge, so that it can drift without showing a border.
pub(super) const SPARE: f32 = 24.;
/// How much further out the picture starts, and the time it takes to zoom in to its place.
pub(super) const ZOOM: f32 = 40.;
pub(super) const ZOOM_SECONDS: f32 = 1.6;
/// One slow lap of the drift, and how long the drift takes to wake after the zoom.
pub(super) const DRIFT_SECONDS: f32 = 22.;
pub(super) const DRIFT_WAKE_AT: f32 = 0.8;
pub(super) const DRIFT_WAKE_SECONDS: f32 = 2.;
/// How long one piece takes to come in, and how far each rises.
pub(super) const REVEAL_SECONDS: f32 = 0.7;
pub(super) const TITLE_RISE: f32 = 14.;
pub(super) const ROW_RISE: f32 = 10.;
/// When each piece starts, in seconds from the open.
pub(super) const KICKER_AT: f32 = 0.12;
pub(super) const VERSION_AT: f32 = 0.2;
pub(super) const PANEL_AT: f32 = 0.34;
pub(super) const ROWS_AT: f32 = 0.42;
pub(super) const ROW_STEP: f32 = 0.08;
/// The space round the words, the gap round the dark panel, and its corner.
pub(super) const SIDE: f32 = 34.;
pub(super) const PANEL_GAP: f32 = 12.;
pub(super) const PANEL_CORNER: f32 = 14.;
/// How much of the panel's fill shows: the picture shows through a little.
pub(super) const PANEL_ALPHA: f32 = 0.92;
