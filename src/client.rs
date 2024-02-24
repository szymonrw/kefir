use std::io::{Read, Write};
use std::{io::Result, net::TcpStream};

pub fn send(stream: &mut TcpStream, cmd: &[u8]) -> Result<[u8; 8]> {
    let mut result = [0; 8];
    stream.write(&cmd)?;
    stream.read(&mut result)?;
    println!("{cmd:?}<->{result:?}");
    Ok(result)
}
