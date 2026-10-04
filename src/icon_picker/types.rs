use gpui_kit::SharedString;

pub enum IconPickerEvent {
    /// The reader chose this file, a path under the project.
    Choose(SharedString),
    /// The reader wants the letter back in place of the image.
    Clear,
    /// The reader chose this colour of the palette (see [`crate::project_badge::palette`]) for the letter badge.
    Color(usize),
    Cancel,
}
