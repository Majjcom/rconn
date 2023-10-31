use super::net_service::*;
use crate::conn::*;
use rayon::{ThreadPool, ThreadPoolBuilder};
pub use serde;
pub use serde_json;
use serde_json::Value;
use std::net::{TcpListener, TcpStream};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

pub struct Server {
    tcp: TcpListener,
    matcher: Option<Arc<Mutex<FnMatcher>>>,
    pool: ThreadPool,
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

    pub fn send_data(tcp: &mut TcpStream, json_data: &Value, custom_data: &Vec<u8>) {
        let header = DefaultHeader {
            act: String::from_str("resp").unwrap(),
            custom_data_size: custom_data.len(),
            data: json_data.clone(),
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
