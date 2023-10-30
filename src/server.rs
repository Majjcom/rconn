use crate::conn::*;
use std::net::TcpListener;

mod pool;

pub struct Server {
    tcp: TcpListener,
    matcher: Option<&'static FnMatcher>,
}

impl Server {
    fn new(addr: &str) -> Server {
        let tcp = TcpListener::bind(addr).unwrap();
        Server { tcp, matcher: None }
    }

    fn start(&mut self) {}
}

impl RConnection for Server {
    fn set_matcher(&mut self, matcher: &'static FnMatcher) {
        self.matcher = Some(matcher);
    }
}
