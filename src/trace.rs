//! `BEUI_TRACE_MOTION=1` prints a line, with the time in milliseconds since the first one, for each input a motion
//! component handles, for the frame that starts its animation, and for each frame the story draws while one runs.
//! It is for reading how many frames pass between a click and the first frame that moves, and how long the
//! frames are. Off, it costs one atomic read.
use std::{sync::OnceLock, time::Instant};

fn state() -> &'static (bool, Instant) {
    static ON: OnceLock<(bool, Instant)> = OnceLock::new();
    ON.get_or_init(|| (std::env::var_os("BEUI_TRACE_MOTION").is_some(), Instant::now()))
}

/// Whether tracing is on.
pub fn on() -> bool {
    state().0
}

/// Prints `what` under the component's `name`, when tracing is on.
pub fn motion(name: &str, what: &str) {
    let (on, epoch) = state();
    if *on {
        eprintln!("trace {:>9.1} ms  {name} {what}", epoch.elapsed().as_secs_f64() * 1000.);
    }
}
