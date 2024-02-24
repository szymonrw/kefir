use std::io::prelude::*;
use std::net::TcpStream;
use std::{cmp, env};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let command = if args.len() > 1 { args[1].as_str() } else { "" };
    let mut stream = TcpStream::connect("192.168.0.2:50001")?;

    match command {
        "volume" | "vol" | "v" => {
            let volume = if args.len() > 3 {
                let sign: i8 = if args[2] == "-" { -1 } else { 1 };
                let amount = sign * i8::from_str_radix(&args[3], 10)?;
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
            println!("Status: TODO");
        }
        cmd => {
            println!("Unknown command: {cmd}");
        }
    }

    stream.shutdown(std::net::Shutdown::Both)?;
    Ok(())
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
