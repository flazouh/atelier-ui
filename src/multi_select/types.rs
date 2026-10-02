use gpui_kit::SharedString;

/// The field: `min-h-11`, `rounded-xl`, `px-2.5 py-1.5`, `gap-2`, and a border in `--border` (`--border-strong` on hover).
pub(super) const FIELD_HEIGHT: f32 = 44.;

pub(super) const FIELD_RADIUS: f32 = 12.;

pub(super) const FIELD_PAD_X: f32 = 10.;

pub(super) const FIELD_PAD_Y: f32 = 6.;

pub(super) const FIELD_GAP: f32 = 8.;

/// A chip: `h-7`, `rounded-lg`, `px-2`, `gap-1`; its remove button is 20px, `-mr-1`, `rounded-md`, with a 12px cross.
pub(super) const CHIP_HEIGHT: f32 = 28.;

pub(super) const CHIP_RADIUS: f32 = 8.;

pub(super) const CHIP_GAP: f32 = 6.;

pub(super) const REMOVE: f32 = 20.;

/// The text field: `h-7 min-w-12 flex-1`, in a box `min-w-20`.
pub(super) const FIELD_INPUT_MIN: f32 = 48.;

pub(super) const FIELD_INPUT_BOX_MIN: f32 = 80.;

/// The list: `max-h-64`, `p-1.5`. A row is `py-2` under a 20px line; a group is `py-0.5` with a label of `py-1.5`
/// under a 10.88px font on a 16.32px line. The empty message is `px-3 py-8` under a 20px line.
pub(super) const LIST_MAX: f32 = 256.;

pub(super) const LIST_PAD: f32 = 6.;

pub(super) const ROW: f32 = 36.;

pub(super) const GROUP_PAD: f32 = 2.;

pub(super) const LABEL: f32 = 28.32;

pub(super) const EMPTY: f32 = 84.;

/// The list sits 6px below the field (`sideOffset`), and its edge is 1px.
pub(super) const SIDE_OFFSET: f32 = 6.;

/// A chip rises this far as it comes in; a chip's wipe takes 160ms.
pub(super) const CHIP_RISE: f32 = 6.;

pub(super) const WIPE: f32 = 0.16;

pub(super) const ENTER_FADE: f32 = 0.18;

/// `border-border` and `--border-strong`, as parts of the foreground; the focus ring is `foreground/20`.
pub(super) const BORDER: f32 = 0.06;

pub(super) const BORDER_STRONG: f32 = 0.12;

pub(super) const RING: f32 = 0.2;

/// The label's letter spacing, 0.12em of 10.88px.
pub(super) const TRACKING: f32 = 1.3056;

pub enum MultiSelectEvent {
    Change(Vec<SharedString>),
}
