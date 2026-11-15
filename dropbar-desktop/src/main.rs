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
    GestureClick, Label, Orientation, ScrolledWindow, Stack, StackTransitionType, WindowHandle,
};
use notify_rust::Notification;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

const APP_ID: &str = "io.dropbar.desktop";

// Hyprland Native Theme (matching ~/.config/hypr rounding=12 and Catppuccin Onyx)
const CSS_STYLE: &str = r#"
window {
    background-color: transparent;
    font-family: "JetBrains Mono", "Google Sans", "Inter", sans-serif;
}

/* Hyprland Glass Shelf Container (1/5 ratio of screen, 12px rounding) */
.serpantinum-shelf {
    background-color: rgba(29, 29, 34, 0.95);
    border: 1px solid rgba(180, 190, 254, 0.25);
    border-radius: 12px;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.7);
    padding: 12px;
}

/* Hyprland Edge Logo Pill (Collapsed State) */
.serpantinum-logo-pill {
    background: rgba(29, 29, 34, 0.96);
    border: 1px solid rgba(180, 190, 254, 0.35);
    border-radius: 12px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.6), 0 0 12px rgba(203, 166, 247, 0.25);
    padding: 12px 6px;
    transition: all 180ms ease;
}

.serpantinum-logo-pill:hover, .serpantinum-logo-pill.active {
    border-color: #cba6f7;
    background: rgba(43, 43, 51, 0.98);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.8), 0 0 20px rgba(203, 166, 247, 0.5);
}

.logo-icon {
    font-size: 24px;
    color: #cba6f7;
    margin-bottom: 4px;
}

.logo-sub-icon {
    font-size: 16px;
    color: #89b4fa;
    margin-top: 4px;
}

.logo-text-vertical {
    font-size: 9px;
    font-weight: 800;
    letter-spacing: 2px;
    color: #cba6f7;
    line-height: 1.1;
}

.drag-grip {
    color: #5a5a6a;
    font-size: 13px;
    font-weight: bold;
    padding: 2px 0 0 0;
}

.drag-grip:hover {
    color: #cba6f7;
}

.drop-bar-header {
    background: transparent;
    padding: 0 2px 4px 2px;
}

.title-badge {
    background: rgba(203, 166, 247, 0.12);
    border: 1px solid rgba(203, 166, 247, 0.28);
    border-radius: 8px;
    padding: 3px 8px;
    color: #cba6f7;
    font-weight: 700;
    font-size: 11px;
    letter-spacing: 0.5px;
}

.status-badge {
    background: rgba(148, 226, 213, 0.12);
    border: 1px solid rgba(148, 226, 213, 0.28);
    border-radius: 8px;
    padding: 3px 8px;
    color: #94e2d5;
    font-size: 11px;
    font-weight: 600;
}

.status-searching {
    background: rgba(249, 226, 175, 0.1);
    border: 1px solid rgba(249, 226, 175, 0.25);
    border-radius: 8px;
    padding: 3px 8px;
    color: #f9e2af;
    font-size: 11px;
}

.collapse-btn {
    background: rgba(43, 43, 51, 0.6);
    border: 1px solid rgba(90, 90, 106, 0.4);
    border-radius: 8px;
    color: #d5d7dc;
    padding: 3px 8px;
    font-size: 12px;
    font-weight: bold;
}

.collapse-btn:hover {
    background: rgba(203, 166, 247, 0.25);
    border-color: #cba6f7;
    color: #ffffff;
}

.web-portal-banner {
    background: rgba(137, 180, 250, 0.08);
    border: 1px solid rgba(137, 180, 250, 0.22);
    border-radius: 8px;
    padding: 6px 10px;
}

.web-portal-text {
    font-size: 11px;
    color: #89b4fa;
    font-weight: 600;
}

.drop-zone {
    background: rgba(29, 29, 34, 0.65);
    border: 1.5px dashed rgba(203, 166, 247, 0.35);
    border-radius: 12px;
    padding: 16px 14px;
    transition: all 180ms ease;
}

.drop-zone:hover, .drop-zone.active {
    background: rgba(203, 166, 247, 0.14);
    border-color: #cba6f7;
    box-shadow: 0 0 16px rgba(203, 166, 247, 0.2);
}

.drop-zone-text {
    font-size: 12px;
    font-weight: 700;
    color: #eeeff2;
}

