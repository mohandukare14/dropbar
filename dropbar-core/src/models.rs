use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeviceType {
    ArchLinux,
    Linux,
    Android,
    IOS,
    Unknown,
}

impl Default for DeviceType {
    fn default() -> Self {
        #[cfg(target_os = "android")]
        return DeviceType::Android;
        #[cfg(target_os = "ios")]
        return DeviceType::IOS;
        #[cfg(target_os = "linux")]
        return DeviceType::ArchLinux;
        #[cfg(not(any(target_os = "android", target_os = "ios", target_os = "linux")))]
        return DeviceType::Unknown;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub device_type: DeviceType,
    pub ip: String,
    pub port: u16,
    pub last_seen: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PayloadType {
    File,
    Text,
    Url,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadMetadata {
    pub id: String,
    pub name: String,
    pub size: u64,
    pub mime_type: Option<String>,
    pub sha256: Option<String>,
    pub payload_type: PayloadType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPayload {
    pub text: String,
    pub is_url: bool,
    pub sender_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferResponse {
    pub success: bool,
    pub message: String,
    pub payload_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BeaconMessage {
    Announce(DeviceInfo),
    Goodbye(String),
}

