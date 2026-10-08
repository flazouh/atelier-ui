use gpui_kit::SharedString;

/// A provider in the chart's legend: its name and the shades of its hue that the sources use.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LegendEntry {
    pub name: SharedString,
    pub hue: usize,
    pub shades: Vec<usize>,
}
