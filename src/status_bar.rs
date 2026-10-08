//! The bar at the foot of the window, in three cards left to right: the app's version (a press shows what is new in it),
//! the agents that wait on the reader with how much of each provider's plan is used (a press opens the usage), and how
//! hard this machine works (processor and memory). It takes plain data and draws it; the app measures and
//! asks. Calm while there is room, amber from 60%, red from 85%, as the context meter is. A hover tells the numbers.

mod consts;
mod enums;
mod helpers;
mod impls;
mod structs;

pub use consts::HEIGHT;
pub use enums::{GaugeState, Pressure};
pub use structs::{Gauge, ProviderGauge, StatusBar, SystemLoad, Work};

#[cfg(test)]
mod tests;
