//! UART modem driver for the Waveshare SX1262 868M LoRa HAT.
//!
//! This is intentionally not a native Semtech SX1262 SPI driver. The HAT exposes
//! a firmware-owned UART register protocol. All blocking operations are bounded.

mod config;
mod driver;
mod error;
mod protocol;
#[cfg(feature = "rppal-backend")]
pub mod rppal_backend;
mod transport;

pub use config::*;
pub use driver::*;
pub use error::*;
pub use protocol::{Command, RawTransaction};
pub use transport::{AuxLevel, ModePins, Transport, UartConfig};
