pub mod client;
pub mod conn;
pub mod server;
pub use once_cell::sync::Lazy;
pub use serde_json;
mod net_service;

#[cfg(test)]
mod tests {
    mod client;
    mod server;
    

    
}
