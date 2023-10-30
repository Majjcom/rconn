pub mod client;
pub mod server;

pub mod conn;

#[cfg(test)]
mod tests {
    use crate::server;

    #[test]
    fn run_server() {
        let mut s = server::Server::new("0.0.0.0:5000");
        s.start();
    }
}
