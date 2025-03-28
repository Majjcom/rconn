use super::crypto::RCipher;
use super::net_service::*;
use crate::config;
use crate::conn::*;
use log::{debug, error, info, warn};
pub use serde;
use serde::Serialize;
pub use serde_json;
use serde_json::{to_value, Value};
use std::boxed::Box;
use std::io::ErrorKind::Interrupted;
use std::io::Write;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::runtime::Runtime;

pub struct Server {
    tcp: TcpListener,
    const_key: String,
    matcher: Option<Arc<Mutex<FnMatcher>>>,
    // pool: ThreadPool,
    tko_runtime: Arc<Box<Runtime>>,
    timeout_dur: Option<Duration>,
    max_stream_header_size: u64,
    max_stream_size: u64,
}

impl Server {
    pub fn new(addr: &str, threads: usize) -> Server {
        let tcp = TcpListener::bind(addr).unwrap();
        // let pool = ThreadPoolBuilder::new()
        //     .num_threads(threads)
        //     .build()
        //     .unwrap();
        let tko_runtime = Arc::new(Box::new(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .worker_threads(threads)
                .build()
                .unwrap(),
        ));
        Server {
            tcp,
            matcher: None,
            const_key: String::new(),
            tko_runtime,
            timeout_dur: None,
            max_stream_header_size: config::DEFAULT_MAX_STREAM_HEADER_SIZE,
            max_stream_size: config::DEFAULT_MAX_STREAM_SIZE,
        }
    }

    pub fn set_max_stream_header_size(&mut self, size: u64) -> &mut Self {
        self.max_stream_header_size = size;
        debug!("Set max stream header size to {}", size);
        self
    }

    pub fn set_max_stream_size(&mut self, size: u64) -> &mut Self {
        self.max_stream_size = size;
        debug!("Set max stream size to {}", size);
        self
    }

    pub fn set_timeout(&mut self, dur: Option<Duration>) -> &mut Self {
        self.timeout_dur = dur;
        debug!("set network timeout to {:?}", dur);
        self
    }

    pub fn set_const_key(&mut self, key: &str) -> &mut Self {
        self.const_key = String::from(key);
        debug!("Set const key to {}", key);
        self
    }

    pub fn send_data<T: Serialize>(
        tcp: &mut TcpStream,
        json_data: T,
        custom_data: &Vec<u8>,
        cipher: &RCipher,
    ) {
        let mut cuslen = custom_data.len();
        if cuslen > 0 {
            cuslen = cuslen + (16 - cuslen % 16);
        }
        let header = DefaultHeader {
            act: String::from("resp"),
            custom_data_size: cuslen,
            data: to_value(json_data).unwrap(),
        };
        send_data(tcp, &header, custom_data, cipher);
    }

    pub fn send_data_value(
        tcp: &mut TcpStream,
        json_value: &Value,
        custom_data: &Vec<u8>,
        cipher: &RCipher,
    ) {
        let mut cuslen = custom_data.len();
        if cuslen > 0 {
            cuslen = cuslen + (16 - cuslen % 16);
        }
        let header = DefaultHeader {
            act: String::from("resp"),
            custom_data_size: cuslen,
            data: json_value.clone(),
        };
        send_data(tcp, &header, custom_data, cipher);
    }

    pub fn start(&mut self) {
        let tko_runtime = self.tko_runtime.clone();
        tko_runtime.block_on(async {
            self._start().await;
        });
    }

    async fn _start(&mut self) {
        let stream_header_max = self.max_stream_header_size;
        let stream_max = self.max_stream_size;
        for stream in self.tcp.incoming() {
            // Get Matcher
            let matcher = if let Some(matcher) = &self.matcher {
                matcher.clone()
            } else {
                info!("Matcher not matched of {:?}", stream);
                continue;
            };
            if let Err(_) = stream {
                error!("tcpstream {:?} can't clone stream.", stream);
                continue;
            }

            // Get stream
            let mut s = stream.unwrap();
            s.set_read_timeout(self.timeout_dur).unwrap();
            s.set_write_timeout(self.timeout_dur).unwrap();

            let const_key = self.const_key.clone();

            // Worker
            let func = async move {
                // Exchange key
                // Generate key
                let generated_key = RCipher::gen_key();
                debug!(
                    "Generated key: {}.",
                    String::from_utf8_lossy(&generated_key)
                );

                // Send key
                if let Ok(r) = s.write(&generated_key) {
                    debug!("Send Key success: {}.", r);
                    s.flush().unwrap();
                } else {
                    error!("Send key failed.");
                    s.shutdown(Shutdown::Both).ok();
                    return;
                }

                // Create cipher
                let cipher = if let Ok(r) = RCipher::new(&generated_key, const_key.as_bytes()) {
                    r
                } else {
                    error!("Create RCipher Failed.");
                    s.shutdown(Shutdown::Both).ok();
                    return;
                };

                // Worker loop
                loop {
                    /*
                    stream structer:
                        header size[4Byte]
                        heaser data[?Bytes][Json]
                        byte data[?Bytes]
                    */
                    let header_size = get_stream_header_size(&mut s);
                    let header_size = match header_size {
                        Ok(s) => s,
                        Err(e) => {
                            if e.kind() == Interrupted {
                                debug!("Connection closed");
                            } else {
                                debug!("Can't read header size");
                            }
                            s.shutdown(Shutdown::Both).ok();
                            break;
                        }
                    };
                    // header size check
                    if header_size as u64 > stream_header_max {
                        s.shutdown(Shutdown::Both).ok();
                        warn!(
                            "connection header size({}) of {:?} is out of range",
                            header_size, s
                        );
                        break;
                    }
                    // Read Start
                    let header_data = get_header_json(&mut s, header_size, &cipher);
                    let header_data = match header_data {
                        Ok(d) => d,
                        Err(_) => {
                            s.shutdown(Shutdown::Both).ok();
                            debug!("Fail to read header of {:?}", s);
                            break;
                        }
                    };
                    let custom_data = get_custom_data(&mut s, &header_data, stream_max, &cipher);
                    let custom_data = match custom_data {
                        Ok(d) => d,
                        Err(_) => {
                            s.shutdown(Shutdown::Both).ok();
                            debug!("Fail to read stream data of {:?}", s);
                            break;
                        }
                    };
                    // Handle
                    let handle = matcher.lock().unwrap()(&header_data.act);
                    handle
                        .lock()
                        .unwrap()
                        .handle(&mut s, &header_data.data, &custom_data, &cipher);
                }
            };
            self.tko_runtime.spawn(func);
        }
    }
}

impl RConnection for Server {
    fn set_matcher(&mut self, matcher: &'static FnMatcher) -> &mut Self {
        self.matcher = Some(Arc::new(Mutex::new(matcher)));
        self
    }
}
