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
}

#[derive(Debug)]
pub struct ReadContent {
    pub data: Value,
    pub custom_data: Vec<u8>,
    pub act: String,
}

impl Client {
    pub fn new(addr: &str, port: u16, timeout: u32) -> Result<Client, std::io::Error> {
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
        let tcp = match TcpStream::connect_timeout(&addr, Duration::from_millis(timeout as u64)) {
            Ok(t) => t,
            Err(e) => {
                return Err(e);
            }
        };
        Ok(Client { tcp })
    }

    pub fn send<T: Serialize>(&mut self, act: &str, json_data: &T, custom_data: &Vec<u8>) {
        let header = DefaultHeader {
            act: String::from_str(act).unwrap(),
            custom_data_size: custom_data.len(),
            data: to_value(json_data).unwrap(),
        };
        send_data(&mut self.tcp, &header, custom_data);
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
        let read = match get_stream_header_size(&mut self.tcp) {
            Ok(header_size) => {
                let header_data = get_header_json(&mut self.tcp, header_size);
                let header_data = match header_data {
                    Ok(d) => d,
                    Err(_) => return Err(()),
                };
                let custom_data = get_custom_data(&mut self.tcp, &header_data);
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