.drop-zone-sub {
    font-size: 10px;
    color: #b8bbc2;
}

.item-row {
    background: #2b2b33;
    border: 1px solid rgba(57, 57, 68, 0.7);
    border-radius: 8px;
    padding: 5px 10px;
    margin-bottom: 4px;
}

.item-name {
    font-size: 11px;
    color: #eeeff2;
}

.send-btn {
    background: linear-gradient(135deg, #cba6f7, #89b4fa);
    color: #0f0f12;
    font-weight: 700;
    font-size: 12px;
    border-radius: 8px;
    padding: 7px 14px;
    border: none;
    box-shadow: 0 4px 14px rgba(203, 166, 247, 0.3);
}

.send-btn:hover {
    background: linear-gradient(135deg, #d5b8ff, #9ac1ff);
}

.send-btn:disabled {
    background: #2b2b33;
    color: #5a5a6a;
    box-shadow: none;
}

.serpantinum-btn {
    background: #2b2b33;
    border: 1px solid rgba(90, 90, 106, 0.4);
    border-radius: 8px;
    color: #d5d7dc;
    padding: 5px 10px;
    font-size: 11px;
    font-weight: 600;
}

.serpantinum-btn:hover {
    background: #393944;
    border-color: #cba6f7;
    color: #ffffff;
}

.clear-btn {
    background: rgba(243, 139, 168, 0.12);
    border: 1px solid rgba(243, 139, 168, 0.35);
    border-radius: 8px;
    color: #f38ba8;
    padding: 5px 10px;
    font-size: 11px;
    font-weight: 600;
}

.clear-btn:hover {
    background: rgba(243, 139, 168, 0.22);
    border-color: #f38ba8;
}

.auto-beam-check {
    color: #b8bbc2;
    font-size: 10px;
}
"#;

struct AppState {
    stashed_files: Vec<PathBuf>,
    discovered_peers: HashMap<String, DeviceInfo>,
    selected_peer: Option<DeviceInfo>,
    auto_beam: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let rt_handle = tokio::runtime::Handle::current();
    let local_ip = get_local_ip();
    let port = 53318;

    let local_device = DeviceInfo {
        id: format!("desktop_{}", uuid::Uuid::new_v4()),
        name: "Arch-Hyprland (DropBar)".to_string(),
        device_type: DeviceType::ArchLinux,
        ip: local_ip.clone(),
        port,
        last_seen: chrono::Utc::now(),
    };

    let download_dir = get_default_download_dir();
    let discovery = DiscoveryService::new(local_device.clone(), DEFAULT_BEACON_PORT);
    let (server, mut payload_rx) = DropBarServer::new(local_device.clone(), download_dir.clone(), port);

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

    // Notify upon received incoming payloads from P4/mobile
    rt_handle.spawn(async move {
        while let Ok(payload) = payload_rx.recv().await {
            match payload {
                IncomingPayload::File { filename, sender, path, .. } => {
                    let sender_display = sender.unwrap_or_else(|| "Mobile".to_string());
                    let _ = Notification::new()
                        .summary("📥 DropBar Received File")
                        .body(&format!("'{}' received from {}", filename, sender_display))
                        .icon("document-save")
                        .show();
                    let _ = open::that(path);
                }
                IncomingPayload::Text { text, sender, is_url } => {
                    let summary = if is_url { "🔗 DropBar Received Link" } else { "📋 DropBar Received Text" };
                    let _ = Notification::new()
                        .summary(summary)
                        .body(&format!("From {}: {}", sender, text))
                        .icon("emblem-shared")
                        .show();
                }
                IncomingPayload::DeviceRegistered(_) => {}
            }
        }
    });

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
        if let Some(existing_win) = application.windows().first() {
            existing_win.set_opacity(1.0);
            existing_win.present();
            dock_to_side("right", 1920, 1920 / 5);
            return;
        }
        build_ui(application, discovery_for_ui.clone(), rt_handle_for_ui.clone(), local_ip_for_ui.clone(), port);
    });

    app.run();
    Ok(())
}

