use std::io::prelude::*;
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    let volume = get_volume();
    println!("{volume:?}");
    Ok(())
}

fn get_volume() -> std::io::Result<u8> {
    let bytes = match send([0x47, 0x25, 0x80]) {
        Ok(bytes) => bytes,
        Err(error) => return Err(error),
    };
    println!("{bytes:?}");
    Ok(bytes[0])
}

fn send(cmd: [u8; 3]) -> std::io::Result<[u8; 32]> {
    let mut stream = TcpStream::connect("192.168.0.2:50001")?;
    let mut result = [0; 32];
    stream.write(&cmd)?;
    stream.read(&mut result)?;
    Ok(result)
}
