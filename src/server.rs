use super::net_service::*;
use crate::conn::*;
use rayon::{ThreadPool, ThreadPoolBuilder};
pub use serde;
use serde::Serialize;
pub use serde_json;
use serde_json::{to_value, Value};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct Server {
    tcp: TcpListener,
    matcher: Option<Arc<Mutex<FnMatcher>>>,
    pool: ThreadPool,
    timeout_dur: Option<Duration>,
}

impl Server {
    pub fn new(addr: &str, threads: usize) -> Server {
        let tcp = TcpListener::bind(addr).unwrap();
        let pool = ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap();
        Server {
            tcp,
            matcher: None,
            pool,
            timeout_dur: None,
        }
    }

    pub fn set_timeout(&mut self, dur: Option<Duration>) -> &mut Self {
        self.timeout_dur = dur;
        self
    }

    pub fn send_data<T: Serialize>(tcp: &mut TcpStream, json_data: T, custom_data: &Vec<u8>) {
        let header = DefaultHeader {
            act: String::from("resp"),
            custom_data_size: custom_data.len(),
            data: to_value(json_data).unwrap(),
        };
        send_data(tcp, &header, custom_data);
    }

    pub fn send_data_value(tcp: &mut TcpStream, json_value: &Value, custom_data: &Vec<u8>) {
        let header = DefaultHeader {
            act: String::from("resp"),
            custom_data_size: custom_data.len(),
            data: json_value.clone(),
        };
        send_data(tcp, &header, custom_data);
    }

    pub fn start(&mut self) {
        for stream in self.tcp.incoming() {
            match &self.matcher {
                Some(matcher) => {
                    let matcher = matcher.clone();
                    if let Err(_) = stream {
                        return ();
                    }
                    let mut s = stream.unwrap();
                    s.set_read_timeout(self.timeout_dur).unwrap();
                    s.set_write_timeout(self.timeout_dur).unwrap();
                    self.pool.spawn(move || loop {
                        let header_size = get_stream_header_size(&mut s);
                        let header_size = match header_size {
                            Ok(s) => s,
                            Err(_) => {
                                s.shutdown(Shutdown::Both).ok();
                                break;
                            }
                        };
                        // header size check
                        if header_size > 64 * 1024 * 1024 {
                            s.shutdown(Shutdown::Both).ok();
                            break;
                        }
                        // Read Start
                        let header_data = get_header_json(&mut s, header_size);
                        let header_data = match header_data {
                            Ok(d) => d,
                            Err(_) => {
                                s.shutdown(Shutdown::Both).ok();
                                break;
                            }
                        };
                        let custom_data = get_custom_data(&mut s, &header_data);
                        let custom_data = match custom_data {
                            Ok(d) => d,
                            Err(_) => {
                                s.shutdown(Shutdown::Both).ok();
                                break;
                            }
                        };
                        // Handle
                        let handle = matcher.lock().unwrap()(&header_data.act);
                        handle
                            .lock()
                            .unwrap()
                            .handle(&mut s, &header_data.data, &custom_data);
                    });
                }
                None => println!("No Handler"),
            }
        }
    }
}

impl RConnection for Server {
    fn set_matcher(&mut self, matcher: &'static FnMatcher) {
        self.matcher = Some(Arc::new(Mutex::new(matcher)));
    }
}
