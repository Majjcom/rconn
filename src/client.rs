use crate::config;
use crate::crypto::RCipher;
use crate::net_service::*;
pub use serde;
use serde::Serialize;
pub use serde_json;
use serde_json::{to_value, Value};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
use std::str::FromStr;
use std::time::Duration;

pub struct Client {
    tcp: TcpStream,
    max_stream_header_size: u64,
    max_stream_size: u64,
    cipher: RCipher,
}

#[derive(Debug)]
pub struct ReadContent {
    pub data: Value,
    pub custom_data: Vec<u8>,
    pub act: String,
}

impl Client {
    pub fn new(
        addr: &str,
        port: u16,
        timeout: u32,
        const_key: &str,
    ) -> Result<Client, std::io::Error> {
        let addr = match addr.parse::<Ipv4Addr>() {
            Ok(a) => a,
            Err(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::AddrNotAvailable,
                    "Parse Address Failed.",
                ))
            }
        };
        let addr = SocketAddr::V4(SocketAddrV4::new(addr, port));
        let mut tcp = TcpStream::connect_timeout(&addr, Duration::from_millis(timeout as u64))?;
        tcp.set_read_timeout(Some(Duration::from_millis(timeout as u64)))?;
        tcp.set_write_timeout(Some(Duration::from_millis(timeout as u64)))?;

        let key = if let Ok(r) = get_stream_key(&mut tcp) {
            r
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Get key Failed.",
            ));
        };
        let cipher = if let Ok(r) = RCipher::new(&key, const_key.as_bytes()) {
            r
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Create Cipher Failed.",
            ));
        };
        Ok(Client {
            tcp,
            max_stream_header_size: config::DEFAULT_MAX_STREAM_HEADER_SIZE,
            max_stream_size: config::DEFAULT_MAX_STREAM_SIZE,
            cipher,
        })
    }

    pub fn send<T: Serialize>(&mut self, act: &str, json_data: &T, custom_data: &Vec<u8>) {
        let mut cuslen = custom_data.len();
        if cuslen > 0 {
            cuslen = cuslen + (16 - cuslen % 16);
        }
        let header = DefaultHeader {
            act: String::from_str(act).unwrap(),
            custom_data_size: cuslen,
            data: to_value(json_data).unwrap(),
        };
        send_data(&mut self.tcp, &header, custom_data, &self.cipher);
    }

    pub fn request<T: Serialize>(
        &mut self,
        act: &str,
        json_data: &T,
        custom_data: &Vec<u8>,
    ) -> Result<ReadContent, ()> {
        self.send(act, json_data, custom_data);
        self.read()
    }

    pub fn read(&mut self) -> Result<ReadContent, ()> {
        let stream_max = self.max_stream_size;
        let header_max = self.max_stream_header_size;
        let read = match get_stream_header_size(&mut self.tcp) {
            Ok(header_size) => {
                if header_size as u64 > header_max {
                    return Err(());
                }
                let header_data = get_header_json(&mut self.tcp, header_size, &self.cipher);
                let header_data = match header_data {
                    Ok(d) => d,
                    Err(_) => return Err(()),
                };
                let custom_data =
                    get_custom_data(&mut self.tcp, &header_data, stream_max, &self.cipher);
                let custom_data = match custom_data {
                    Ok(d) => d,
                    Err(_) => return Err(()),
                };
                ReadContent {
                    custom_data,
                    data: header_data.data,
                    act: header_data.act,
                }
            }
            Err(_) => return Err(()),
        };
        Ok(read)
    }
}
