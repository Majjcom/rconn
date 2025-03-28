use super::crypto::RCipher;
use log::warn;
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, to_string, Value};
use std::io::{Read, Write};
use std::net::TcpStream;

pub fn get_stream_header_size(s: &mut TcpStream) -> Result<u32, std::io::Error> {
    let mut buff = Vec::new();
    buff.resize(4, 0u8);
    let mut size = 0;
    while size != 4 {
        match s.read(&mut buff[size..]) {
            Ok(read_size) if read_size > 0 => size += read_size,
            Ok(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    "Connection closed",
                ))
            }
            Err(e) => return Err(e),
        }
    }
    let mut size: u32 = 0;
    for i in 0..3 {
        size += buff[i] as u32;
        size <<= 8;
    }
    size += buff[3] as u32;
    Ok(size)
}

pub fn get_header_json(
    s: &mut TcpStream,
    header_size: u32,
    cipher: &RCipher,
) -> Result<DefaultHeader, ()> {
    let mut header_buffer = Vec::new();
    header_buffer.resize(header_size as usize, 0u8);
    let mut size = 0;
    while size != header_size {
        let read_size = s.read(&mut header_buffer[size as usize..]);
        let read_size = match read_size {
            Ok(r) if r > 0 => r,
            Ok(_) => return Err(()),
            Err(_) => return Err(()),
        };
        size += read_size as u32;
    }
    let data = if let Ok(r) = cipher.decript_data(&header_buffer) {
        r
    } else {
        return Err(());
    };
    let json = from_slice(&data);
    if let Ok(j) = json {
        Ok(j)
    } else {
        Err(())
    }
}

pub fn get_custom_data(
    s: &mut TcpStream,
    header: &DefaultHeader,
    max_size: u64,
    cipher: &RCipher,
) -> Result<Vec<u8>, ()> {
    let size = header.custom_data_size;
    if size as u64 > max_size {
        warn!("stream data size({}) of {:?} is out of range", size, s);
        return Err(());
    }
    let mut data = Vec::<u8>::new();
    let mut buffer: [u8; 4096] = [0; 4096];
    while data.len() != size {
        let rest = size - data.len();
        let end = if rest > 4096 { 4096 } else { rest };
        let read_size = s.read(&mut buffer[..end]);
        let read_size = match read_size {
            Ok(r) if r > 0 => r,
            Ok(_) => return Err(()),
            Err(_) => return Err(()),
        };
        let buffer_decoded = cipher.decript_data(&buffer[..read_size]);
        if let Ok(r) = buffer_decoded {
            data.extend(r);
        } else {
            return Err(());
        }
    }
    Ok(data)
}

pub fn send_data(s: &mut TcpStream, header: &DefaultHeader, custom_data: &Vec<u8>) {
    let json_data = to_string(header).unwrap();
    let lenj = json_data.len();
    let mut lenb = [0u8; 4];
    for i in 0..4 {
        lenb[i] = ((lenj >> (3 - i) * 8) & 0xFF) as u8;
    }
    s.write(&lenb).unwrap();
    s.write(json_data.as_bytes()).unwrap();
    s.write(&custom_data).unwrap();
}

#[derive(Serialize, Deserialize)]
pub struct DefaultHeader {
    pub act: String,
    pub custom_data_size: usize,
    pub data: Value,
}
