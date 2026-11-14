use dropbar_core::{
    get_default_download_dir, get_local_ip, DeviceInfo, DeviceType, DiscoveryService, DropBarClient,
    DropBarServer, IncomingPayload, DEFAULT_BEACON_PORT,
};
use gtk4::gdk;
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, CheckButton, CssProvider, DropTarget,
    Label, Orientation, ScrolledWindow, WindowHandle,
};
use notify_rust::Notification;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

const APP_ID: &str = "io.dropbar.desktop";
const CSS_STYLE: &str = r#"
window {
    background-color: rgba(15, 20, 28, 0.96);
    border: 1.5px solid #00FFCC;
    border-radius: 16px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.6);
}

.drag-grip {
    color: #475569;
    font-size: 14px;
    font-weight: bold;
    padding: 2px 0 0 0;
}

.drag-grip:hover {
    color: #00FFCC;
}

.drop-bar-header {
    background: transparent;
    padding: 4px 8px;
}

.title-label {
    font-size: 13px;
    font-weight: bold;
    color: #00FFCC;
}

.status-label {
    font-size: 11px;
    color: #94A3B8;
}

.status-hotspot {
    font-size: 11px;
    font-weight: 600;
    color: #10B981;
}

.web-portal-banner {
    background: rgba(0, 255, 204, 0.1);
    border: 1px solid rgba(0, 255, 204, 0.25);
    border-radius: 8px;
    padding: 6px 10px;
}

.web-portal-text {
    font-size: 11px;
    color: #00FFCC;
    font-weight: 500;
}

.drop-zone {
    background: rgba(30, 41, 59, 0.6);
    border: 2px dashed #334155;
    border-radius: 12px;
    padding: 16px 20px;
    transition: all 180ms ease;
}

.drop-zone:hover, .drop-zone.active {
    background: rgba(0, 255, 204, 0.18);
    border-color: #00FFCC;
}

.drop-zone-text {
    font-size: 13px;
    font-weight: 600;
    color: #F8FAFC;
}

.drop-zone-sub {
    font-size: 11px;
    color: #64748B;
}

.item-row {
    background: #1E293B;
    border-radius: 8px;
    padding: 6px 10px;
    margin-bottom: 4px;
}

.item-name {
    font-size: 12px;
    color: #E2E8F0;
}

.send-btn {
    background: #00FFCC;
    color: #0F172A;
    font-weight: bold;
    border-radius: 8px;
    padding: 8px 16px;
    border: none;
}

.send-btn:hover {
    background: #10B981;
}

.send-btn:disabled {
    background: #1E293B;
    color: #475569;
}

.clear-btn {
    background: rgba(239, 68, 68, 0.15);
    color: #EF4444;
    border-radius: 8px;
    border: 1px solid rgba(239, 68, 68, 0.4);
    padding: 6px 12px;
}

.auto-beam-check {
    color: #94A3B8;
    font-size: 11px;
}
"#;

struct AppState {
    stashed_files: Vec<PathBuf>,
    discovered_peers: HashMap<String, DeviceInfo>,
    selected_peer: Option<DeviceInfo>,
    auto_beam: bool,
}

