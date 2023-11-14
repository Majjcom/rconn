use super::net_service::*;
use crate::conn::*;
use rayon::{ThreadPool, ThreadPoolBuilder};
pub use serde;
use serde::Serialize;
pub use serde_json;
use serde_json::to_value;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

pub struct Server {
    tcp: TcpListener,
    matcher: Option<Arc<Mutex<FnMatcher>>>,
    pool: ThreadPool,
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
        }
    }

    pub fn send_data<T: Serialize>(tcp: &mut TcpStream, json_data: T, custom_data: &Vec<u8>) {
        let header = DefaultHeader {
            act: String::from("resp"),
            custom_data_size: custom_data.len(),
            data: to_value(json_data).unwrap(),
        };
        send_data(tcp, &header, custom_data);
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
                                    let handle = matcher.lock().unwrap()(&header_data.act);
                                    handle.lock().unwrap().handle(
                                        &mut s,
                                        &header_data.data,
                                        &custom_data,
                                    );
                                }
                                Err(_) => {
                                    break;
                                }
                            }
                        }),
                        Err(_) => (),
                    }
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
