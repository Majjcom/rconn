use std::sync::{Arc, Mutex};

use serde_json::Value;

pub type FnMatcher = dyn Fn(&str) -> THandle + Send + Sync;

pub type THandle = Arc<Mutex<Box<dyn RHandle + 'static>>>;

pub trait RHandle: Send {
    fn handle(&mut self, json_data: Value, custom_data: Vec<u8>);
}

pub trait RConnection {
    fn set_matcher(&mut self, matcher: &'static FnMatcher);
}