fn main() -> glib::ExitCode {
    // 1. Initialize Tokio runtime on background thread
    let rt = tokio::runtime::Runtime::new().expect("Failed to start Tokio runtime");
    let rt_handle = rt.handle().clone();

    // 2. Start core services: Discovery + Server
    let local_ip = get_local_ip();
    let port = 53318;
    let local_device = DeviceInfo {
        id: uuid::Uuid::new_v4().to_string(),
        name: "Arch-Hyprland".to_string(),
        device_type: DeviceType::ArchLinux,
        ip: local_ip.clone(),
        port,
        last_seen: chrono::Utc::now(),
    };

    let download_dir = get_default_download_dir();
    let discovery = DiscoveryService::new(local_device.clone(), DEFAULT_BEACON_PORT);
    let (server, mut payload_rx) = DropBarServer::new(local_device.clone(), download_dir.clone(), port);

    // Spawn server & discovery in background runtime
    let discovery_clone = discovery.clone();
    rt_handle.spawn(async move {
        if let Err(e) = discovery_clone.start().await {
            eprintln!("Discovery error: {}", e);
        }
    });

    rt_handle.spawn(async move {
        if let Err(e) = server.run().await {
            eprintln!("DropBar Server error: {}", e);
        }
    });

    // 3. Listen for incoming drops from mobile and show desktop notification
    let discovery_for_reg = discovery.clone();
    rt_handle.spawn(async move {
        while let Ok(payload) = payload_rx.recv().await {
            match payload {
                IncomingPayload::File { filename, size, sender, .. } => {
                    let sender_name = sender.unwrap_or_else(|| "Mobile".to_string());
                    let _ = Notification::new()
                        .summary(&format!("📥 DropBar: File from {}", sender_name))
                        .body(&format!("Received '{}' ({:.1} KB)", filename, size as f64 / 1024.0))
                        .icon("emblem-downloads")
                        .appname("DropBar")
                        .show();
                }
                IncomingPayload::Text { text, sender, .. } => {
                    let _ = Notification::new()
                        .summary(&format!("📥 DropBar: Text from {}", sender))
                        .body(&text)
                        .icon("text-x-generic")
                        .appname("DropBar")
                        .show();
                }
                IncomingPayload::DeviceRegistered(device) => {
                    discovery_for_reg.register_peer(device);
                }
            }
        }
    });

    // 4. GTK4 Application
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    let discovery_for_ui = discovery.clone();
    let rt_handle_for_ui = rt_handle.clone();
    let local_ip_for_ui = local_ip.clone();

    app.connect_startup(|_| {
        let provider = CssProvider::new();
        provider.load_from_data(CSS_STYLE);
        if let Some(display) = gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    });

    app.connect_activate(move |application| {
        build_ui(application, discovery_for_ui.clone(), rt_handle_for_ui.clone(), local_ip_for_ui.clone(), port);
    });

    app.run()
}

