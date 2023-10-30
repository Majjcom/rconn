use crate::conn::*;
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::io::{Read, Write};
use std::net::TcpListener;

pub struct Server {
    tcp: TcpListener,
    matcher: Option<&'static FnMatcher>,
    pool: ThreadPool,
}

impl Server {
    fn new(addr: &str) -> Server {
        let tcp = TcpListener::bind(addr).unwrap();
        let pool = ThreadPoolBuilder::new().num_threads(16).build().unwrap();
        Server {
            tcp,
            matcher: None,
            pool,
        }
    }

    fn start(&mut self) {
        for stream in self.tcp.incoming() {
            match stream {
                Ok(s) => self.pool.spawn(move || loop {
                    let mut buff: [u8; 8];
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
