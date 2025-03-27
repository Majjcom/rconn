use std::net::TcpStream;
pub use std::sync::{Arc, Mutex};

use serde_json::Value;

pub type FnMatcher = dyn Fn(&str) -> THandle + Send + Sync;

pub type THandle = Arc<Mutex<Box<dyn RHandle>>>;

pub trait RHandle: Send {
    fn handle(&mut self, tcp: &mut TcpStream, json_data: &Value, custom_data: &Vec<u8>);
}

#[macro_export]
macro_rules! rhandle_impl_new {
    ($class:ident) => {
        impl $class {
            pub fn new() -> THandle {
                std::sync::Arc::new(std::sync::Mutex::new(Box::new($class::default())))
            }
        }
    };
}

pub trait RConnection {
    fn set_matcher(&mut self, matcher: &'static FnMatcher) -> &mut Self;
}
