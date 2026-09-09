//! Raspberry Pi backend using `/dev/serial0` and BCM GPIO numbering.
use crate::*;
use rppal::gpio::{Gpio, InputPin, OutputPin};
use serialport::{ClearBuffer, DataBits, FlowControl, SerialPort, StopBits};
use std::{
    io::{Read, Write},
    time::{Duration, Instant},
};

pub struct RppalTransport {
    path: String,
    port: Option<Box<dyn SerialPort>>,
    m0: OutputPin,
    m1: OutputPin,
    aux: InputPin,
    uart: UartConfig,
}
impl RppalTransport {
    pub fn new(path: impl Into<String>, m0_bcm: u8, m1_bcm: u8, aux_bcm: u8) -> Result<Self> {
        let gpio = Gpio::new().map_err(|e| Error::Transport(e.to_string()))?;
        Ok(Self {
            path: path.into(),
            port: None,
            m0: gpio
                .get(m0_bcm)
                .map_err(|e| Error::Transport(e.to_string()))?
                .into_output_low(),
            m1: gpio
                .get(m1_bcm)
                .map_err(|e| Error::Transport(e.to_string()))?
                .into_output_low(),
            aux: gpio
                .get(aux_bcm)
                .map_err(|e| Error::Transport(e.to_string()))?
                .into_input(),
            uart: UartConfig {
                baud: 9600,
                parity: Parity::None,
            },
        })
    }
    fn port(&mut self) -> Result<&mut Box<dyn SerialPort>> {
        self.port.as_mut().ok_or(Error::DeviceNotOpen)
    }
    fn apply(&mut self) -> Result<()> {
        let cfg = self.uart;
        let p = self.port()?;
        p.set_baud_rate(cfg.baud)
            .map_err(|e| Error::Transport(e.to_string()))?;
        p.set_data_bits(DataBits::Eight)
            .map_err(|e| Error::Transport(e.to_string()))?;
        p.set_stop_bits(StopBits::One)
            .map_err(|e| Error::Transport(e.to_string()))?;
        p.set_flow_control(FlowControl::None)
            .map_err(|e| Error::Transport(e.to_string()))?;
        p.set_parity(match cfg.parity {
            Parity::None => serialport::Parity::None,
            Parity::Odd => serialport::Parity::Odd,
            Parity::Even => serialport::Parity::Even,
        })
        .map_err(|e| Error::Transport(e.to_string()))
    }
}
impl Transport for RppalTransport {
    fn open(&mut self) -> Result<()> {
        let p = serialport::new(&self.path, self.uart.baud)
            .timeout(Duration::from_millis(100))
            .open()
            .map_err(|e| Error::Transport(e.to_string()))?;
        self.port = Some(p);
        self.apply()
    }
    fn close(&mut self) -> Result<()> {
        self.port.take();
        Ok(())
    }
    fn is_open(&self) -> bool {
        self.port.is_some()
    }
    fn set_uart_config(&mut self, c: UartConfig) -> Result<()> {
        self.uart = c;
        if self.port.is_some() {
            self.apply()
        } else {
            Ok(())
        }
    }
    fn uart_config(&self) -> UartConfig {
        self.uart
    }
    fn set_mode_pins(&mut self, p: ModePins) -> Result<()> {
        if p.m1 {
            self.m1.set_high()
        } else {
            self.m1.set_low()
        }
        if p.m0 {
            self.m0.set_high()
        } else {
            self.m0.set_low()
        }
        Ok(())
    }
    fn aux_level(&self) -> Result<AuxLevel> {
        Ok(if self.aux.is_high() {
            AuxLevel::High
        } else {
            AuxLevel::Low
        })
    }
    fn write_all(&mut self, b: &[u8], timeout: Duration) -> Result<()> {
        let start = Instant::now();
        let mut off = 0;
        while off < b.len() {
            if start.elapsed() >= timeout {
                return Err(Error::UartTimeout {
                    operation: "writing",
                });
            }
            match self.port()?.write(&b[off..]) {
                Ok(0) => {}
                Ok(n) => off += n,
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                Err(e) => return Err(Error::Transport(e.to_string())),
            }
        }
        self.port()?
            .flush()
            .map_err(|e| Error::Transport(e.to_string()))
    }
    fn read_exact(&mut self, b: &mut [u8], timeout: Duration) -> Result<()> {
        let start = Instant::now();
        let mut off = 0;
        while off < b.len() {
            if start.elapsed() >= timeout {
                return Err(Error::ShortResponse {
                    expected: b.len(),
                    actual: off,
                });
            }
            match self.port()?.read(&mut b[off..]) {
                Ok(0) => {}
                Ok(n) => off += n,
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                Err(e) => return Err(Error::Transport(e.to_string())),
            }
        }
        Ok(())
    }
    fn read_some(&mut self, b: &mut [u8], timeout: Duration) -> Result<usize> {
        let start = Instant::now();
        loop {
            if !timeout.is_zero() && start.elapsed() >= timeout {
                return Err(Error::UartTimeout {
                    operation: "reading",
                });
            }
            match self.port()?.read(b) {
                Ok(0) => {}
                Ok(n) => return Ok(n),
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                Err(e) => return Err(Error::Transport(e.to_string())),
            }
        }
    }
    fn flush_input(&mut self) -> Result<()> {
        self.port()?
            .clear(ClearBuffer::Input)
            .map_err(|e| Error::Transport(e.to_string()))
    }
}
