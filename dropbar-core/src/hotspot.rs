use crate::models::DeviceInfo;
use std::fs;
use std::net::Ipv4Addr;
use std::process::Command;
use std::time::Duration;
use tracing::debug;

/// Retrieves the active Wi-Fi connection SSID (e.g. "P4")
pub fn get_active_ssid() -> Option<String> {
    if let Ok(output) = Command::new("nmcli")
        .args(["-t", "-f", "active,ssid", "dev", "wifi"])
        .output()
    {
        if let Ok(text) = String::from_utf8(output.stdout) {
            for line in text.lines() {
                if let Some(ssid) = line.strip_prefix("yes:") {
                    let clean = ssid.trim();
                    if !clean.is_empty() {
                        return Some(clean.to_string());
                    }
                }
            }
        }
    }
    None
}

/// Parses /proc/net/route to find the default gateway IP address
pub fn get_default_gateway() -> Option<String> {
    if let Ok(content) = fs::read_to_string("/proc/net/route") {
        for line in content.lines().skip(1) {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() >= 3 {
                let destination = fields[1];
                let gateway_hex = fields[2];

                // 00000000 destination denotes the default route
                if destination == "00000000" && gateway_hex != "00000000" {
                    if let Ok(num) = u32::from_str_radix(gateway_hex, 16) {
                        let ip = Ipv4Addr::from(num.to_be());
                        return Some(ip.to_string());
                    }
                }
            }
        }
    }
    None
}

/// Parses /proc/net/arp to find active neighbor IPs on the local subnet/hotspot
pub fn get_arp_neighbors() -> Vec<String> {
    let mut ips = Vec::new();
    if let Ok(content) = fs::read_to_string("/proc/net/arp") {
        for line in content.lines().skip(1) {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() >= 4 {
                let ip = fields[0];
                let flags = fields[2];
                let hw = fields[3];

                // 0x2 flag means complete/resolved entry
                if flags == "0x2" && hw != "00:00:00:00:00:00" && !ips.contains(&ip.to_string()) {
                    ips.push(ip.to_string());
                }
            }
        }
    }
    ips
}

/// Finds all network interface broadcast addresses
pub fn get_broadcast_addresses() -> Vec<String> {
    let mut broadcasts = vec!["255.255.255.255".to_string()];

    // Common hotspot fallback broadcast addresses
    broadcasts.push("192.168.43.255".to_string()); // Android Hotspot
    broadcasts.push("172.20.10.15".to_string());   // iOS Hotspot (/28 subnet)
    broadcasts.push("172.20.10.255".to_string());  // iOS Hotspot alternative
    broadcasts.push("10.42.0.255".to_string());    // Linux NetworkManager Hotspot default
    broadcasts.push("192.168.12.255".to_string()); // Common hotspot

    broadcasts.dedup();
    broadcasts
}

/// Directly queries a candidate IP (such as a hotspot phone or gateway) for DropBar service
pub async fn probe_candidate(ip: &str, port: u16) -> Option<DeviceInfo> {
    let url = format!("http://{}:{}/api/v1/ping", ip, port);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(400))
        .build()
        .ok()?;

    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(device) = resp.json::<DeviceInfo>().await {
                debug!("Hotspot probe succeeded for {}:{} -> {}", ip, port, device.name);
                return Some(device);
            }
        }
        _ => {}
    }
    None
}
