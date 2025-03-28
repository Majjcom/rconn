use crate::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Test;

// #[test]
fn _cilent_test() {
    let mut client = Client::new("127.0.0.1", 5000, 10000, "").unwrap();
    let ndata = Test {};
    let cusd = Vec::new();
    let readed = client.request("test", &ndata, &cusd).unwrap();
    println!("Get: {:?}", readed);
}
