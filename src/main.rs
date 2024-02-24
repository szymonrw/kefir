use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use std::io::prelude::*;
use std::net::TcpStream;
use std::{cmp, env};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let command = if args.len() > 1 { args[1].as_str() } else { "" };
    let mut stream = TcpStream::connect("192.168.0.2:50001")?;

    match command {
        "source" | "src" | "s" => {}
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

#[derive(Debug, FromPrimitive, ToPrimitive)]
enum Source {
    Wifi = 0b0010,
    USB = 0b1100,
    Bluetooth = 0b1001,
    AUX = 0b1010,
    Optical = 0b1011,
}

#[derive(Debug, FromPrimitive, ToPrimitive)]
enum AutoStandby {
    TwentyMinutes = 0b00,
    SixtyMinutes = 0b01,
    Never = 0b10,
}

#[derive(Debug, FromPrimitive, ToPrimitive)]
enum Power {
    Off = 1,
    On = 0,
}

#[derive(Debug, FromPrimitive, ToPrimitive)]
enum SpeakerOrientation {
    MainIsLeft = 1,
    MainIsRight = 0,
}

#[derive(Debug)]
struct Status {
    power: Power,
    orientation: SpeakerOrientation,
    source: Source,
    standby: AutoStandby,
}

fn get_status(stream: &mut TcpStream) -> std::io::Result<Status> {
    let bytes = send(stream, &[0x47, 0x30, 0x80])?;
    let bits = bytes[3];
    println!("{bits:b}");
    let source = match FromPrimitive::from_u8(bits & 0b1111) {
        Some(Source::Wifi) => Source::Wifi,
        Some(Source::USB) => Source::USB,
        Some(Source::Bluetooth) => Source::Bluetooth,
        Some(Source::AUX) => Source::AUX,
        Some(Source::Optical) => Source::Optical,
        None => Source::Optical,
    };
    let standby = match FromPrimitive::from_u8((bits >> 4) & 0b11) {
        Some(AutoStandby::TwentyMinutes) => AutoStandby::TwentyMinutes,
        Some(AutoStandby::SixtyMinutes) => AutoStandby::SixtyMinutes,
        Some(AutoStandby::Never) => AutoStandby::Never,
        None => AutoStandby::TwentyMinutes,
    };
    let orientation = match FromPrimitive::from_u8((bits >> 6) & 1) {
        Some(SpeakerOrientation::MainIsLeft) => SpeakerOrientation::MainIsLeft,
        Some(SpeakerOrientation::MainIsRight) => SpeakerOrientation::MainIsRight,
        None => SpeakerOrientation::MainIsRight,
    };
    let power = match FromPrimitive::from_u8((bits >> 7) & 1) {
        Some(Power::Off) => Power::Off,
        Some(Power::On) => Power::On,
        None => Power::Off,
    };
    Ok(Status {
        power,
        orientation,
        source,
        standby,
    })
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
