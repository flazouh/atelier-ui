/// The scale of the day chart: its top, a round number that holds the tallest day, and the ticks up to it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Axis {
    pub top: u64,
    /// The values the grid lines stand at, lowest first; the last is `top`.
    pub ticks: Vec<u64>,
}
