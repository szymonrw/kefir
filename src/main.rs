use env_logger;
use std::env;
use std::net::TcpStream;

mod client;
mod status;
mod volume;
use status::{change_status, get_status, set_source, AutoOff, Power, Source, SpeakerOrientation};
use volume::{change_volume, get_volume, set_volume};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let command = if args.len() > 1 { args[1].as_str() } else { "" };
    let mut stream = TcpStream::connect("192.168.0.2:50001")?;

    match command {
        "on" => {
            change_status(&mut stream, |status| {
                status.power = Power::On;
            })?;
        }
        "off" => {
            change_status(&mut stream, |status| {
                status.power = Power::Off;
            })?;
        }
        "toggle" | "t" => {
            change_status(&mut stream, |status| {
                status.power = status.power.invert();
            })?;
        }
        "wifi" | "w" => {
            set_source(&mut stream, Source::Wifi)?;
        }
        "usb" | "u" => {
            set_source(&mut stream, Source::USB)?;
        }
        "bluetooth" | "bt" | "b" => {
            set_source(&mut stream, Source::BluetoothPaired)?;
        }
        "aux" | "a" => {
            set_source(&mut stream, Source::AUX)?;
        }
        "optical" | "opt" | "o" => {
            set_source(&mut stream, Source::Optical)?;
        }
        "source" | "src" | "s" => {
            println!("{0}", get_status(&mut stream)?.source);
        }
        "auto-off" | "ao" => {
            let auto_off = if args.len() > 2 {
                let new_auto_off = match args[2].as_str() {
                    "20" => AutoOff::TwentyMinutes,
                    "60" => AutoOff::SixtyMinutes,
                    "never" | "off" => AutoOff::Never,
                    unknown => panic!("Unknown auto off value {unknown}"),
                };
                change_status(&mut stream, |status| {
                    status.auto_off = new_auto_off;
                })?;
                new_auto_off
            } else {
                get_status(&mut stream)?.auto_off
            };
            println!("{auto_off}")
        }
        "main" | "orientation" => {
            let orientation = if args.len() > 2 {
                let new_orientation = match args[2].as_str() {
                    "left" => SpeakerOrientation::MainIsLeft,
                    "right" => SpeakerOrientation::MainIsRight,
                    unknown => panic!("Unknown orientation value {unknown}"),
                };
                change_status(&mut stream, |status| {
                    status.orientation = new_orientation;
                })?;
                new_orientation
            } else {
                get_status(&mut stream)?.orientation
            };
            println!("{orientation}");
        }
        "+" => {
            let amount = args[2].parse::<i8>()?;
            let volume = change_volume(&mut stream, amount)?;
            println!("{volume}");
        }
        "-" => {
            let amount = -args[2].parse::<i8>()?;
            let volume = change_volume(&mut stream, amount)?;
            println!("{volume}");
        }
        "volume" | "vol" | "v" => {
            let volume = if args.len() > 2 {
                let volume = u8::from_str_radix(&args[2], 10)?;
                set_volume(&mut stream, volume)?
            } else {
                get_volume(&mut stream)?
            };
            println!("{volume}");
        }
        "status" | "" => {
            let status = get_status(&mut stream)?;
            let volume = get_volume(&mut stream)?;
            println!("Power:\t\t{0}", status.power);
            println!("Source:\t\t{0}", status.source);
            println!("Volume:\t\t{0}", volume);
            println!("Auto-Off:\t{0}", status.auto_off);
            println!("Orientation:\t{0}", status.orientation);
        }
        cmd => {
            println!("Unknown command: {cmd}");
        }
    }

    stream.shutdown(std::net::Shutdown::Both)?;
    Ok(())
}
