use std::io::prelude::*;
use std::net::TcpStream;
use std::{cmp, env};

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];
    let mut stream = TcpStream::connect("192.168.0.2:50001")?;

    match command.as_str() {
        "volume" => {
            let volume = if args.len() > 2 {
                let amount = match i8::from_str_radix(&args[2], 10) {
                    Ok(n) => n,
                    Err(x) => panic!("asdf"),
                };
                change_volume(&mut stream, amount)?
            } else {
                get_volume(&mut stream)?
            };
            println!("{volume:?}");
        }
        _ => {}
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
    send(stream, &[0x53, 0x25, 0x81, volume])?;
    Ok(volume)
}

fn send(stream: &mut TcpStream, cmd: &[u8]) -> std::io::Result<[u8; 16]> {
    let mut result = [0; 16];
    stream.write(&cmd)?;
    stream.read(&mut result)?;
    println!("{result:?}");
    Ok(result)
}
