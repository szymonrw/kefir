use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::{fmt::Display, io::Result, net::TcpStream};

use crate::client::send;

trait Setting {
    fn from_bits(bits: u8) -> Self;
    fn to_bits(&self) -> u8;
}

#[derive(Copy, Clone, FromPrimitive)]
pub enum Source {
    Wifi = 0b0010,
    USB = 0b1100,
    BluetoothPaired = 0b1001,
    BluetoothUnpaired = 0b1111,
    AUX = 0b1010,
    Optical = 0b1011,
}

impl Setting for Source {
    fn to_bits(&self) -> u8 {
        (self.clone() as u8) & 0b1111
    }

    fn from_bits(bits: u8) -> Self {
        match Source::from_u8(bits & 0b1111) {
            Some(x) => x,
            None => Source::Optical,
        }
    }
}

impl Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            Source::Wifi => "Wifi",
            Source::USB => "USB",
            Source::BluetoothPaired | Source::BluetoothUnpaired => "Bluetooth",
            Source::AUX => "AUX",
            Source::Optical => "Optical",
        };
        write!(f, "{}", out)
    }
}

#[derive(Copy, Clone, FromPrimitive)]
pub enum AutoOff {
    TwentyMinutes = 0b00,
    SixtyMinutes = 0b01,
    Never = 0b10,
}

impl Setting for AutoOff {
    fn from_bits(bits: u8) -> Self {
        match AutoOff::from_u8((bits >> 4) & 0b11) {
            Some(x) => x,
            None => AutoOff::TwentyMinutes,
        }
    }

    fn to_bits(&self) -> u8 {
        (self.clone() as u8) << 4
    }
}

impl Display for AutoOff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            AutoOff::TwentyMinutes => "20 minutes",
            AutoOff::SixtyMinutes => "60 minutes",
            AutoOff::Never => "Never",
        };
        write!(f, "{}", out)
    }
}

#[derive(Copy, Clone, FromPrimitive)]
pub enum SpeakerOrientation {
    MainIsRight = 0,
    MainIsLeft = 1,
}

impl Setting for SpeakerOrientation {
    fn from_bits(bits: u8) -> Self {
        match SpeakerOrientation::from_u8((bits >> 6) & 1) {
            Some(x) => x,
            None => SpeakerOrientation::MainIsRight,
        }
    }

    fn to_bits(&self) -> u8 {
        (self.clone() as u8) << 6
    }
}

impl Display for SpeakerOrientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            Self::MainIsLeft => "Main speaker is on the left",
            Self::MainIsRight => "Main speaker is on the right",
        };
        write!(f, "{}", out)
    }
}

#[derive(Copy, Clone, FromPrimitive)]
pub enum Power {
    On = 0,
    Off = 1,
}

impl Power {
    pub fn invert(&self) -> Power {
        match self {
            Power::Off => Power::On,
            Power::On => Power::Off,
        }
    }
}

impl Setting for Power {
    fn from_bits(bits: u8) -> Self {
        match Power::from_u8((bits >> 7) & 1) {
            Some(x) => x,
            None => Power::Off,
        }
    }

    fn to_bits(&self) -> u8 {
        (self.clone() as u8) << 7
    }
}

impl Display for Power {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            Power::On => "On",
            Power::Off => "Off",
        };
        write!(f, "{}", out)
    }
}

#[derive(Copy, Clone)]
pub struct Status {
    pub power: Power,
    pub orientation: SpeakerOrientation,
    pub source: Source,
    pub auto_off: AutoOff,
}

impl Setting for Status {
    fn from_bits(bits: u8) -> Self {
        Status {
            source: Source::from_bits(bits),
            auto_off: AutoOff::from_bits(bits),
            orientation: SpeakerOrientation::from_bits(bits),
            power: Power::from_bits(bits),
        }
    }
    fn to_bits(&self) -> u8 {
        self.source.to_bits()
            | self.auto_off.to_bits()
            | self.orientation.to_bits()
            | self.power.to_bits()
    }
}

pub fn change_status<F>(stream: &mut TcpStream, f: F) -> Result<()>
where
    F: Fn(&mut Status),
{
    let mut status = get_status(stream)?;
    f(&mut status);
    set_status(stream, status)?;
    Ok(())
}

pub fn set_source(stream: &mut TcpStream, source: Source) -> Result<()> {
    change_status(stream, |status| {
        status.source = source;
        status.power = Power::On;
    })?;
    Ok(())
}

pub fn get_status(stream: &mut TcpStream) -> Result<Status> {
    let bytes = send(stream, &[0x47, 0x30, 0x80])?;
    let bits = bytes[3];
    Ok(Status::from_bits(bits))
}

fn set_status(stream: &mut TcpStream, status: Status) -> Result<u8> {
    let bits = status.to_bits();
    send(stream, &[0x53, 0x30, 0x81, bits])?;
    Ok(bits)
}
