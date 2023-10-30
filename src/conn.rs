use serde_json::Value;

pub type FnMatcher = dyn Fn(&str) -> dyn RHandle;

pub trait RHandle {
    fn handle(&mut self, json_data: Value, custom_data: Vec<u8>);
}

pub trait RConnection {
    fn set_matcher(&mut self, matcher: &'static FnMatcher);
}
