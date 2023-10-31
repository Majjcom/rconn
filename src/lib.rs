pub mod client;
pub mod conn;
pub mod server;
pub use once_cell::sync::Lazy;

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::{
        conn::{RConnection, RHandle, THandle},
        rhandle_impl_new,
        server::Server,
    };

    struct Handler {
        counter: usize,
    }

    rhandle_impl_new!(Handler);
    // impl Handler {
    //     fn new() -> THandle {
    //         Arc::new(Mutex::new(Box::new(Handler { counter: 0 })))
    //     }
    // }

    impl Default for Handler {
        fn default() -> Self {
            Handler { counter: 0 }
        }
    }

    impl RHandle for Handler {
        fn handle(&mut self, json_data: serde_json::Value, custom_data: Vec<u8>) {
            println!("Handleing...");
            println!("Json Data: {:?}", json_data);
            println!("Custom Data: {}", String::from_utf8(custom_data).unwrap());
            self.counter += 1;
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
}
