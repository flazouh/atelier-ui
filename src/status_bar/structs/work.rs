/// What the agents are doing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Work {
    pub working: usize,
    pub needs_you: usize,
}
