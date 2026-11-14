use clap::{Parser, Subcommand};
use colored::*;
use dropbar_core::{
    get_default_download_dir, get_local_ip, DeviceInfo, DeviceType, DiscoveryService, DropBarClient,
    DropBarServer, IncomingPayload, DEFAULT_BEACON_PORT,
};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "dropbar-cli")]
#[command(about = "DropBar CLI - Transfer files between Arch Linux and Mobile devices seamlessly", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the background receiver daemon and peer announcer
    Daemon {
        #[arg(short, long, default_value = "DropBar-Arch")]
        name: String,
        #[arg(short, long, default_value_t = 53318)]
        port: u16,
        #[arg(short, long)]
        save_dir: Option<PathBuf>,
    },
    /// Scan and list available DropBar peers on the local network
    Peers {
        #[arg(short, long, default_value_t = 3)]
        wait_seconds: u64,
    },
    /// Send a file or text payload to a remote device
    Send {
        /// Target device IP or partial name
        target: String,
        /// Path to the file or text message to send
        payload: String,
        #[arg(short, long, default_value_t = 53318)]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Daemon { name, port, save_dir } => {
            let save_path = save_dir.unwrap_or_else(get_default_download_dir);
            let local_ip = get_local_ip();
            let device_id = uuid::Uuid::new_v4().to_string();

            let device_info = DeviceInfo {
                id: device_id.clone(),
                name: name.clone(),
                device_type: DeviceType::ArchLinux,
                ip: local_ip.clone(),
                port,
                last_seen: chrono::Utc::now(),
            };

            println!("{}", "═════════════════════════════════════════════════════".cyan());
            println!("  🚀 DropBar Daemon Started on Arch Linux");
            println!("  📌 Device: {} ({})", name.green().bold(), device_id);
            println!("  🌐 Address: http://{}:{}", local_ip, port);
            println!("  📂 Download Directory: {}", save_path.display().to_string().yellow());
            println!("{}", "═════════════════════════════════════════════════════".cyan());

            // 1. Start discovery
            let discovery = DiscoveryService::new(device_info.clone(), DEFAULT_BEACON_PORT);
            discovery.start().await?;

            // 2. Start server
            let (server, mut rx) = DropBarServer::new(device_info, save_path.clone(), port);
            tokio::spawn(async move {
                if let Err(e) = server.run().await {
                    eprintln!("Server error: {}", e);
                }
            });

            // 3. Listen for incoming payloads
            println!("{}", "⚡ Waiting for files / drops from mobile devices...".dimmed());
            while let Ok(payload) = rx.recv().await {
                match payload {
                    IncomingPayload::File { filename, path, size, sha256, sender } => {
                        println!("\n{}", "📥 [INCOMING FILE RECEIVED]".green().bold());
                        println!("  📄 Filename: {}", filename.bold());
                        println!("  📍 Saved to: {}", path.display());
                        println!("  ⚖️ Size: {} bytes ({:.2} MB)", size, size as f64 / 1_048_576.0);
                        println!("  👤 From: {}", sender.unwrap_or_else(|| "Unknown".to_string()).cyan());
                        println!("  🔒 SHA256: {}", sha256);
                    }
                    IncomingPayload::Text { text, is_url, sender } => {
                        println!("\n{}", "📥 [INCOMING TEXT RECEIVED]".yellow().bold());
                        println!("  👤 From: {}", sender.cyan());
                        println!("  🔗 Is URL: {}", is_url);
                        println!("  💬 Content:\n{}", text);
                    }
                    IncomingPayload::DeviceRegistered(device) => {
                        println!("\n{}", format!("🟢 [DEVICE REGISTERED: {} ({})]", device.name, device.ip).green().bold());
                    }
                }
            }
        }
        Commands::Peers { wait_seconds } => {
            let local_ip = get_local_ip();
            let device_info = DeviceInfo {
                id: uuid::Uuid::new_v4().to_string(),
                name: "DropBar-Scanner".to_string(),
                device_type: DeviceType::ArchLinux,
                ip: local_ip,
                port: 53318,
                last_seen: chrono::Utc::now(),
            };

            let discovery = DiscoveryService::new(device_info, DEFAULT_BEACON_PORT);
            discovery.start().await?;

            println!("🔍 Scanning for DropBar peers for {} seconds...", wait_seconds);
            tokio::time::sleep(Duration::from_secs(wait_seconds)).await;

            let peers = discovery.get_peers().await;
            if peers.is_empty() {
                println!("{}", "No active DropBar peers found on the network.".yellow());
            } else {
                println!("\nFound {} peer(s):", peers.len().to_string().green().bold());
                for (idx, peer) in peers.iter().enumerate() {
                    println!(
                        "  [{}] {} ({:?}) -> {}:{}",
                        idx + 1,
                        peer.name.bold(),
                        peer.device_type,
                        peer.ip.cyan(),
                        peer.port
                    );
                }
            }
        }
        Commands::Send { target, payload, port } => {
            let client = DropBarClient::new("Arch-CLI");
            let target_device = DeviceInfo {
                id: "manual".to_string(),
                name: target.clone(),
                device_type: DeviceType::Unknown,
                ip: target.clone(),
                port,
                last_seen: chrono::Utc::now(),
            };

            let path = PathBuf::from(&payload);
            if path.exists() {
                println!("📤 Sending file '{}' to {}:{}...", payload.cyan(), target, port);
                let resp = client.send_file(&target_device, &path).await?;
                println!("✅ Success: {}", resp.message.green().bold());
            } else {
                let is_url = payload.starts_with("http://") || payload.starts_with("https://");
                println!("📤 Sending {} to {}:{}...", if is_url { "URL" } else { "Text" }, target, port);
                let resp = client.send_text(&target_device, &payload, is_url).await?;
                println!("✅ Success: {}", resp.message.green().bold());
            }
        }
    }

    Ok(())
}

