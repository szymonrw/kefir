use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::io::prelude::*;
use std::net::TcpStream;
use std::{cmp, env};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let command = if args.len() > 1 { args[1].as_str() } else { "" };
    let mut stream = TcpStream::connect("192.168.0.2:50001")?;

    match command {
        "on" => {
            let mut status = get_status(&mut stream)?;
            match status.power {
                Power::Off => {
                    status.power = Power::On;
                    set_status(&mut stream, status)?;
                }
                Power::On => {}
            }
        }
        "off" => {
            let mut status = get_status(&mut stream)?;
            match status.power {
                Power::On => {
                    status.power = Power::Off;
                    set_status(&mut stream, status)?;
                }
                Power::Off => {}
            }
        }
        "toggle" => {
            let mut status = get_status(&mut stream)?;
            match status.power {
                Power::On => {
                    status.power = Power::Off;
                }
                Power::Off => {
                    status.power = Power::On;
                }
            };
            set_status(&mut stream, status)?;
        }
        "source" | "src" | "s" => {
            let source = if args.len() > 2 {
                let new_source = match args[2].as_str() {
                    "wifi" | "w" => Source::Wifi,
                    "usb" | "u" => Source::USB,
                    "bluetooth" | "bt" | "b" => Source::BluetoothPaired,
                    "aux" | "a" => Source::AUX,
                    "optical" | "opt" | "o" => Source::Optical,
                    unknown => panic!("Unknown source: {unknown}"),
                };
                let mut status = get_status(&mut stream)?;
                status.source = new_source;
                set_status(&mut stream, status)?;
                new_source
            } else {
                get_status(&mut stream)?.source
            };
            println!("{source:?}");
        }
        "auto-off" | "ao" => {
            let auto_off = if args.len() > 2 {
                let new_auto_off = match args[2].as_str() {
                    "20" => AutoOff::TwentyMinutes,
                    "60" => AutoOff::SixtyMinutes,
                    "never" | "off" => AutoOff::Never,
                    unknown => panic!("Unknown auto stand-by value {unknown}"),
                };
                let mut status = get_status(&mut stream)?;
                status.auto_off = new_auto_off;
                set_status(&mut stream, status)?;
                new_auto_off
            } else {
                get_status(&mut stream)?.auto_off
            };
            println!("{auto_off:?}")
        }
        "volume" | "vol" | "v" => {
            let volume = if args.len() > 3 {
                let sign: i8 = if args[2] == "-" { -1 } else { 1 };
                let amount = sign * args[3].parse::<i8>()?;
                change_volume(&mut stream, amount)?
            } else if args.len() > 2 {
                let volume = u8::from_str_radix(&args[2], 10)?;
                set_volume(&mut stream, volume)?
            } else {
                get_volume(&mut stream)?
            };
            println!("{volume:?}");
        }
        "status" | "" => {
            let status = get_status(&mut stream);
            println!("{status:?}");
        }
        cmd => {
            println!("Unknown command: {cmd}");
        }
    }

    stream.shutdown(std::net::Shutdown::Both)?;
    Ok(())
}

#[derive(Copy, Clone, FromPrimitive, Debug)]
enum Source {
    Wifi = 0b0010,
    USB = 0b1100,
    BluetoothPaired = 0b1001,
    BluetoothUnpaired = 0b1111,
    AUX = 0b1010,
    Optical = 0b1011,
}

#[derive(Copy, Clone, FromPrimitive, Debug)]
enum AutoOff {
    TwentyMinutes = 0b00,
    SixtyMinutes = 0b01,
    Never = 0b10,
}

#[derive(Copy, Clone, FromPrimitive, Debug)]
enum Power {
    Off = 1,
    On = 0,
}

#[derive(Copy, Clone, FromPrimitive, Debug)]
enum SpeakerOrientation {
    MainIsLeft = 1,
    MainIsRight = 0,
}

#[derive(Copy, Clone, Debug)]
struct Status {
    power: Power,
    orientation: SpeakerOrientation,
    source: Source,
    auto_off: AutoOff,
}

trait Compile {
    fn compile(&self) -> u8;
}

impl Compile for Status {
    fn compile(&self) -> u8 {
        self.source as u8
            | ((self.auto_off as u8) << 4)
            | ((self.orientation as u8) << 6)
            | ((self.power as u8) << 7)
    }
}

fn parse_status(bits: u8) -> Status {
    let source = match Source::from_u8(bits & 0b1111) {
        Some(x) => x,
        None => Source::Optical,
    };
    let auto_off = match AutoOff::from_u8((bits >> 4) & 0b11) {
        Some(x) => x,
        None => AutoOff::TwentyMinutes,
    };
    let orientation = match SpeakerOrientation::from_u8((bits >> 6) & 1) {
        Some(x) => x,
        None => SpeakerOrientation::MainIsRight,
    };
    let power = match Power::from_u8((bits >> 7) & 1) {
        Some(x) => x,
        None => Power::Off,
    };
    Status {
        power,
        orientation,
        source,
        auto_off,
    }
}

fn get_status(stream: &mut TcpStream) -> std::io::Result<Status> {
    let bytes = send(stream, &[0x47, 0x30, 0x80])?;
    let bits = bytes[3];
    Ok(parse_status(bits))
}

fn set_status(stream: &mut TcpStream, status: Status) -> std::io::Result<u8> {
    let bits = status.compile();
    send(stream, &[0x53, 0x30, 0x81, bits])?;
    Ok(bits)
}

fn change_volume(stream: &mut TcpStream, amount: i8) -> std::io::Result<u8> {
    let mut volume = get_volume(stream)?;
    volume = if amount < 0 {
        volume - cmp::min(-amount as u8, volume)
    } else {
        volume + cmp::min(amount as u8, 100 - volume)
    };
    set_volume(stream, volume)?;
    Ok(volume)
}

fn get_volume(stream: &mut TcpStream) -> std::io::Result<u8> {
    let bytes = send(stream, &[0x47, 0x25, 0x80])?;
    Ok(bytes[3])
}

fn set_volume(stream: &mut TcpStream, volume: u8) -> std::io::Result<u8> {
    let corrected = volume.clamp(0, 100);
    send(stream, &[0x53, 0x25, 0x81, corrected])?;
    Ok(corrected)
}

fn send(stream: &mut TcpStream, cmd: &[u8]) -> std::io::Result<[u8; 8]> {
    let mut result = [0; 8];
    stream.write(&cmd)?;
    stream.read(&mut result)?;
    println!("{cmd:?}<->{result:?}");
    Ok(result)
}