fn build_ui(
    app: &Application,
    discovery: DiscoveryService,
    rt: tokio::runtime::Handle,
    local_ip: String,
    port: u16,
) {
    // Determine 1/5 ratio of screen width and screen height
    let mut screen_width = 1920;
    let mut screen_height = 1080;
    if let Some(display) = gdk::Display::default() {
        let monitors = display.monitors();
        if let Some(first) = monitors.item(0) {
            if let Ok(monitor) = first.downcast::<gdk::Monitor>() {
                let geom = monitor.geometry();
                if geom.width() > 0 {
                    screen_width = geom.width();
                }
                if geom.height() > 0 {
                    screen_height = geom.height();
                }
            }
        }
    }

    let expanded_width = (screen_width / 5).max(360);
    let expanded_height = 580;
    let collapsed_width = 54;
    let collapsed_height = 170;

    let window = ApplicationWindow::builder()
        .application(app)
        .title("DropBar")
        .default_width(collapsed_width)
        .default_height(collapsed_height)
        .resizable(true)
        .decorated(false)
        .build();

    let state = Rc::new(RefCell::new(AppState {
        stashed_files: Vec::new(),
        discovered_peers: HashMap::new(),
        selected_peer: None,
        auto_beam: true,
    }));

    let stack = Stack::new();
    stack.set_hhomogeneous(false);
    stack.set_vhomogeneous(false);
    stack.set_transition_type(StackTransitionType::SlideLeftRight);
    stack.set_transition_duration(180);

    // ==========================================
    // 1. COLLAPSED VIEW: EDGE LOGO PILL
    // ==========================================
    let logo_pill_box = GtkBox::new(Orientation::Vertical, 4);
    logo_pill_box.add_css_class("serpantinum-logo-pill");
    logo_pill_box.set_valign(gtk4::Align::Center);
    logo_pill_box.set_halign(gtk4::Align::Fill);
    logo_pill_box.set_hexpand(true);
    logo_pill_box.set_size_request(collapsed_width, collapsed_height);

    let logo_icon = Label::new(Some("⚡"));
    logo_icon.add_css_class("logo-icon");

    let logo_txt = Label::new(Some("D\nR\nO\nP"));
    logo_txt.add_css_class("logo-text-vertical");
    logo_txt.set_justify(gtk4::Justification::Center);

    let logo_sub = Label::new(Some("📥"));
    logo_sub.add_css_class("logo-sub-icon");

    logo_pill_box.append(&logo_icon);
    logo_pill_box.append(&logo_txt);
    logo_pill_box.append(&logo_sub);

    // ==========================================
    // 2. EXPANDED VIEW: 1/5 RATIO SERPANTINUM SHELF
    // ==========================================
    let shelf_box = GtkBox::new(Orientation::Vertical, 8);
    shelf_box.add_css_class("serpantinum-shelf");
    shelf_box.set_size_request(expanded_width, expanded_height);

    // Header row with grip, title badge, hotspot status, edge flip, and collapse button
    let window_handle = WindowHandle::new();
    let header_box = GtkBox::new(Orientation::Vertical, 4);
    header_box.add_css_class("drop-bar-header");

    let grip_lbl = Label::new(Some("⠿ ⠿ ⠿"));
    grip_lbl.add_css_class("drag-grip");
    grip_lbl.set_tooltip_text(Some("Drag handle to reposition DropBar"));
    header_box.append(&grip_lbl);

    let header_row = GtkBox::new(Orientation::Horizontal, 6);
    let title_lbl = Label::new(Some("⚡ DROPBAR"));
    title_lbl.add_css_class("title-badge");

    let status_lbl = Label::new(Some("🔍 Searching Hotspot..."));
    status_lbl.add_css_class("status-searching");
    status_lbl.set_hexpand(true);
    status_lbl.set_halign(gtk4::Align::End);

    let side_btn = Button::with_label("⇋ Side");
    side_btn.add_css_class("serpantinum-btn");
    side_btn.set_tooltip_text(Some("Dock to Left or Right edge"));

    let collapse_btn = Button::with_label("❯");
    collapse_btn.add_css_class("collapse-btn");
    collapse_btn.set_tooltip_text(Some("Collapse to edge logo"));

    header_row.append(&title_lbl);
    header_row.append(&status_lbl);
    header_row.append(&side_btn);
    header_row.append(&collapse_btn);
    header_box.append(&header_row);

    window_handle.set_child(Some(&header_box));
    shelf_box.append(&window_handle);

    // Mobile Web Portal Link Banner
    let web_box = GtkBox::new(Orientation::Horizontal, 6);
    web_box.add_css_class("web-portal-banner");

    let portal_url = format!("http://{}:{}", local_ip, port);
    let web_lbl = Label::new(Some(&format!("📱 P4 Portal: {}", portal_url)));
    web_lbl.add_css_class("web-portal-text");
    web_lbl.set_hexpand(true);
    web_lbl.set_halign(gtk4::Align::Start);

    let copy_link_btn = Button::with_label("Copy");
    copy_link_btn.add_css_class("serpantinum-btn");
    let portal_url_clone = portal_url.clone();
    copy_link_btn.connect_clicked(move |_| {
        if let Some(display) = gdk::Display::default() {
            display.clipboard().set_text(&portal_url_clone);
            let _ = Notification::new()
                .summary("📋 DropBar Portal Link Copied")
                .body(&format!("Open {} in phone browser", portal_url_clone))
                .show();
        }
    });

    web_box.append(&web_lbl);
    web_box.append(&copy_link_btn);
    shelf_box.append(&web_box);

    // Main Drop Zone Area
    let drop_box = GtkBox::new(Orientation::Vertical, 4);
    drop_box.add_css_class("drop-zone");
    drop_box.set_halign(gtk4::Align::Fill);

    let drop_text = Label::new(Some("📥 Drag & Drop Files or Text Here"));
    drop_text.add_css_class("drop-zone-text");

    let drop_sub = Label::new(Some("Auto-beams to connected Mobile Hotspot (P4)"));
    drop_sub.add_css_class("drop-zone-sub");

    drop_box.append(&drop_text);
    drop_box.append(&drop_sub);

    // Stashed files queue list
    let file_list_box = GtkBox::new(Orientation::Vertical, 4);
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .min_content_height(80)
        .max_content_height(140)
        .child(&file_list_box)
        .build();

    // Options Bar: Auto-Beam toggle
    let options_box = GtkBox::new(Orientation::Horizontal, 8);
    let auto_beam_check = CheckButton::with_label("Auto-beam on drop (Hotspot ready)");
    auto_beam_check.add_css_class("auto-beam-check");
    auto_beam_check.set_active(true);

    let state_check = state.clone();
    auto_beam_check.connect_toggled(move |btn| {
        state_check.borrow_mut().auto_beam = btn.is_active();
    });
    options_box.append(&auto_beam_check);

    // Action Buttons Bar
    let action_box = GtkBox::new(Orientation::Horizontal, 6);
    let clear_btn = Button::with_label("Clear");
    clear_btn.add_css_class("clear-btn");

    let add_files_btn = Button::with_label("➕ Add Files");
    add_files_btn.add_css_class("serpantinum-btn");

    let open_downloads_btn = Button::with_label("📂 Downloads");
    open_downloads_btn.add_css_class("serpantinum-btn");

    let send_btn = Button::with_label("🚀 Beam");
    send_btn.add_css_class("send-btn");
    send_btn.set_sensitive(false);
    send_btn.set_hexpand(true);

    action_box.append(&clear_btn);
    action_box.append(&add_files_btn);
    action_box.append(&open_downloads_btn);
    action_box.append(&send_btn);

    shelf_box.append(&drop_box);
    shelf_box.append(&scrolled);
    shelf_box.append(&options_box);
    shelf_box.append(&action_box);

    // Add both views to Stack
    stack.add_named(&logo_pill_box, Some("logo"));
    stack.add_named(&shelf_box, Some("shelf"));
    stack.set_visible_child_name("logo"); // initial collapsed logo state

    window.set_child(Some(&stack));

    // Shared edge state
    let current_edge = Rc::new(RefCell::new("right")); // default right

    // ==========================================
    // INTERACTION 1: EXPAND / COLLAPSE LOGIC
    // ==========================================
    // 1. Clicking collapse button folds to edge logo pill
    let stack_for_coll = stack.clone();
    let win_for_coll = window.clone();
    let current_edge_coll = current_edge.clone();
    let sw_coll = screen_width;
    collapse_btn.connect_clicked(move |_| {
        win_for_coll.set_size_request(collapsed_width, collapsed_height);
        win_for_coll.set_default_size(collapsed_width, collapsed_height);
        stack_for_coll.set_visible_child_name("logo");
        dock_to_side(&*current_edge_coll.borrow(), sw_coll, collapsed_width);
    });

    // 2. Clicking the logo pill expands it to 1/5 ratio shelf
    let stack_for_expand = stack.clone();
    let win_for_expand = window.clone();
    let current_edge_expand = current_edge.clone();
    let sw_expand = screen_width;
    let click_gesture = GestureClick::new();
    click_gesture.connect_released(move |_, _, _, _| {
        win_for_expand.set_size_request(expanded_width, expanded_height);
        win_for_expand.set_default_size(expanded_width, expanded_height);
        stack_for_expand.set_visible_child_name("shelf");
        dock_to_side(&*current_edge_expand.borrow(), sw_expand, expanded_width);
    });
    logo_pill_box.add_controller(click_gesture);

    // 3. Dragging over the logo pill immediately maximizes to 1/5 ratio shelf
    let logo_drop_target = DropTarget::new(glib::types::Type::INVALID, gdk::DragAction::all());
    logo_drop_target.set_types(&[
        gdk::FileList::static_type(),
        gio::File::static_type(),
        glib::types::Type::STRING,
    ]);

    let stack_for_drag = stack.clone();
    let win_for_drag = window.clone();
    let current_edge_drag = current_edge.clone();
    let sw_drag = screen_width;
    logo_drop_target.connect_enter(move |_, _, _| {
        win_for_drag.set_size_request(expanded_width, expanded_height);
        win_for_drag.set_default_size(expanded_width, expanded_height);
        stack_for_drag.set_visible_child_name("shelf");
        dock_to_side(&*current_edge_drag.borrow(), sw_drag, expanded_width);
        gdk::DragAction::COPY
    });
    logo_drop_target.connect_motion(|_, _, _| gdk::DragAction::COPY);

    let state_logo_drop = state.clone();
    let file_list_logo = file_list_box.clone();
    let send_btn_logo = send_btn.clone();
    let rt_logo = rt.clone();
    let stack_logo_post = stack.clone();
    let win_logo_post = window.clone();
    let current_edge_drop = current_edge.clone();
    let sw_drop = screen_width;

    logo_drop_target.connect_drop(move |_, value, _, _| {
        win_logo_post.set_size_request(expanded_width, expanded_height);
        win_logo_post.set_default_size(expanded_width, expanded_height);
        stack_logo_post.set_visible_child_name("shelf");
        dock_to_side(&*current_edge_drop.borrow(), sw_drop, expanded_width);

        let files = extract_files_from_value(value);
        if files.is_empty() {
            return false;
        }

        let mut st = state_logo_drop.borrow_mut();
        let mut added = false;
        for path in files {
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

        refresh_file_list(&file_list_logo, &state_logo_drop);
        if should_auto_beam {
            if let Some(target_peer) = target {
                trigger_beam(&send_btn_logo, &state_logo_drop, &file_list_logo, &rt_logo, target_peer);
            }
        }
        true
    });
    logo_pill_box.add_controller(logo_drop_target);

    // ==========================================
    // INTERACTION 2: DOCKING & DRAG-AWARE EDGE FOLLOWER
    // ==========================================
    let current_edge_click = current_edge.clone();
    let sw_click = screen_width;
    let ew_click = expanded_width;

    side_btn.connect_clicked(move |_| {
        let mut edge = current_edge_click.borrow_mut();
        let new_edge = if *edge == "right" { "left" } else { "right" };
        *edge = new_edge;
        dock_to_side(new_edge, sw_click, ew_click);
    });

    // Realtime Drag Detection & Edge Follower
    // When a drag starts (left mouse button held down anywhere on screen),
    // the DropBar logo pill automatically appears on that side of the screen!
    // When the cursor moves over the logo, it opens into the 1/5 screen ratio shelf.
    let current_edge_poll = current_edge.clone();
    let sw_poll = screen_width;
    let sh_poll = screen_height;
    let stack_poll = stack.clone();
    let win_poll = window.clone();
    let state_poll = state.clone();

    // Start with window hidden (opacity 0.0) until drag action starts
    win_poll.set_opacity(0.0);
    win_poll.set_size_request(collapsed_width, collapsed_height);
    win_poll.set_default_size(collapsed_width, collapsed_height);
    stack_poll.set_visible_child_name("logo");
    dock_window_to_pos("right", sw_poll, collapsed_width, 180);

    let mut last_edge = "right".to_string();
    let mut last_y = 180;
    let mut last_view = "logo".to_string();
    let mut hide_timer = 0;

    glib::timeout_add_local(std::time::Duration::from_millis(80), move || {
        let is_dragging = check_is_dragging();

        if let Some(s) = hypr_socket_request("cursorpos") {
            let parts: Vec<&str> = s.trim().split(',').collect();
            if parts.len() == 2 {
                let cursor_x: f64 = parts[0].trim().parse().unwrap_or(0.0);
                let cursor_y: f64 = parts[1].trim().parse().unwrap_or(200.0);

                let target_edge = if cursor_x < (sw_poll as f64 / 2.0) {
                    "left"
                } else {
                    "right"
                };

                let target_y = (cursor_y as i32 - 80).clamp(60, (sh_poll - collapsed_height - 30).max(60));
                let is_shelf_open = stack_poll.visible_child_name().as_deref() == Some("shelf");

                if !is_shelf_open {
                    if is_dragging {
                        // Drag action occurs (like double tapping and moving object)!
                        // DropBar appears at end of screen!
                        win_poll.set_opacity(1.0);

                        // Check if cursor moved over the edge logo pill
                        let on_logo = if target_edge == "left" {
                            cursor_x <= (collapsed_width as f64 + 40.0) && (cursor_y as i32 - target_y).abs() <= (collapsed_height + 40)
                        } else {
                            cursor_x >= (sw_poll as f64 - collapsed_width as f64 - 40.0) && (cursor_y as i32 - target_y).abs() <= (collapsed_height + 40)
                        };

                        if on_logo {
                            // Dragged onto logo: MAXIMIZE to 1/5 screen ratio shelf!
                            win_poll.set_size_request(expanded_width, expanded_height);
                            win_poll.set_default_size(expanded_width, expanded_height);
                            stack_poll.set_visible_child_name("shelf");
                            let shelf_y = (target_y - 60).clamp(60, (sh_poll - expanded_height - 30).max(60));
                            dock_window_to_pos(target_edge, sw_poll, expanded_width, shelf_y);
                            last_y = shelf_y;
                            last_view = "shelf".to_string();
                        } else {
                            // Only update position when edge changes or cursor moved by >= 20px
                            let edge_changed = target_edge != last_edge;
                            let y_moved = (target_y - last_y).abs() >= 20;
                            let view_changed = last_view != "logo";

                            if edge_changed || y_moved || view_changed {
                                win_poll.set_size_request(collapsed_width, collapsed_height);
                                win_poll.set_default_size(collapsed_width, collapsed_height);
                                stack_poll.set_visible_child_name("logo");
                                dock_window_to_pos(target_edge, sw_poll, collapsed_width, target_y);
                                last_y = target_y;
                                last_view = "logo".to_string();
                            }
                        }
                    } else {
                        // DropBar shouldn't appear while normal cursor movement
                        win_poll.set_opacity(0.0);
                    }
                } else {
                    // Shelf is open
                    let st = state_poll.borrow();
                    let has_files = !st.stashed_files.is_empty();
                    drop(st);

                    if has_files {
                        win_poll.set_opacity(1.0);
                        hide_timer = 0;
                    } else {
                        if !is_dragging {
                            // Drag finished without files: collapse and disappear
                            hide_timer += 1;
                            if hide_timer >= 12 { // ~1.0 second grace period
                                hide_timer = 0;
                                win_poll.set_size_request(collapsed_width, collapsed_height);
                                win_poll.set_default_size(collapsed_width, collapsed_height);
                                stack_poll.set_visible_child_name("logo");
                                win_poll.set_opacity(0.0);
                                last_view = "logo".to_string();
                            }
                        } else {
                            hide_timer = 0;
                            win_poll.set_opacity(1.0);
                        }
                    }
                }

                last_edge = target_edge.to_string();
                *current_edge_poll.borrow_mut() = target_edge;
            }
        }
        glib::ControlFlow::Continue
    });

    // ==========================================
    // INTERACTION 3: MAIN DROP ZONE TARGET
    // ==========================================
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
        let files_to_add = extract_files_from_value(value);
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

    // Add Files Dialog Handler (Nautilus Native File Chooser)
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

    // Open Downloads Handler (Directly launches GNOME Files / Nautilus)
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
                status_lbl_c.remove_css_class("status-searching");
                status_lbl_c.add_css_class("status-badge");
                status_lbl_c.set_text(&format!("🟢 {} ({})", first.name, first.ip));
                send_btn_c.set_sensitive(true);
            } else {
                st.selected_peer = None;
                status_lbl_c.remove_css_class("status-badge");
                status_lbl_c.add_css_class("status-searching");
                status_lbl_c.set_text("🔍 Hotspot...");
                send_btn_c.set_sensitive(false);
            }
        }
    });

    window.present();
}

