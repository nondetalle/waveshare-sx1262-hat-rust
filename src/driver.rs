use crate::*;
use parking_lot::Mutex;
use std::{
    sync::Arc,
    thread,
    time::{Duration, Instant, SystemTime},
};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mode {
    Normal,
    Wor,
    Configuration,
    Sleep,
}
impl Mode {
    fn pins(self) -> ModePins {
        match self {
            Self::Normal => ModePins {
                m1: false,
                m0: false,
            },
            Self::Wor => ModePins {
                m1: false,
                m0: true,
            },
            Self::Configuration => ModePins {
                m1: true,
                m0: false,
            },
            Self::Sleep => ModePins { m1: true, m0: true },
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum State {
    Closed,
    Starting,
    Normal,
    Wor,
    Configuration,
    Sleep,
    Fault,
}
#[derive(Clone, Copy, Debug)]
pub struct Timeouts {
    pub mode: Duration,
    pub uart: Duration,
    pub debounce: Duration,
}
impl Default for Timeouts {
    fn default() -> Self {
        Self {
            mode: Duration::from_secs(3),
            uart: Duration::from_secs(2),
            debounce: Duration::from_millis(5),
        }
    }
}
#[derive(Clone, Debug)]
pub struct ProductId(pub [u8; 7]);
impl ProductId {
    pub fn hex(&self) -> String {
        self.0.iter().map(|b| format!("{b:02X}")).collect()
    }
}
#[derive(Clone, Debug)]
pub struct ReceivedFrame {
    pub payload: Vec<u8>,
    pub rssi_raw: Option<u8>,
    pub rssi_dbm: Option<f32>,
    pub timestamp: SystemTime,
}
#[derive(Clone, Debug, Default)]
pub struct Diagnostics {
    pub aux_timeouts: u64,
    pub malformed_responses: u64,
    pub mode_transitions: u64,
    pub persistent_writes: u64,
    pub last_mode_transition: Option<(Mode, SystemTime)>,
    pub last_transaction: Option<RawTransaction>,
}
struct Inner {
    transport: Box<dyn Transport>,
    state: State,
    normal_uart: UartConfig,
    timeouts: Timeouts,
    diag: Diagnostics,
    configuration: Option<Configuration>,
}
#[derive(Clone)]
pub struct Driver {
    inner: Arc<Mutex<Inner>>,
}
impl Driver {
    pub fn new(transport: Box<dyn Transport>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                transport,
                state: State::Closed,
                normal_uart: UartConfig {
                    baud: 9600,
                    parity: Parity::None,
                },
                timeouts: Timeouts::default(),
                diag: Diagnostics::default(),
                configuration: None,
            })),
        }
    }
    pub fn set_timeouts(&self, v: Timeouts) {
        self.inner.lock().timeouts = v
    }
    pub fn open(&self) -> Result<()> {
        let mut i = self.inner.lock();
        i.state = State::Starting;
        i.transport.open()?;
        Self::transition(&mut i, Mode::Normal)?;
        Ok(())
    }
    pub fn close(&self) -> Result<()> {
        let mut i = self.inner.lock();
        i.transport.close()?;
        i.state = State::Closed;
        Ok(())
    }
    pub fn state(&self) -> State {
        self.inner.lock().state
    }
    pub fn diagnostics(&self) -> Diagnostics {
        self.inner.lock().diag.clone()
    }
    fn ensure_open(i: &Inner) -> Result<()> {
        if !i.transport.is_open() || i.state == State::Closed {
            Err(Error::DeviceNotOpen)
        } else {
            Ok(())
        }
    }
    fn wait_aux(i: &mut Inner, timeout: Duration) -> Result<()> {
        let until = Instant::now() + timeout;
        loop {
            if i.transport.aux_level()? == AuxLevel::High {
                thread::sleep(i.timeouts.debounce);
                if i.transport.aux_level()? == AuxLevel::High {
                    return Ok(());
                }
            }
            if Instant::now() >= until {
                i.diag.aux_timeouts += 1;
                return Err(Error::AuxTimeout(timeout));
            }
            thread::sleep(Duration::from_millis(1))
        }
    }
    pub fn wait_aux_ready(&self, timeout: Duration) -> Result<()> {
        let mut i = self.inner.lock();
        Self::ensure_open(&i)?;
        Self::wait_aux(&mut i, timeout)
    }
    fn transition(i: &mut Inner, mode: Mode) -> Result<()> {
        Self::ensure_open(i)?;
        let timeout = i.timeouts.mode;
        Self::wait_aux(i, timeout)?;
        i.transport.set_mode_pins(mode.pins())?;
        Self::wait_aux(i, timeout).map_err(|_| Error::ModeTransitionTimeout { mode, timeout })?;
        let uart = if mode == Mode::Configuration {
            UartConfig::CONFIGURATION
        } else {
            i.normal_uart
        };
        i.transport.set_uart_config(uart)?;
        i.state = match mode {
            Mode::Normal => State::Normal,
            Mode::Wor => State::Wor,
            Mode::Configuration => State::Configuration,
            Mode::Sleep => State::Sleep,
        };
        i.diag.mode_transitions += 1;
        i.diag.last_mode_transition = Some((mode, SystemTime::now()));
        Ok(())
    }
    pub fn enter_mode(&self, mode: Mode) -> Result<()> {
        let mut i = self.inner.lock();
        Self::transition(&mut i, mode)
    }
    fn tx(i: &mut Inner, cmd: Command, start: u8, data: &[u8], read_len: usize) -> Result<Vec<u8>> {
        if i.state != State::Configuration {
            return Err(Error::WrongMode {
                required: Mode::Configuration,
                actual: i.state,
            });
        }
        let timeout = i.timeouts.uart;
        let (v, tr) = protocol::transact(&mut *i.transport, cmd, start, data, read_len, timeout)?;
        i.diag.last_transaction = Some(tr);
        Self::wait_aux(i, timeout)?;
        Ok(v)
    }
    pub fn read_registers(&self, start: u8, length: usize) -> Result<Vec<u8>> {
        let mut i = self.inner.lock();
        Self::tx(&mut i, Command::Read, start, &[], length)
    }
    pub fn write_registers_temporary(&self, start: u8, data: &[u8]) -> Result<()> {
        let mut i = self.inner.lock();
        Self::tx(&mut i, Command::TemporaryWrite, start, data, 0)?;
        Ok(())
    }
    pub fn write_registers_persistent(&self, start: u8, data: &[u8]) -> Result<()> {
        let mut i = self.inner.lock();
        Self::tx(&mut i, Command::PersistentWrite, start, data, 0)?;
        i.diag.persistent_writes += 1;
        Ok(())
    }
    pub fn read_product_id(&self) -> Result<ProductId> {
        let v = self.read_registers(0x80, 7)?;
        let mut a = [0; 7];
        a.copy_from_slice(&v);
        Ok(ProductId(a))
    }
    pub fn read_configuration(&self) -> Result<Configuration> {
        let v = self.read_registers(0, 9)?;
        let mut a = [0; 9];
        a.copy_from_slice(&v);
        let c = Configuration::decode(a);
        self.inner.lock().configuration = Some(c.clone());
        Ok(c)
    }
    pub fn apply_configuration(
        &self,
        c: &Configuration,
        persistent: bool,
        target: Mode,
    ) -> Result<()> {
        c.validate()?;
        let mut i = self.inner.lock();
        Self::ensure_open(&i)?;
        Self::transition(&mut i, Mode::Configuration)?;
        i.transport.flush_input()?;
        let current = Self::tx(&mut i, Command::Read, 0, &[], 9)?;
        let desired = c.encode()?;
        let mut first = 0usize;
        while first < 9 && current[first] == desired[first] {
            first += 1
        }
        if first < 9 {
            let mut last = 8;
            while last > first && current[last] == desired[last] {
                last -= 1
            }
            let cmd = if persistent {
                Command::PersistentWrite
            } else {
                Command::TemporaryWrite
            };
            Self::tx(&mut i, cmd, first as u8, &desired[first..=last], 0)?;
            if persistent {
                i.diag.persistent_writes += 1
            }
            let actual = Self::tx(&mut i, Command::Read, 0, &[], 7)?;
            if actual != desired[..7] {
                return Err(Error::ReadbackMismatch {
                    start: 0,
                    expected: desired[..7].to_vec(),
                    actual,
                });
            }
        }
        i.normal_uart = UartConfig {
            baud: c.uart_baud.as_u32(),
            parity: c.uart_parity,
        };
        i.configuration = Some(c.clone());
        Self::transition(&mut i, target)
    }
    fn require_data_mode(i: &Inner) -> Result<()> {
        match i.state {
            State::Normal | State::Wor => Ok(()),
            _ => Err(Error::WrongMode {
                required: Mode::Normal,
                actual: i.state,
            }),
        }
    }
    pub fn send_transparent(&self, payload: &[u8]) -> Result<()> {
        if payload.is_empty() || payload.len() > 1000 {
            return Err(Error::InvalidPayloadLength(payload.len()));
        }
        let mut i = self.inner.lock();
        Self::require_data_mode(&i)?;
        let timeout = i.timeouts.uart;
        i.transport.write_all(payload, timeout)?;
        Self::wait_aux(&mut i, timeout)
    }
    pub fn send_fixed(&self, address: u16, channel: u8, payload: &[u8]) -> Result<()> {
        if channel > 80 {
            return Err(Error::InvalidChannel(channel));
        }
        if payload.is_empty() || payload.len() > 997 {
            return Err(Error::InvalidPayloadLength(payload.len()));
        }
        let mut frame = Vec::with_capacity(payload.len() + 3);
        frame.extend_from_slice(&address.to_be_bytes());
        frame.push(channel);
        frame.extend_from_slice(payload);
        let mut i = self.inner.lock();
        Self::require_data_mode(&i)?;
        let timeout = i.timeouts.uart;
        i.transport.write_all(&frame, timeout)?;
        Self::wait_aux(&mut i, timeout)
    }
    pub fn send_broadcast(&self, channel: u8, payload: &[u8]) -> Result<()> {
        self.send_fixed(0xffff, channel, payload)
    }
    pub fn receive(
        &self,
        max_len: usize,
        timeout: Duration,
        known_packet_boundary: bool,
    ) -> Result<ReceivedFrame> {
        if max_len == 0 {
            return Err(Error::InvalidPayloadLength(0));
        }
        let mut i = self.inner.lock();
        Self::require_data_mode(&i)?;
        let mut buf = vec![0; max_len];
        let n = i.transport.read_some(&mut buf, timeout)?;
        buf.truncate(n);
        let appended = i
            .configuration
            .as_ref()
            .map(|c| c.append_rssi)
            .unwrap_or(false)
            && known_packet_boundary
            && !buf.is_empty();
        let raw = if appended { buf.pop() } else { None };
        Ok(ReceivedFrame {
            payload: buf,
            rssi_raw: raw,
            rssi_dbm: raw.map(|v| -(v as f32) / 2.0),
            timestamp: SystemTime::now(),
        })
    }
    pub fn query_ambient_rssi_experimental(&self) -> Result<u8> {
        let mut i = self.inner.lock();
        Self::require_data_mode(&i)?;
        let req = [0xc0, 0xc1, 0xc2, 0xc3, 0x00, 0x01];
        let timeout = i.timeouts.uart;
        i.transport.write_all(&req, timeout)?;
        let mut r = [0; 4];
        i.transport.read_exact(&mut r, timeout)?;
        if r[..3] != [0xc1, 0, 1] {
            return Err(Error::MalformedResponse(format!(
                "ambient RSSI response: {r:02X?}"
            )));
        }
        Self::wait_aux(&mut i, timeout)?;
        Ok(r[3])
    }
    pub fn resynchronize(&self) -> Result<()> {
        let mut i = self.inner.lock();
        Self::ensure_open(&i)?;
        i.transport.flush_input()?;
        Self::transition(&mut i, Mode::Configuration)?;
        i.transport.flush_input()?;
        Self::tx(&mut i, Command::Read, 0, &[], 3)?;
        Ok(())
    }
}
