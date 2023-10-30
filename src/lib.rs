pub mod client;
pub mod server;

pub mod conn;

#[cfg(test)]
mod tests {
    use crate::server::Server;

    #[test]
    fn server_test() {
        let mut s = Server::new("0.0.0.0:5000");
        s.start();
    }
}
