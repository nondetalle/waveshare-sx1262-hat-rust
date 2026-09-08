use crate::{Error, Result};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum UartBaud {
    B1200 = 0,
    B2400 = 1,
    B4800 = 2,
    B9600 = 3,
    B19200 = 4,
    B38400 = 5,
    B57600 = 6,
    B115200 = 7,
}
impl UartBaud {
    pub fn as_u32(self) -> u32 {
        [1200, 2400, 4800, 9600, 19200, 38400, 57600, 115200][self as usize]
    }
    pub fn from_code(v: u8) -> Self {
        match v & 7 {
            0 => Self::B1200,
            1 => Self::B2400,
            2 => Self::B4800,
            3 => Self::B9600,
            4 => Self::B19200,
            5 => Self::B38400,
            6 => Self::B57600,
            _ => Self::B115200,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Parity {
    None,
    Odd,
    Even,
}
impl Parity {
    pub(crate) fn code(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Odd => 1,
            Self::Even => 2,
        }
    }
    pub(crate) fn from_code(v: u8) -> Self {
        match v & 3 {
            1 => Self::Odd,
            2 => Self::Even,
            _ => Self::None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum AirRate {
    Kbps0_3 = 0,
    Kbps1_2 = 1,
    Kbps2_4 = 2,
    Kbps4_8 = 3,
    Kbps9_6 = 4,
    Kbps19_2 = 5,
    Kbps38_4 = 6,
    Kbps62_5 = 7,
}
impl AirRate {
    pub fn kbps(self) -> f32 {
        [0.3, 1.2, 2.4, 4.8, 9.6, 19.2, 38.4, 62.5][self as usize]
    }
    pub fn from_code(v: u8) -> Self {
        match v & 7 {
            0 => Self::Kbps0_3,
            1 => Self::Kbps1_2,
            2 => Self::Kbps2_4,
            3 => Self::Kbps4_8,
            4 => Self::Kbps9_6,
            5 => Self::Kbps19_2,
            6 => Self::Kbps38_4,
            _ => Self::Kbps62_5,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum SubpacketSize {
    Bytes240 = 0,
    Bytes128 = 1,
    Bytes64 = 2,
    Bytes32 = 3,
}
impl SubpacketSize {
    pub fn bytes(self) -> usize {
        [240, 128, 64, 32][self as usize]
    }
    pub fn from_code(v: u8) -> Self {
        match v & 3 {
            0 => Self::Bytes240,
            1 => Self::Bytes128,
            2 => Self::Bytes64,
            _ => Self::Bytes32,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum TxPower {
    Dbm22 = 0,
    Dbm17 = 1,
    Dbm13 = 2,
    Dbm10 = 3,
}
impl TxPower {
    pub fn dbm(self) -> u8 {
        [22, 17, 13, 10][self as usize]
    }
    pub fn from_code(v: u8) -> Self {
        match v & 3 {
            0 => Self::Dbm22,
            1 => Self::Dbm17,
            2 => Self::Dbm13,
            _ => Self::Dbm10,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorRole {
    Receiver,
    Transmitter,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum WorPeriod {
    Ms500 = 0,
    Ms1000 = 1,
    Ms1500 = 2,
    Ms2000 = 3,
    Ms2500 = 4,
    Ms3000 = 5,
    Ms3500 = 6,
    Ms4000 = 7,
}
impl WorPeriod {
    pub fn milliseconds(self) -> u16 {
        (self as u16 + 1) * 500
    }
    pub fn from_code(v: u8) -> Self {
        match v & 7 {
            0 => Self::Ms500,
            1 => Self::Ms1000,
            2 => Self::Ms1500,
            3 => Self::Ms2000,
            4 => Self::Ms2500,
            5 => Self::Ms3000,
            6 => Self::Ms3500,
            _ => Self::Ms4000,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Configuration {
    pub address: u16,
    pub network_id: u8,
    pub uart_baud: UartBaud,
    pub uart_parity: Parity,
    pub air_rate: AirRate,
    pub subpacket_size: SubpacketSize,
    pub ambient_rssi_query_enabled: bool,
    pub tx_power: TxPower,
    pub channel: u8,
    pub append_rssi: bool,
    pub fixed_transmission: bool,
    pub relay_enabled: bool,
    pub lbt_enabled: bool,
    pub wor_role: WorRole,
    pub wor_period: WorPeriod,
    pub encryption_key: Option<u16>,
    pub raw_reserved_bits: u8,
}
impl Default for Configuration {
    fn default() -> Self {
        Self {
            address: 0,
            network_id: 0,
            uart_baud: UartBaud::B9600,
            uart_parity: Parity::None,
            air_rate: AirRate::Kbps2_4,
            subpacket_size: SubpacketSize::Bytes240,
            ambient_rssi_query_enabled: false,
            tx_power: TxPower::Dbm22,
            channel: 0x12,
            append_rssi: false,
            fixed_transmission: false,
            relay_enabled: false,
            lbt_enabled: false,
            wor_role: WorRole::Receiver,
            wor_period: WorPeriod::Ms500,
            encryption_key: None,
            raw_reserved_bits: 0,
        }
    }
}
impl Configuration {
    pub fn validate(&self) -> Result<()> {
        if self.channel > 80 {
            return Err(Error::InvalidChannel(self.channel));
        }
        if self.raw_reserved_bits & !0x1c != 0 {
            return Err(Error::UnsupportedConfiguration(
                "reserved bits must be contained in REG1 bits 4:2".into(),
            ));
        }
        Ok(())
    }
    pub fn frequency_mhz(&self) -> f64 {
        850.125 + self.channel as f64
    }
    pub fn channel_from_frequency_mhz(mhz: f64) -> Result<u8> {
        let c = mhz - 850.125;
        if c < 0.0 || c > 80.0 || (c - c.round()).abs() > 1e-6 {
            Err(Error::InvalidFrequency(mhz))
        } else {
            Ok(c.round() as u8)
        }
    }
    pub fn decode(raw: [u8; 9]) -> Self {
        let r0 = raw[3];
        let r1 = raw[4];
        let r3 = raw[6];
        Self {
            address: u16::from_be_bytes([raw[0], raw[1]]),
            network_id: raw[2],
            uart_baud: UartBaud::from_code(r0 >> 5),
            uart_parity: Parity::from_code(r0 >> 3),
            air_rate: AirRate::from_code(r0),
            subpacket_size: SubpacketSize::from_code(r1 >> 6),
            ambient_rssi_query_enabled: r1 & 0x20 != 0,
            tx_power: TxPower::from_code(r1),
            channel: raw[5],
            append_rssi: r3 & 0x80 != 0,
            fixed_transmission: r3 & 0x40 != 0,
            relay_enabled: r3 & 0x20 != 0,
            lbt_enabled: r3 & 0x10 != 0,
            wor_role: if r3 & 8 != 0 {
                WorRole::Transmitter
            } else {
                WorRole::Receiver
            },
            wor_period: WorPeriod::from_code(r3),
            encryption_key: None,
            raw_reserved_bits: r1 & 0x1c,
        }
    }
    pub fn encode(&self) -> Result<[u8; 9]> {
        self.validate()?;
        let [h, l] = self.address.to_be_bytes();
        let r0 = (self.uart_baud as u8) << 5 | self.uart_parity.code() << 3 | self.air_rate as u8;
        let r1 = (self.subpacket_size as u8) << 6
            | (self.ambient_rssi_query_enabled as u8) << 5
            | (self.raw_reserved_bits & 0x1c)
            | self.tx_power as u8;
        let r3 = (self.append_rssi as u8) << 7
            | (self.fixed_transmission as u8) << 6
            | (self.relay_enabled as u8) << 5
            | (self.lbt_enabled as u8) << 4
            | ((matches!(self.wor_role, WorRole::Transmitter) as u8) << 3)
            | self.wor_period as u8;
        let [kh, kl] = self.encryption_key.unwrap_or(0).to_be_bytes();
        Ok([h, l, self.network_id, r0, r1, self.channel, r3, kh, kl])
    }
}
