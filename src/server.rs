use crate::conn::*;
use rayon::{ThreadPool, ThreadPoolBuilder};
use serde_json::{de, Value};
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

pub struct Server {
    tcp: TcpListener,
    matcher: Option<Arc<Mutex<FnMatcher>>>,
    pool: ThreadPool,
}

fn get_stream_header_size(s: &mut TcpStream) -> Result<usize, std::io::Error> {
    let mut buff = Vec::new();
    buff.resize(8, 0u8);
    let mut size = 0;
    while size != 8 {
        match s.read(&mut buff[size..]) {
            Ok(readed_size) if readed_size > 0 => size += readed_size,
            Ok(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Connection closed",
                ))
            }
            Err(e) => return Err(e),
        }
    }
    let mut size: usize = 0;
    for i in 0..7 {
        size += buff[i] as usize;
        size <<= 1;
    }
    size += buff[7] as usize;
    Ok(size)
}

fn get_header_json(s: &mut TcpStream, header_size: usize) -> Value {
    let mut header_buffer = Vec::new();
    header_buffer.resize(header_size, 0u8);
    let mut size = 0;
    while size != header_size {
        let readed_size = s.read(&mut header_buffer[size..]).unwrap();
        size += &readed_size;
    }
    de::from_slice(&header_buffer).unwrap()
}

fn get_custom_data(s: &mut TcpStream, header: &Value) -> Vec<u8> {
    let val_size = header.get("custom_data_size").unwrap();
    let size = val_size.as_u64().unwrap() as usize;
    let mut data = Vec::<u8>::new();
    let mut buffer: [u8; 4096] = [0; 4096];
    while data.len() != size {
        let readed_size = s.read(buffer.as_mut_slice()).unwrap();
        data.extend(&buffer[..readed_size]);
    }
    data
}

impl Server {
    pub fn new(addr: &str) -> Server {
        let tcp = TcpListener::bind(addr).unwrap();
        let pool = ThreadPoolBuilder::new().num_threads(16).build().unwrap();
        Server {
            tcp,
            matcher: None,
            pool,
        }
    }

    pub fn start(&mut self) {
        for stream in self.tcp.incoming() {
            match &self.matcher {
                Some(matcher) => {
                    let matcher = matcher.clone();
                    match stream {
                        Ok(mut s) => self.pool.spawn(move || loop {
                            match get_stream_header_size(&mut s) {
                                Ok(header_size) => {
                                    // Read Start
                                    let header_data = get_header_json(&mut s, header_size);
                                    let custom_data = get_custom_data(&mut s, &header_data);
                                    // Handle
                                    let act = header_data.get("act").unwrap().as_str().unwrap();
                                    let handle = matcher.lock().unwrap()(act);
                                    handle.lock().unwrap().handle(header_data, custom_data);
                                }
                                Err(_) => {
                                    println!("Connrction closed...");
                                    break;
                                }
                            }
                        }),
                        Err(_) => (),
                    }
                }
                None => println!("No Handler"),
            }
            println!("One Connection Entered...");
        }
    }
}

impl RConnection for Server {
    fn set_matcher(&mut self, matcher: &'static FnMatcher) {
        self.matcher = Some(Arc::new(Mutex::new(matcher)));
    }
}