fn check_is_dragging() -> bool {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());
    let sig = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap_or_default();
    let mut log_path = if !sig.is_empty() {
        format!("{}/hypr/{}/hyprland.log", runtime_dir, sig)
    } else {
        String::new()
    };

    if log_path.is_empty() || !std::path::Path::new(&log_path).exists() {
        if let Ok(entries) = std::fs::read_dir(format!("{}/hypr", runtime_dir)) {
            for entry in entries.flatten() {
                let candidate = entry.path().join("hyprland.log");
                if candidate.exists() {
                    log_path = candidate.to_string_lossy().to_string();
                    break;
                }
            }
        }
    }

    if log_path.is_empty() {
        return false;
    }

    if let Ok(mut file) = std::fs::File::open(&log_path) {
        use std::io::{Read, Seek, SeekFrom};
        let len = file.metadata().map(|m| m.len()).unwrap_or(0);
        let seek_pos = len.saturating_sub(4096);
        let _ = file.seek(SeekFrom::Start(seek_pos));
        let mut raw = Vec::new();
        let _ = file.read_to_end(&mut raw);
        let buf = String::from_utf8_lossy(&raw);
        let mut is_drag = false;
        for line in buf.lines() {
            if line.contains("TAP_STATE_1FGTAP_DRAGGING")
                || line.contains("BUTTON_STATE_BOTTOM")
                || line.contains("BUTTON_EVENT_IN_BOTTOM_L")
            {
                is_drag = true;
            } else if line.contains("TAP_EVENT_RELEASE")
                || line.contains("TAP_STATE_IDLE")
                || line.contains("TAP_STATE_DEAD")
                || line.contains("BUTTON_EVENT_UP")
                || line.contains("BUTTON_STATE_NONE")
            {
                is_drag = false;
            }
        }
        return is_drag;
    }
    false
}