fn build_ui(
    app: &Application,
    discovery: DiscoveryService,
    rt: tokio::runtime::Handle,
    local_ip: String,
    port: u16,
) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("DropBar")
        .default_width(460)
        .default_height(320)
        .resizable(true)
        .decorated(false)
        .build();

    let state = Rc::new(RefCell::new(AppState {
        stashed_files: Vec::new(),
        discovered_peers: HashMap::new(),
        selected_peer: None,
        auto_beam: true,
    }));

    let main_box = GtkBox::new(Orientation::Vertical, 8);
    main_box.set_margin_start(14);
    main_box.set_margin_end(14);
    main_box.set_margin_top(6);
    main_box.set_margin_bottom(12);

    // 1. Moveable Window Handle
    let window_handle = WindowHandle::new();
    let header_box = GtkBox::new(Orientation::Vertical, 2);
    header_box.add_css_class("drop-bar-header");

    let grip_lbl = Label::new(Some("⠿ ⠿ ⠿"));
    grip_lbl.add_css_class("drag-grip");
    grip_lbl.set_tooltip_text(Some("Click & drag to move DropBar anywhere on screen"));
    header_box.append(&grip_lbl);

    let header_row = GtkBox::new(Orientation::Horizontal, 8);
    let title_lbl = Label::new(Some("⚡ DropBar Shelf"));
    title_lbl.add_css_class("title-label");

    let status_lbl = Label::new(Some("🔍 Searching Hotspot / Wi-Fi..."));
    status_lbl.add_css_class("status-label");
    status_lbl.set_hexpand(true);
    status_lbl.set_halign(gtk4::Align::End);

    header_row.append(&title_lbl);
    header_row.append(&status_lbl);
    header_box.append(&header_row);

    window_handle.set_child(Some(&header_box));
    main_box.append(&window_handle);

    // 2. Web Portal Link Banner (Zero install on P4)
    let web_box = GtkBox::new(Orientation::Horizontal, 6);
    web_box.add_css_class("web-portal-banner");

    let portal_url = format!("http://{}:{}", local_ip, port);
    let web_lbl = Label::new(Some(&format!("📱 On P4, open browser: {}", portal_url)));
    web_lbl.add_css_class("web-portal-text");
    web_lbl.set_hexpand(true);
    web_lbl.set_halign(gtk4::Align::Start);

    let copy_link_btn = Button::with_label("Copy Link");
    copy_link_btn.add_css_class("clear-btn");
    let portal_url_clone = portal_url.clone();
    copy_link_btn.connect_clicked(move |_| {
        if let Some(display) = gdk::Display::default() {
            display.clipboard().set_text(&portal_url_clone);
            let _ = Notification::new()
                .summary("📋 DropBar Link Copied")
                .body(&format!("Open {} on your phone's browser", portal_url_clone))
                .show();
        }
    });

    web_box.append(&web_lbl);
    web_box.append(&copy_link_btn);
    main_box.append(&web_box);

    // 3. Drop Zone Target Area
    let drop_box = GtkBox::new(Orientation::Vertical, 4);
    drop_box.add_css_class("drop-zone");
    drop_box.set_halign(gtk4::Align::Fill);

    let drop_text = Label::new(Some("📥 Drag & Drop Files or Text Here"));
    drop_text.add_css_class("drop-zone-text");

    let drop_sub = Label::new(Some("Auto-beams to connected Mobile Hotspot"));
    drop_sub.add_css_class("drop-zone-sub");

    drop_box.append(&drop_text);
    drop_box.append(&drop_sub);

    // 4. File list container
    let file_list_box = GtkBox::new(Orientation::Vertical, 4);
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .min_content_height(70)
        .max_content_height(120)
        .child(&file_list_box)
        .build();

    // 5. Options Bar: Auto-Beam Toggle
    let options_box = GtkBox::new(Orientation::Horizontal, 8);
    let auto_beam_check = CheckButton::with_label("Auto-beam on drop (Hotspot ready)");
    auto_beam_check.add_css_class("auto-beam-check");
    auto_beam_check.set_active(true);

    let state_check = state.clone();
    auto_beam_check.connect_toggled(move |btn| {
        state_check.borrow_mut().auto_beam = btn.is_active();
    });

    options_box.append(&auto_beam_check);

    // 6. Action buttons bar
    let action_box = GtkBox::new(Orientation::Horizontal, 8);
    let clear_btn = Button::with_label("Clear");
    clear_btn.add_css_class("clear-btn");

    let add_files_btn = Button::with_label("➕ Add Files");
    add_files_btn.add_css_class("clear-btn");

    let open_downloads_btn = Button::with_label("📂 Downloads");
    open_downloads_btn.add_css_class("clear-btn");

    let send_btn = Button::with_label("🚀 Beam to Mobile");
    send_btn.add_css_class("send-btn");
    send_btn.set_sensitive(false);
    send_btn.set_hexpand(true);

    action_box.append(&clear_btn);
    action_box.append(&add_files_btn);
    action_box.append(&open_downloads_btn);
    action_box.append(&send_btn);

    main_box.append(&drop_box);
    main_box.append(&scrolled);
    main_box.append(&options_box);
    main_box.append(&action_box);

    window.set_child(Some(&main_box));

    // Connect Drop Target with Automatic Transfer
    let drop_target = DropTarget::new(glib::types::Type::INVALID, gdk::DragAction::all());
    drop_target.set_types(&[
        gdk::FileList::static_type(),
        gio::File::static_type(),
        glib::types::Type::STRING,
    ]);

    drop_target.connect_enter(|_, _, _| gdk::DragAction::COPY);
    drop_target.connect_motion(|_, _, _| gdk::DragAction::COPY);

    let state_drop = state.clone();
    let file_list_box_drop = file_list_box.clone();
    let send_btn_drop = send_btn.clone();
    let rt_drop = rt.clone();

    drop_target.connect_drop(move |_, value, _, _| {
        let mut files_to_add: Vec<PathBuf> = Vec::new();

        if let Ok(file_list) = value.get::<gdk::FileList>() {
            for file in file_list.files() {
                if let Some(path) = file.path() {
                    files_to_add.push(path);
                }
            }
        } else if let Ok(file) = value.get::<gio::File>() {
            if let Some(path) = file.path() {
                files_to_add.push(path);
            }
        } else if let Ok(text) = value.get::<String>() {
            for line in text.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if trimmed.starts_with("file://") {
                    if let Ok((path, _)) = glib::filename_from_uri(trimmed) {
                        if path.exists() {
                            files_to_add.push(path);
                        }
                    }
                } else {
                    let path = PathBuf::from(trimmed);
                    if path.exists() {
                        files_to_add.push(path);
                    }
                }
            }
        }

        if files_to_add.is_empty() {
            return false;
        }

        let mut st = state_drop.borrow_mut();
        let mut added = false;
        for path in files_to_add {
            if !st.stashed_files.contains(&path) {
                st.stashed_files.push(path);
                added = true;
            }
        }

        if !added {
            return false;
        }

        let should_auto_beam = st.auto_beam;
        let target = st.selected_peer.clone();
        drop(st);

        refresh_file_list(&file_list_box_drop, &state_drop);

        if should_auto_beam {
            if let Some(target_peer) = target {
                trigger_beam(
                    &send_btn_drop,
                    &state_drop,
                    &file_list_box_drop,
                    &rt_drop,
                    target_peer,
                );
            }
        }
        true
    });
    drop_box.add_controller(drop_target);

    // Add Files Dialog Handler
    let state_add = state.clone();
    let file_list_box_add = file_list_box.clone();
    let send_btn_add = send_btn.clone();
    let rt_add = rt.clone();
    let window_weak = window.downgrade();

    add_files_btn.connect_clicked(move |_| {
        let parent_win = window_weak.upgrade();
        let dialog = gtk4::FileChooserNative::new(
            Some("Select Files to Beam"),
            parent_win.as_ref(),
            gtk4::FileChooserAction::Open,
            Some("Select"),
            Some("Cancel"),
        );
        dialog.set_select_multiple(true);

        let state_f = state_add.clone();
        let file_box_f = file_list_box_add.clone();
        let send_f = send_btn_add.clone();
        let rt_f = rt_add.clone();

        dialog.connect_response(move |d, resp| {
            if resp == gtk4::ResponseType::Accept {
                let files = d.files();
                let mut added = false;
                let mut st = state_f.borrow_mut();
                for i in 0..files.n_items() {
                    if let Some(obj) = files.item(i) {
                        if let Ok(file) = obj.downcast::<gio::File>() {
                            if let Some(path) = file.path() {
                                if !st.stashed_files.contains(&path) {
                                    st.stashed_files.push(path);
                                    added = true;
                                }
                            }
                        }
                    }
                }
                let should_auto_beam = st.auto_beam;
                let target = st.selected_peer.clone();
                drop(st);

                if added {
                    refresh_file_list(&file_box_f, &state_f);
                    if should_auto_beam {
                        if let Some(target_peer) = target {
                            trigger_beam(&send_f, &state_f, &file_box_f, &rt_f, target_peer);
                        }
                    }
                }
            }
        });

        dialog.show();
    });

    // Clear Button Handler
    let state_clear = state.clone();
    let file_list_box_clear = file_list_box.clone();
    clear_btn.connect_clicked(move |_| {
        state_clear.borrow_mut().stashed_files.clear();
        refresh_file_list(&file_list_box_clear, &state_clear);
    });

    // Open Downloads Handler (prioritizes Nautilus to avoid editor interception)
    open_downloads_btn.connect_clicked(|_| {
        let path = get_default_download_dir();
        let _ = std::fs::create_dir_all(&path);
        if std::process::Command::new("nautilus").arg(&path).spawn().is_err() {
            let _ = open::that(&path);
        }
    });

    // Send Button Handler
    let state_send = state.clone();
    let file_list_box_send = file_list_box.clone();
    let rt_send = rt.clone();

    send_btn.connect_clicked(move |btn| {
        let st = state_send.borrow();
        let target = match &st.selected_peer {
            Some(p) => p.clone(),
            None => {
                let _ = Notification::new()
                    .summary("⚠️ DropBar")
                    .body("Connecting to mobile hotspot... please ensure mobile app or web portal is active.")
                    .show();
                return;
            }
        };
        drop(st);

        trigger_beam(btn, &state_send, &file_list_box_send, &rt_send, target);
    });

    // Background Peer Discovery Poller channel for UI Status
    let status_lbl_c = status_lbl.clone();
    let state_c = state.clone();
    let send_btn_c = send_btn.clone();
    let (peer_tx, mut peer_rx) = tokio::sync::mpsc::channel(8);

    rt.spawn(async move {
        loop {
            let peers = discovery.get_peers().await;
            if peer_tx.send(peers).await.is_err() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        }
    });

    glib::spawn_future_local(async move {
        while let Some(peers) = peer_rx.recv().await {
            let mut st = state_c.borrow_mut();
            st.discovered_peers.clear();
            for p in &peers {
                st.discovered_peers.insert(p.id.clone(), p.clone());
            }

            if let Some(first) = peers.first() {
                st.selected_peer = Some(first.clone());
                status_lbl_c.remove_css_class("status-label");
                status_lbl_c.add_css_class("status-hotspot");
                status_lbl_c.set_text(&format!("🟢 Ready: {} ({})", first.name, first.ip));
                send_btn_c.set_sensitive(true);
            } else {
                st.selected_peer = None;
                status_lbl_c.remove_css_class("status-hotspot");
                status_lbl_c.add_css_class("status-label");
                status_lbl_c.set_text("🟡 Hotspot connecting...");
                send_btn_c.set_sensitive(false);
            }
        }
    });

    window.present();
}

