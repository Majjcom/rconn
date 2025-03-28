use crate::crypto::RCipher;
use crate::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::net::TcpStream;
use std::time::Duration;

struct Handler;

rhandle_impl_new!(Handler);

impl Default for Handler {
    fn default() -> Self {
        Handler {}
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    result: String,
}

impl RHandle for Handler {
    fn handle(&mut self, tcp: &mut TcpStream, data: &Value, _: &Vec<u8>, cipher: &RCipher) {
        let resp = Response {
            result: String::from("OK"),
        };
        let data: Response = serde_json::from_value(data.clone()).unwrap();
        println!("Server {:?} Recv: {:?}", std::thread::current().id(), data);
        let cusd = Vec::new();
        Server::send_data(tcp, &resp, &cusd, cipher);
    }
}

static MAIN_HANDLER: Lazy<THandle> = Lazy::new(|| Handler::new());

fn matcher(act: &str) -> THandle {
    println!("Matcher: match {}", act);
    match act {
        _ => MAIN_HANDLER.clone(),
    }
}

#[test]
fn server_test() {
    use std::thread;
    thread::spawn(|| {
        Server::new("127.0.0.1:5000", 4)
            .set_matcher(&matcher)
            .set_timeout(Some(Duration::from_secs(10)))
            .start();
    });
    thread::sleep(Duration::from_millis(500));
    let client_runner = || {
        let mut client = Client::new("127.0.0.1", 5000, 10000, "").unwrap();
        for _ in 0..10 {
            let res = client
                .request(
                    "act",
                    &Response {
                        result: String::from("aaa"),
                    },
                    &Vec::new(),
                )
                .ok();
            println!("Client Recv: {:?}", res);
        }
    };
    let mut threads = Vec::new();
    for _ in 0..16 {
        threads.push(thread::spawn(client_runner));
    }
    for thread in threads {
        thread.join().unwrap();
    }
}
