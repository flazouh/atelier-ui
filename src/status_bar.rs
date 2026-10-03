//! The bar at the foot of the window: how hard this machine works (processor and memory), how many agents work or wait
//! on the reader, and how much of each provider's plan is used. It takes plain data and draws it; the app measures and
//! asks. Calm while there is room, amber from 60%, red from 85%, as the context meter is. A hover tells the numbers.

mod consts;
mod enums;
mod impls;
mod structs;

pub use consts::HEIGHT;
pub use enums::{GaugeState, Pressure};
pub use structs::{Gauge, ProviderGauge, StatusBar, SystemLoad, Work};

#[cfg(test)]
mod tests;