fn trigger_beam(
    btn: &Button,
    state: &Rc<RefCell<AppState>>,
    file_list_box: &GtkBox,
    rt: &tokio::runtime::Handle,
    target: DeviceInfo,
) {
    let files = state.borrow().stashed_files.clone();
    if files.is_empty() {
        return;
    }

    btn.set_sensitive(false);
    let btn_widget = btn.clone();
    let state_after = state.clone();
    let file_list_after = file_list_box.clone();
    let rt_handle = rt.clone();

    glib::spawn_future_local(async move {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let target_c = target.clone();
        let files_c = files.clone();

        rt_handle.spawn(async move {
            let client = DropBarClient::new("Arch-Hyprland");
            let mut sent_count = 0;

            for file in &files_c {
                if client.send_file(&target_c, file).await.is_ok() {
                    sent_count += 1;
                }
            }
            let _ = tx.send(sent_count);
        });

        if let Ok(sent_count) = rx.await {
            let _ = Notification::new()
                .summary("🚀 DropBar Hotspot Transfer")
                .body(&format!("Successfully beamed {} file(s) to {}", sent_count, target.name))
                .show();

            state_after.borrow_mut().stashed_files.clear();
            refresh_file_list(&file_list_after, &state_after);
        }

        btn_widget.set_sensitive(true);
    });
}

fn refresh_file_list(container: &GtkBox, state: &Rc<RefCell<AppState>>) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    let st = state.borrow();
    if st.stashed_files.is_empty() {
        let empty_lbl = Label::new(Some("Shelf is empty"));
        empty_lbl.add_css_class("status-label");
        empty_lbl.set_margin_top(6);
        container.append(&empty_lbl);
        return;
    }

    for path in &st.stashed_files {
        let row = GtkBox::new(Orientation::Horizontal, 8);
        row.add_css_class("item-row");

        let file_name = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let label = Label::new(Some(&format!("📄 {}", file_name)));
        label.add_css_class("item-name");
        label.set_hexpand(true);
        label.set_halign(gtk4::Align::Start);

        row.append(&label);
        container.append(&row);
    }
}
