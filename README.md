# Rconn

This is a simple network service with its own protocol.

## Usage

### Create a server

```rust
use std::net::TcpStream;
use rconn::prelude::*;
use serde::Serialize;
use serde_json::Value;

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
    fn handle(&mut self, tcp: &mut TcpStream, _: &Value, _: &Vec<u8>, cipher: &RCipher) {
        let resp = Response {
            result: String::from("OK"),
        };
        let cusd = Vec::new();
        Server::send_data(tcp, &resp, &cusdm, cipher);
    }
}

static MAIN_HANDLER: Lazy<THandle> = Lazy::new(|| Handler::new());

fn matcher(act: &str) -> THandle {
    println!("Matcher: match {}", act);
    match act {
        _ => MAIN_HANDLER.clone(),
    }
}

fn main() {
    let mut s = Server::new("0.0.0.0:5000", 16);
    s.set_matcher(&matcher);
    s.start();
}
```

### Client

```rust
use crate::client::{
    serde::{Deserialize, Serialize},
    Client,
};

#[derive(Serialize, Deserialize)]
struct Test;

fn main() {
    let mut client = Client::new("127.0.0.1", 5000, 10000, "").unwrap();
    let ndata = Test {};
    let cusd = Vec::new();
    let readed = client.request("test", &ndata, &cusd).unwrap();
    println!("Get: {:?}", readed);
}
```
