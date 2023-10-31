pub mod client;
pub mod server;

pub mod conn;

#[cfg(test)]
mod tests {
    use crate::{
        conn::{RConnection, RHandle},
        server::Server,
    };

    struct Handler;
    impl RHandle for Handler {
        fn handle(&mut self, json_data: serde_json::Value, custom_data: Vec<u8>) {
            println!("Handleing...");
            println!("Json Data: {:?}", json_data);
            println!("Custom Data: {}", String::from_utf8(custom_data).unwrap());
            println!("End Hendle...");
        }
    }

    fn matcher(act: &str) -> Box<dyn RHandle + 'static> {
        println!("Matcher: match {}", act);
        Box::new(Handler {})
    }

    #[test]
    fn server_test() {
        let mut s = Server::new("0.0.0.0:5000");
        s.set_matcher(&matcher);
        s.start();
    }
}
