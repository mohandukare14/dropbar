pub mod client;
pub mod discovery;
pub mod hotspot;
pub mod models;
pub mod server;

pub use client::DropBarClient;
pub use discovery::{DiscoveryService, DEFAULT_BEACON_PORT};
pub use hotspot::{get_arp_neighbors, get_broadcast_addresses, get_default_gateway};
pub use models::{BeaconMessage, DeviceInfo, DeviceType, PayloadMetadata, PayloadType, TextPayload, TransferResponse};
pub use server::{DropBarServer, IncomingPayload};

/// Helper to get local IP address or fallback to 127.0.0.1
pub fn get_local_ip() -> String {
    local_ip_address::local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|_| "127.0.0.1".to_string())
}

/// Helper to get default download directory (~/Downloads/DropBar)
pub fn get_default_download_dir() -> std::path::PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        std::path::PathBuf::from(home).join("Downloads").join("DropBar")
    } else {
        std::path::PathBuf::from("/tmp/DropBar")
    }
}
