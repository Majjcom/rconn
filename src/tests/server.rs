use std::{
    net::TcpStream,
    sync::{Arc, Mutex},
};

use crate::{
    conn::{RConnection, RHandle, THandle},
    rhandle_impl_new,
    server::{
        serde::Serialize,
        serde_json::Value,
        Server,
    },
};

struct Handler;

rhandle_impl_new!(Handler);

impl Default for Handler {
    fn default() -> Self {
        Handler {}
    }
}

#[derive(Serialize)]
struct Response {
    result: String,
}

impl RHandle for Handler {
    fn handle(&mut self, tcp: &mut TcpStream, json_data: &Value, custom_data: &Vec<u8>) {
        let resp = Response {
            result: String::from("OK"),
        };
        let cusd = Vec::new();
        Server::send_json_data(tcp, &resp, &cusd);
    }
}

use crate::Lazy;

static MAIN_HANDLER: Lazy<THandle> = Lazy::new(|| Handler::new());

fn matcher(act: &str) -> THandle {
    println!("Matcher: match {}", act);
    match act {
        _ => MAIN_HANDLER.clone(),
    }
}

#[test]
fn server_test() {
    let mut s = Server::new("0.0.0.0:5000", 16);
    s.set_matcher(&matcher);
    s.start();
}
