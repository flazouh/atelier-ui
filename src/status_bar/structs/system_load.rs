/// How hard this machine works.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SystemLoad {
    /// The processor in use now, 0 to 1.
    pub cpu: f32,
    /// The last samples of it, oldest first.
    pub cpu_history: Vec<f32>,
    /// Bytes of memory in use, and all there is.
    pub memory_used: u64,
    pub memory_total: u64,
    /// Bytes the app itself holds.
    pub app_memory: Option<u64>,
}
