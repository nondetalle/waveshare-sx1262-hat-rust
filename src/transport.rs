use crate::Result;
use std::time::Duration;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuxLevel { Low, High }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModePins { pub m1: bool, pub m0: bool }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UartConfig { pub baud: u32, pub parity: crate::Parity }

impl UartConfig {
    pub const CONFIGURATION: Self = Self { baud: 9_600, parity: crate::Parity::None };
}

/// Platform abstraction. Implementations must honor the supplied timeouts.
pub trait Transport: Send {
    fn open(&mut self) -> Result<()>;
    fn close(&mut self) -> Result<()>;
    fn is_open(&self) -> bool;
    fn set_uart_config(&mut self, config: UartConfig) -> Result<()>;
    fn uart_config(&self) -> UartConfig;
    fn set_mode_pins(&mut self, pins: ModePins) -> Result<()>;
    fn aux_level(&self) -> Result<AuxLevel>;
    fn write_all(&mut self, bytes: &[u8], timeout: Duration) -> Result<()>;
    fn read_exact(&mut self, bytes: &mut [u8], timeout: Duration) -> Result<()>;
    fn read_some(&mut self, bytes: &mut [u8], timeout: Duration) -> Result<usize>;
    fn flush_input(&mut self) -> Result<()>;
}
