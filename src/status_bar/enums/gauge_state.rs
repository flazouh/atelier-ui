use gpui_kit::SharedString;

/// How current a provider's numbers are.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum GaugeState {
    #[default]
    Live,
    /// The last numbers read, shown dimmed while a new read is not yet in.
    Stale,
    /// There are no numbers, and the words say why.
    Unavailable(SharedString),
}
