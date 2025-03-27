pub mod client;
pub mod conn;
pub mod server;
pub use once_cell::sync::Lazy;
pub use serde_json;
pub mod config;
mod net_service;
mod prelude;

#[cfg(test)]
mod tests {
    mod client;
    mod server;
}
