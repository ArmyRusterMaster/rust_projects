use std::fs::File;
use std::io::{self, Read};

pub fn get_random_byte() -> io::Result<u8> {
    let mut file = File::open("/dev/urandom")?;
    let mut buf = [0u8; 1];
    file.read_exact(&mut buf)?;
    Ok(buf[0])
}

pub fn get_random_index(max: usize) -> io::Result<usize> {
    if max == 0 {
        return Ok(0);
    }
    let byte = get_random_byte()? as usize;
    Ok(byte % max)
}

