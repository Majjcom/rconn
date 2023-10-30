use crate::conn::*;
use rayon::{ThreadPool, ThreadPoolBuilder};
use serde_json::{de, Value};
use std::io::Read;
use std::net::{TcpListener, TcpStream};

pub struct Server {
    tcp: TcpListener,
    matcher: Option<&'static FnMatcher>,
    pool: ThreadPool,
}

fn get_stream_header_size(s: &mut TcpStream) -> usize {
    let mut buff = Vec::new();
    buff.resize(8, 0u8);
    let mut size = 0;
    while size != 8 {
        let readed_size = s.read(&mut buff[size..]).unwrap();
        size += &readed_size;
    }
    let mut size: usize = 0;
    for i in 0..8 {
        size += buff[i] as usize;
        size <<= 1;
    }
    size
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
            match stream {
                Ok(mut s) => self.pool.spawn(move || loop {
                    // Read Start
                    let header_size = get_stream_header_size(&mut s);
                    let header_data = get_header_json(&mut s, header_size);
                    println!("Data: {:?}", header_data);
                }),
                Err(_) => (),
            }
        }
    }
}

impl RConnection for Server {
    fn set_matcher(&mut self, matcher: &'static FnMatcher) {
        self.matcher = Some(matcher);
    }
}
