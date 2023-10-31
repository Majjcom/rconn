use crate::client::{
    serde::{Deserialize, Serialize},
    serde_json::to_value,
    Client,
};

#[derive(Serialize, Deserialize)]
struct Test;

#[test]
fn cilent_test() {
    let mut client = Client::new("127.0.0.1", 5000, 10000).unwrap();
    let ndata = to_value(Test {}).unwrap();
    let cusd = Vec::new();
    let readed = client.request("test", &ndata, &cusd).unwrap();
    println!("Get: {:?}", readed);
}
