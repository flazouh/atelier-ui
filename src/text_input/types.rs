/// The field's height: atelier's control size.
pub const HEIGHT: f32 = 28.;

/// The field's corner: atelier's field radius.
pub const CORNER: f32 = 8.;

/// The gap between the label, the field and the message.
pub const GAP: f32 = 4.;

/// The room the message line keeps when reserved (`min-h-4`).
pub const MESSAGE_LINE: f32 = 16.;

/// The text's left edge with no icon and with one.
pub const TEXT_INSET: f32 = 10.;

pub const TEXT_INSET_ICON: f32 = 30.;

/// How long the shake takes, and where it goes (`x` in px, at even steps).
pub const SHAKE_SECONDS: f32 = 0.45;

pub const SHAKE: [f32; 7] = [0., -6., 6., -4., 4., -2., 0.];

/// How long the message takes to come up.
pub(super) const MESSAGE_SECONDS: f32 = 0.2;

/// The linear curve, for progress measured in seconds.
pub(super) const LINEAR: [f32; 4] = [0., 0., 1., 1.];

/// Motion's ease-in-out, between the shake's keyframes.
pub(super) const KEYFRAME_EASE: [f32; 4] = [0.42, 0., 0.58, 1.];