fn hypr_socket_request(cmd: &str) -> Option<String> {
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;

    let runtime = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());
    let sig = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap_or_default();
    let sock_path = format!("{}/hypr/{}/.socket.sock", runtime, sig);

    if let Ok(mut stream) = UnixStream::connect(&sock_path) {
        let _ = stream.write_all(cmd.as_bytes());
        let mut buf = Vec::new();
        let _ = stream.read_to_end(&mut buf);
        return Some(String::from_utf8_lossy(&buf).to_string());
    }
    None
}

fn dock_to_side(side: &str, screen_width: i32, window_width: i32) {
    dock_window_to_pos(side, screen_width, window_width, 160);
}

fn dock_window_to_pos(edge: &str, screen_width: i32, window_width: i32, y: i32) {
    let window_height = if window_width <= 80 { 170 } else { 580 };
    let target_x = if edge == "left" {
        8
    } else {
        (screen_width - window_width - 8).max(10)
    };

    let lua_code = format!(
        "eval local wins = hl.get_windows(); for _, w in ipairs(wins) do if w.class == 'io.dropbar.desktop' then hl.dispatch(hl.dsp.window.resize({{ x = {}, y = {}, window = w }})); hl.dispatch(hl.dsp.window.move({{ x = {}, y = {}, window = w }})); hl.dispatch(hl.dsp.window.bring_to_top({{ window = w }})); break end end",
        window_width, window_height, target_x, y
    );

    let _ = hypr_socket_request(&lua_code);
}

fn extract_files_from_value(value: &glib::Value) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(file_list) = value.get::<gdk::FileList>() {
        for file in file_list.files() {
            if let Some(path) = file.path() {
                files.push(path);
            }
        }
    } else if let Ok(file) = value.get::<gio::File>() {
        if let Some(path) = file.path() {
            files.push(path);
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
                        files.push(path);
                    }
                }
            } else {
                let path = PathBuf::from(trimmed);
                if path.exists() {
                    files.push(path);
                }
            }
        }
    }
    files
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
        empty_lbl.add_css_class("status-searching");
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
