use std::io::{Read, Result, Write};

pub fn send(stream: &mut (impl Read + Write), cmd: &[u8]) -> Result<[u8; 8]> {
    let mut result = [0; 8];
    stream.write(&cmd)?;
    stream.read(&mut result)?;
    println!("{cmd:?}<->{result:?}");
    Ok(result)
}
