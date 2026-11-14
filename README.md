# ⚡ DropBar (Arch Linux Hyprland & Mobile)

A high-performance, minimalist drag-and-drop shelf and cross-device sync bar connecting **Arch Linux (Hyprland)** and **Mobile (Android & iOS)** over local Wi-Fi / hotspot.

---

## 🌟 Highlights
- **Wayland / Hyprland Native:** Pinned, floating, borderless bottom pill shelf styled with a sleek dark neon aesthetic.
- **Bi-directional Drag & Drop:**
  - Drag files or snippets into the desktop bar -> beams directly to your phone.
  - Drag photos, PDFs, or files from mobile split-screen / share sheet -> saved directly to `~/Downloads/DropBar/` on Arch Linux.
- **Zero-Config Discovery:** Instant peer detection via local UDP broadcast beacon. No accounts, no cloud relays, no latency.
- **Fast Chunked Streaming:** Supports huge files (10GB+), text snippets, links, and photos with SHA-256 integrity verification.
- **Desktop & Mobile Notifications:** Notifies on incoming drops with quick-open actions.

---

## 📁 Project Structure

```text
Projects/dropbar/
├── Cargo.toml                    # Root workspace
├── dropbar-core/                 # Shared networking, discovery & transfer protocol engine
│   ├── src/
│   │   ├── models.rs             # DeviceInfo, Payload schemas
│   │   ├── discovery.rs          # UDP broadcast beacon service (port 53317)
│   │   ├── server.rs             # Axum streaming HTTP receiver (port 53318)
│   │   ├── client.rs             # Reqwest multipart & streaming sender
│   │   └── lib.rs
│   └── Cargo.toml
├── dropbar-desktop/              # Arch Linux GTK4 floating shelf UI
│   ├── src/
│   │   └── main.rs               # Hyprland floating window, GtkDropTarget, peer selector
│   └── Cargo.toml
├── dropbar-cli/                  # Headless CLI for scanning, daemonizing, and scripting
│   ├── src/
│   │   └── main.rs               # CLI commands: daemon, peers, send
│   └── Cargo.toml
├── dropbar-mobile/               # Cross-platform Flutter mobile client (Android / iOS)
│   ├── android/                  # AndroidManifest.xml with system share sheet intent filters
│   ├── ios/                      # Info.plist with Bonjour & local network permissions
│   ├── lib/
│   │   ├── models/device_info.dart
│   │   ├── services/discovery_service.dart
│   │   ├── services/transfer_service.dart
│   │   ├── services/receiver_service.dart
│   │   ├── screens/home_screen.dart
│   │   └── main.dart
│   └── pubspec.yaml
├── hyprland/
│   └── dropbar.conf              # Hyprland window rules (float, pin, noborder, animation)
├── systemd/
│   └── dropbar.service           # User systemd service unit
├── scripts/
│   └── setup-hyprland.sh         # Auto-configuration script for Hyprland
└── PKGBUILD                      # Arch Linux package build recipe
```

---

## 🚀 Quick Start Guide

### 1. Hyprland Integration Setup
```bash
~/Projects/dropbar/scripts/setup-hyprland.sh
```
*HotKey:* Press **`Super + T`** anywhere in Hyprland to launch or focus the DropBar shelf!

---

### 2. Building & Running Desktop DropBar
From the `~/Projects/dropbar` directory:
```bash
cd ~/Projects/dropbar

# Launch the compiled binary directly:
./target/debug/dropbar-desktop

# Or rebuild with Cargo:
cargo run --bin dropbar-desktop
```

---

### 3. Using the DropBar CLI (Headless / Scripting)
The CLI tool is ideal for headless environments, server boxes, or piping tools like `hyprshot`:

```bash
# Scan for active peers on local Wi-Fi:
~/Projects/dropbar/target/debug/dropbar-cli peers

# Run as background daemon:
~/Projects/dropbar/target/debug/dropbar-cli daemon

# Send a screenshot or file directly to mobile:
~/Projects/dropbar/target/debug/dropbar-cli send <mobile-ip> ~/Pictures/screenshot.png

# Send a URL or text snippet:
~/Projects/dropbar/target/debug/dropbar-cli send <mobile-ip> "https://archlinux.org"
```

---

### 4. Running the Mobile Companion App (Android & iOS)
Inside `dropbar-mobile`:
```bash
cd ~/Projects/dropbar/dropbar-mobile
flutter pub get
flutter run
```
- Both phone and Arch Linux PC must be on the same Wi-Fi network (or mobile hotspot).
- Peer discovery will automatically find each device within 1–3 seconds.
- You can also manually connect via IP by tapping the link icon in the top right.

---

## ⚙️ Network Ports Used
- **`53317` UDP:** Zero-configuration broadcast beacon discovery.
- **`53318` TCP:** Desktop HTTP receiver API.
- **`53319` TCP:** Mobile HTTP receiver API.
