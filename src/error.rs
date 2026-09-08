use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("device is not open")]
    DeviceNotOpen,
    #[error("AUX did not become ready within {0:?}")]
    AuxTimeout(Duration),
    #[error("mode transition to {mode:?} timed out after {timeout:?}")]
    ModeTransitionTimeout { mode: crate::Mode, timeout: Duration },
    #[error("UART timeout while {operation}")]
    UartTimeout { operation: &'static str },
    #[error("short response: expected {expected} bytes, got {actual}")]
    ShortResponse { expected: usize, actual: usize },
    #[error("malformed response: {0}")]
    MalformedResponse(String),
    #[error("module returned FF FF FF")]
    ModuleFormatError,
    #[error("unexpected response command 0x{0:02X}")]
    UnexpectedResponseCommand(u8),
    #[error("unexpected response address: expected 0x{expected:02X}, got 0x{actual:02X}")]
    UnexpectedResponseAddress { expected: u8, actual: u8 },
    #[error("unexpected response length: expected {expected}, got {actual}")]
    UnexpectedResponseLength { expected: u8, actual: u8 },
    #[error("write echo mismatch: expected {expected:02X?}, got {actual:02X?}")]
    WriteEchoMismatch { expected: Vec<u8>, actual: Vec<u8> },
    #[error("readback mismatch at 0x{start:02X}: expected {expected:02X?}, got {actual:02X?}")]
    ReadbackMismatch { start: u8, expected: Vec<u8>, actual: Vec<u8> },
    #[error("invalid channel {0}; supported hardware range is 0..=80")]
    InvalidChannel(u8),
    #[error("frequency {0} MHz is not on a 1 MHz channel in 850.125..=930.125 MHz")]
    InvalidFrequency(f64),
    #[error("invalid register range start=0x{start:02X}, length={length}")]
    InvalidRegisterRange { start: u8, length: usize },
    #[error("invalid payload length {0}")]
    InvalidPayloadLength(usize),
    #[error("operation requires {required:?} mode, current state is {actual:?}")]
    WrongMode { required: crate::Mode, actual: crate::State },
    #[error("unsupported configuration: {0}")]
    UnsupportedConfiguration(String),
    #[error("transport error: {0}")]
    Transport(String),
    #[error("device fault: {0}")]
    DeviceFault(String),
}

pub type Result<T> = std::result::Result<T, Error>;
