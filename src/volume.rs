use std::cmp;
use std::io::Result;
use std::net::TcpStream;

use crate::client::send;

pub fn change_volume(stream: &mut TcpStream, amount: i8) -> Result<u8> {
    let mut volume = get_volume(stream)?;
    volume = if amount < 0 {
        volume - cmp::min(-amount as u8, volume)
    } else {
        volume + cmp::min(amount as u8, 100 - volume)
    };
    set_volume(stream, volume)?;
    Ok(volume)
}

pub fn get_volume(stream: &mut TcpStream) -> std::io::Result<u8> {
    let bytes = send(stream, &[0x47, 0x25, 0x80])?;
    Ok(bytes[3])
}

pub fn set_volume(stream: &mut TcpStream, volume: u8) -> std::io::Result<u8> {
    let corrected = volume.clamp(0, 100);
    send(stream, &[0x53, 0x25, 0x81, corrected])?;
    Ok(corrected)
}
