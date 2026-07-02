#![no_std]
use core::{
    fmt::{self, Write},
    ops::Deref,
};

use bytemuck::{Pod, Zeroable};
pub const MESSAGE: u8 = 0x01;
pub const PRINT: u8 = 0x00;

#[derive(Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct CommandPacket {
    pub purge_time: u32,
    pub purge_interval: u32,
    pub fan_speed: u8,
    pub restart: u8,
    pub mac_no: u8,
    pub send_wait: u8,
    pub receivers: [u8; 2],
    pub _pad: [u8; 2],
}

#[derive(Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct ScanPacket {
    pub id: u64,
}

#[derive(Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct HandshakeInit {
    /// 31 to avoid padding. idk!
    pub name: StaticString<31>,
    pub available_slots: u8,
}
#[derive(Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct HandshakeResponse {
    pub password: StaticString<32>,
}

#[derive(Default, Clone, Copy, Pod, Zeroable, Debug)]
#[repr(C)]
pub struct MessagePacket {
    pub acc: [f32; 3],
    pub gyr: [f32; 3],
    pub temp: f32,
    pub time_since_purge: u32,
    pub elapsed_time: u32,
    pub voltage: f32,
    pub current: f32,
    pub power: f32,
    pub fc_voltage: f32,
    pub fc_current: f32,
    pub batt_flags: u16,
    pub fc_flags: u16,
    pub energy_wh: f32,
    pub hydro_wh: f32,
    pub a1_voltage: f32,
    pub free_heap: u32,
    pub test_data: u32,
}

pub struct BufWriter<'a> {
    pub buf: &'a mut [u8],
    pub pos: usize,
}

impl<'a> Write for BufWriter<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        let remaining = self.buf.len().saturating_sub(self.pos);
        let n = remaining.min(bytes.len());
        self.buf[self.pos..self.pos + n].copy_from_slice(&bytes[..n]);
        self.pos += n;
        Ok(())
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Module {
    FcVcInit = 0x00,
    BatVcInit = 0x01,
    ImuInit = 0x02,
    I2CInit = 0x03,
    FcVc = 0x04,
    BatVc = 0x05,
    Imu = 0x06,
    Adc = 0x08,
    Ledc = 0x09,
}
impl core::fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl TryFrom<u8> for Module {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0x00 => Ok(Self::FcVcInit),
            0x01 => Ok(Self::BatVcInit),
            0x02 => Ok(Self::ImuInit),
            0x03 => Ok(Self::I2CInit),
            0x04 => Ok(Self::FcVc),
            0x05 => Ok(Self::BatVc),
            0x06 => Ok(Self::Imu),
            0x08 => Ok(Self::Adc),
            0x09 => Ok(Self::Ledc),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(transparent)]
pub struct StaticString<const LENGTH: usize> {
    pub data: [u8; LENGTH],
}
impl<const LENGTH: usize> Default for StaticString<LENGTH> {
    fn default() -> Self {
        Self::new("unnamed")
    }
}
impl<const LENGTH: usize> StaticString<LENGTH> {
    pub fn new(text: &str) -> Self {
        assert!(text.len() <= LENGTH);
        let mut packet = Self { data: [0; LENGTH] };
        packet.data[..text.len()].copy_from_slice(text.as_bytes());
        packet
    }
    pub fn as_str(&self) -> &str {
        let len = self
            .data
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(self.data.len());

        core::str::from_utf8(&self.data[..len]).unwrap()
    }
}
impl<const LENGTH: usize> Deref for StaticString<LENGTH> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}
