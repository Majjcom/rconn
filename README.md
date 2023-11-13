# Rconn

This is a simple network service with its own protocol.


## Usage

### Create a server

```rust
use std::{
    net::TcpStream,
    sync::{Arc, Mutex},
};

use rconn::{
    conn::{RConnection, RHandle, THandle},
    rhandle_impl_new,
    server::{
        serde::Serialize,
        serde_json::{to_value, Value},
        Server,
    },
    Lazy,
};

struct Handler;

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
        let resp = Response {
            result: String::from("OK"),
        };
        let json_data = to_value(resp).unwrap();
        let cusd = Vec::new();
        Server::send_data(tcp, &json_data, &cusd);
        println!("End Hendle...");
    }
}

static MAIN_HANDLER: Lazy<THandle> = Lazy::new(|| Handler::new());

fn matcher(act: &str) -> THandle {
    match act {
        _ => MAIN_HANDLER.clone(),
    }
}

fn main() {
    let mut s = Server::new("0.0.0.0:5000");
    s.set_matcher(&matcher);
    s.start();
}

```

### Client

```rust
use crate::client::{
    serde::{Deserialize, Serialize},
    serde_json::to_value,
    Client,
};

#[derive(Serialize, Deserialize)]
struct Test;

let mut client = Client::new("127.0.0.1", 5000, 10000).unwrap();
let ndata = to_value(Test {}).unwrap();
let cusd = Vec::new();
let readed = client.request("test", &ndata, &cusd).unwrap();
println!("Get: {:?}", readed);
```
