use crate::hotspot::{get_active_ssid, get_arp_neighbors, get_broadcast_addresses, get_default_gateway, probe_candidate};
use crate::models::{BeaconMessage, DeviceInfo, DeviceType};
use chrono::Utc;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::sync::{broadcast, RwLock};
use tracing::{error, info};

pub const DEFAULT_BEACON_PORT: u16 = 53317;

#[derive(Clone)]
pub struct DiscoveryService {
    local_device: DeviceInfo,
    beacon_port: u16,
    peers: Arc<RwLock<HashMap<String, DeviceInfo>>>,
    event_sender: broadcast::Sender<DeviceInfo>,
}

impl DiscoveryService {
    pub fn new(local_device: DeviceInfo, beacon_port: u16) -> Self {
        let (event_sender, _) = broadcast::channel(32);
        Self {
            local_device,
            beacon_port,
            peers: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DeviceInfo> {
        self.event_sender.subscribe()
    }

    pub fn register_peer(&self, device: DeviceInfo) {
        let peers_clone = self.peers.clone();
        let event_tx = self.event_sender.clone();
        tokio::spawn(async move {
            let mut map = peers_clone.write().await;
            map.insert(device.id.clone(), device.clone());
            let _ = event_tx.send(device);
        });
    }

    pub async fn get_peers(&self) -> Vec<DeviceInfo> {
        let now = Utc::now();
        let mut peers = self.peers.write().await;
        // Purge peers not seen in the last 20 seconds
        peers.retain(|_, dev| (now - dev.last_seen).num_seconds() < 20);
        peers.values().cloned().collect()
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let listen_addr: SocketAddr = format!("0.0.0.0:{}", self.beacon_port).parse()?;
        let socket = UdpSocket::bind(listen_addr).await?;
        socket.set_broadcast(true)?;

        let socket_recv = Arc::new(socket);
        let socket_send = socket_recv.clone();

        let peers_clone = self.peers.clone();
        let local_id = self.local_device.id.clone();
        let event_tx = self.event_sender.clone();

        // 1. Inbound UDP listener task
        tokio::spawn(async move {
            let mut buf = vec![0u8; 4096];
            loop {
                match socket_recv.recv_from(&mut buf).await {
                    Ok((len, remote_addr)) => {
                        let data = &buf[..len];
                        if let Ok(beacon) = serde_json::from_slice::<BeaconMessage>(data) {
                            match beacon {
                                BeaconMessage::Announce(mut device) => {
                                    if device.id != local_id {
                                        device.ip = remote_addr.ip().to_string();
                                        device.last_seen = Utc::now();

                                        let mut map = peers_clone.write().await;
                                        let is_new = !map.contains_key(&device.id);
                                        map.insert(device.id.clone(), device.clone());
                                        drop(map);

                                        if is_new {
                                            info!(
                                                "Discovered DropBar peer: {} ({}) at {}:{}",
                                                device.name, device.id, device.ip, device.port
                                            );
                                            let _ = event_tx.send(device);
                                        }
                                    }
                                }
                                BeaconMessage::Goodbye(device_id) => {
                                    let mut map = peers_clone.write().await;
                                    map.remove(&device_id);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Discovery recv error: {}", e);
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    }
                }
            }
        });

        // 2. Outbound multi-broadcast task (supporting Hotspot subnets)
        let local_dev = self.local_device.clone();
        let beacon_port = self.beacon_port;

        tokio::spawn(async move {
            loop {
                let mut dev = local_dev.clone();
                dev.last_seen = Utc::now();
                let msg = BeaconMessage::Announce(dev);

                if let Ok(json) = serde_json::to_vec(&msg) {
                    for broadcast_ip in get_broadcast_addresses() {
                        if let Ok(target_addr) = format!("{}:{}", broadcast_ip, beacon_port).parse::<SocketAddr>() {
                            let _ = socket_send.send_to(&json, target_addr).await;
                        }
                    }
                }

                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        });

        // 3. Hotspot SSID & Gateway Auto-Detection + Active Probe
        let peers_hotspot = self.peers.clone();
        let event_tx_hotspot = self.event_sender.clone();
        let local_id_hotspot = self.local_device.id.clone();

        tokio::spawn(async move {
            loop {
                // Check if we are connected to a hotspot by checking SSID and gateway
                let active_ssid = get_active_ssid();
                let default_gw = get_default_gateway();

                if let (Some(ssid), Some(gw)) = (active_ssid.as_ref(), default_gw.as_ref()) {
                    // Register the Hotspot host (e.g. "P4") so it appears in DropBar immediately!
                    let hotspot_device = DeviceInfo {
                        id: format!("hotspot_{}", ssid),
                        name: ssid.clone(),
                        device_type: DeviceType::Android,
                        ip: gw.clone(),
                        port: 53319,
                        last_seen: Utc::now(),
                    };

                    let mut map = peers_hotspot.write().await;
                    let is_new = !map.contains_key(&hotspot_device.id);
                    map.insert(hotspot_device.id.clone(), hotspot_device.clone());
                    drop(map);

                    if is_new {
                        info!("⚡ Hotspot SSID '{}' detected at gateway {} - Registered as ready peer", ssid, gw);
                        let _ = event_tx_hotspot.send(hotspot_device);
                    }
                }

                // Also actively probe candidates
                let mut candidate_ips = Vec::new();
                if let Some(gw) = default_gw {
                    candidate_ips.push(gw);
                }
                candidate_ips.push("192.168.43.1".to_string());
                candidate_ips.push("172.20.10.1".to_string());

                for neighbor in get_arp_neighbors() {
                    if !candidate_ips.contains(&neighbor) {
                        candidate_ips.push(neighbor);
                    }
                }

                for ip in &candidate_ips {
                    for port in [53319, 53318] {
                        if let Some(mut device) = probe_candidate(ip, port).await {
                            if device.id != local_id_hotspot {
                                device.ip = ip.clone();
                                device.last_seen = Utc::now();

                                let mut map = peers_hotspot.write().await;
                                let is_new = !map.contains_key(&device.id);
                                map.insert(device.id.clone(), device.clone());
                                drop(map);

                                if is_new {
                                    info!(
                                        "⚡ Hotspot active probe connected to: {} ({}) at {}:{}",
                                        device.name, device.id, device.ip, device.port
                                    );
                                    let _ = event_tx_hotspot.send(device);
                                }
                                break;
                            }
                        }
                    }
                }

                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        });

        Ok(())
    }
}
