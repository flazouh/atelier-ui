//! A pull request's checks, after GitQuiet's `Checks.tsx`: failing checks first and open, then the jobs
//! that were allowed to fail, then the rest behind one line. A failing check shows its Fault with no
//! click: the failing step, and the line of its log that names the cause. A Tolerated job (it failed in
//! a run that succeeded) says "Allowed to fail" and is never why the run is red. A running check spins.

mod helpers;
mod structs;
mod types;

pub use helpers::{clean_line, fault, groups, standing, tolerated_text};
pub use structs::{CheckRun, ChecksPanel, Fault, Groups, JobStep};
pub use types::{CheckState, Standing};

#[cfg(test)]
use gpui_kit::SharedString;

#[cfg(test)]
mod tests;
