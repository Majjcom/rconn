use std::{
    net::TcpStream,
    sync::{Arc, Mutex},
};

use crate::{
    conn::{RConnection, RHandle, THandle},
    rhandle_impl_new,
    server::{
        serde::Serialize,
        serde_json::{to_value, Value},
        Server,
    },
};

struct Handler {
    counter: usize,
}

rhandle_impl_new!(Handler);

impl Default for Handler {
    fn default() -> Self {
        Handler { counter: 0 }
    }
}

#[derive(Serialize)]
struct Response {
    result: String,
}

impl RHandle for Handler {
    fn handle(&mut self, tcp: &mut TcpStream, json_data: &Value, custom_data: &Vec<u8>) {
        println!("Handleing...");
        println!("Json Data: {:?}", json_data);
        println!(
            "Custom Data: {}",
            String::from_utf8(custom_data.clone()).unwrap()
        );
        self.counter += 1;
        let resp = Response {
            result: String::from("OK"),
        };
        let json_data = to_value(resp).unwrap();
        let cusd = Vec::new();
        Server::send_data(tcp, &json_data, &cusd);
        println!("Handle count: {}", &self.counter);
        println!("End Hendle...");
    }
}

use crate::Lazy;

static MAIN_HANDLER: Lazy<THandle> = Lazy::new(|| Handler::new());

fn matcher(act: &str) -> THandle {
    println!("Matcher: match {}", act);
    match act {
        "hello" => MAIN_HANDLER.clone(),
        _ => MAIN_HANDLER.clone(),
    }
}

#[test]
fn server_test() {
    let mut s = Server::new("0.0.0.0:5000");
    s.set_matcher(&matcher);
    s.start();
}
