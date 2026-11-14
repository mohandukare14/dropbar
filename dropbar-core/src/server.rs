use crate::models::{DeviceInfo, TextPayload, TransferResponse};
use axum::{
    extract::{DefaultBodyLimit, Multipart, Path as AxumPath, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use sha2::{Digest, Sha256};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::{broadcast, RwLock};
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

const MOBILE_WEB_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
    <title>DropBar - P4 Web Drop</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }
        body { background-color: #0F141C; color: #F8FAFC; padding: 20px; display: flex; flex-direction: column; min-height: 100vh; }
        .header { display: flex; align-items: center; justify-content: space-between; padding-bottom: 16px; border-bottom: 1px solid #1E293B; margin-bottom: 20px; }
        .logo { font-size: 20px; font-weight: 800; color: #00FFCC; display: flex; align-items: center; gap: 8px; }
        .badge { background: rgba(0, 255, 204, 0.15); color: #00FFCC; padding: 4px 10px; border-radius: 20px; font-size: 12px; font-weight: 600; }
        .drop-zone { border: 2px dashed #00FFCC; border-radius: 16px; background: rgba(30, 41, 59, 0.5); padding: 36px 20px; text-align: center; cursor: pointer; transition: all 0.2s ease; margin-bottom: 20px; }
        .drop-zone.hover { background: rgba(0, 255, 204, 0.2); transform: scale(1.01); }
        .drop-icon { font-size: 48px; margin-bottom: 12px; display: block; }
        .drop-title { font-size: 16px; font-weight: 700; margin-bottom: 6px; }
        .drop-sub { font-size: 13px; color: #94A3B8; }
        .progress-bar { width: 100%; height: 8px; background: #1E293B; border-radius: 4px; overflow: hidden; margin: 12px 0; display: none; }
        .progress-fill { height: 100%; width: 0%; background: #00FFCC; transition: width 0.2s ease; }
        .text-card { background: #161E2E; border: 1px solid #1E293B; border-radius: 14px; padding: 14px; margin-bottom: 20px; }
        .text-input { width: 100%; background: #0F141C; border: 1px solid #334155; border-radius: 8px; color: #fff; padding: 10px 12px; font-size: 14px; margin-bottom: 10px; resize: none; }
        .btn { width: 100%; background: #00FFCC; color: #0F172A; font-weight: 700; border: none; padding: 12px; border-radius: 8px; font-size: 14px; cursor: pointer; }
        .btn:active { transform: scale(0.98); }
        .history-title { font-size: 12px; font-weight: 700; color: #64748B; letter-spacing: 0.05em; text-transform: uppercase; margin-bottom: 10px; }
        .history-item { background: #1E293B; border-radius: 10px; padding: 12px; display: flex; align-items: center; justify-content: space-between; margin-bottom: 8px; }
        .history-name { font-size: 13px; font-weight: 600; }
        .history-meta { font-size: 11px; color: #94A3B8; }
    </style>
</head>
<body>
    <div class="header">
        <div class="logo">⚡ DropBar</div>
        <div class="badge" id="connStatus">🟢 Connected to Arch</div>
    </div>

    <div class="drop-zone" id="dropZone" onclick="document.getElementById('fileInput').click()">
        <span class="drop-icon">📤</span>
        <div class="drop-title">Tap to Choose or Drop Files</div>
        <div class="drop-sub">Photos, videos, documents sent instantly to Arch Linux</div>
        <input type="file" id="fileInput" multiple style="display: none" onchange="uploadFiles(this.files)">
    </div>

    <div class="progress-bar" id="progressBar">
        <div class="progress-fill" id="progressFill"></div>
    </div>

    <div class="text-card">
        <textarea class="text-input" id="textInput" rows="2" placeholder="Beam link or text snippet to Arch..."></textarea>
        <button class="btn" onclick="sendText()">⚡ Beam Text</button>
    </div>

    <div class="history-title">Recent Transfers</div>
    <div id="historyList"></div>

    <script>
        // Announce presence on load
        fetch('/api/v1/register', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ name: "P4 (Mobile)", device_type: "Android", is_web: true })
        }).catch(() => {});

        // Drag and drop handlers
        const dropZone = document.getElementById('dropZone');
        ['dragenter', 'dragover'].forEach(e => dropZone.addEventListener(e, (ev) => { ev.preventDefault(); dropZone.classList.add('hover'); }));
        ['dragleave', 'drop'].forEach(e => dropZone.addEventListener(e, (ev) => { ev.preventDefault(); dropZone.classList.remove('hover'); }));
        dropZone.addEventListener('drop', (ev) => {
            if (ev.dataTransfer.files.length) uploadFiles(ev.dataTransfer.files);
        });

        function uploadFiles(files) {
            if (!files.length) return;
            const pBar = document.getElementById('progressBar');
            const pFill = document.getElementById('progressFill');
            pBar.style.display = 'block';

            Array.from(files).forEach((file, index) => {
                const formData = new FormData();
                formData.append('sender', 'P4 (Mobile)');
                formData.append('file', file);

                const xhr = new XMLHttpRequest();
                xhr.open('POST', '/api/v1/transfer/file', true);
                xhr.upload.onprogress = (e) => {
                    if (e.lengthComputable) {
                        const percent = (e.loaded / e.total) * 100;
                        pFill.style.width = percent + '%';
                    }
                };
                xhr.onload = () => {
                    if (xhr.status === 200) {
                        addHistory(file.name, (file.size / 1024).toFixed(1) + ' KB', '✅ Sent to Arch');
                        pBar.style.display = 'none';
                        pFill.style.width = '0%';
                    }
                };
                xhr.send(formData);
            });
        }

        function sendText() {
            const input = document.getElementById('textInput');
            const val = input.value.trim();
            if (!val) return;
            fetch('/api/v1/transfer/text', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ text: val, is_url: val.startsWith('http'), sender_name: 'P4 (Mobile)' })
            }).then(r => r.json()).then(res => {
                if (res.success) {
                    addHistory(val.substring(0, 30) + '...', 'Text snippet', '✅ Sent to Arch');
                    input.value = '';
                }
            });
        }

        function addHistory(title, meta, status) {
            const list = document.getElementById('historyList');
            const item = document.createElement('div');
            item.className = 'history-item';
            item.innerHTML = `<div><div class="history-name">${title}</div><div class="history-meta">${meta}</div></div><div class="badge">${status}</div>`;
            list.prepend(item);
        }
    </script>
</body>
</html>"#;

#[derive(Debug, Clone)]
pub enum IncomingPayload {
    File {
        filename: String,
        path: PathBuf,
        size: u64,
        sha256: String,
        sender: Option<String>,
    },
    Text {
        text: String,
        is_url: bool,
        sender: String,
    },
    DeviceRegistered(DeviceInfo),
}

#[derive(Clone)]
pub struct ServerState {
    pub device_info: DeviceInfo,
    pub save_dir: PathBuf,
    pub event_tx: broadcast::Sender<IncomingPayload>,
    pub stashed_files: Arc<RwLock<Vec<PathBuf>>>,
}

pub struct DropBarServer {
    state: ServerState,
    port: u16,
}

impl DropBarServer {
    pub fn new(device_info: DeviceInfo, save_dir: PathBuf, port: u16) -> (Self, broadcast::Receiver<IncomingPayload>) {
        let (event_tx, event_rx) = broadcast::channel(64);
        let state = ServerState {
            device_info,
            save_dir,
            event_tx,
            stashed_files: Arc::new(RwLock::new(Vec::new())),
        };
        (Self { state, port }, event_rx)
    }

    pub fn get_stashed_files_handle(&self) -> Arc<RwLock<Vec<PathBuf>>> {
        self.state.stashed_files.clone()
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tokio::fs::create_dir_all(&self.state.save_dir).await?;

        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        let app = Router::new()
            .route("/", get(web_portal_handler))
            .route("/api/v1/ping", get(ping_handler))
            .route("/api/v1/register", post(register_handler))
            .route("/api/v1/transfer/text", post(transfer_text_handler))
            .route("/api/v1/transfer/file", post(transfer_file_handler))
            .route("/api/v1/download/:filename", get(download_file_handler))
            .layer(DefaultBodyLimit::max(10 * 1024 * 1024 * 1024)) // 10 GB limit
            .layer(cors)
            .with_state(self.state.clone());

        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        info!("DropBar Server listening on http://{}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

async fn web_portal_handler() -> Html<&'static str> {
    Html(MOBILE_WEB_HTML)
}

async fn ping_handler(State(state): State<ServerState>) -> Json<DeviceInfo> {
    Json(state.device_info)
}

#[derive(serde::Deserialize)]
struct RegisterRequest {
    name: Option<String>,
}

async fn register_handler(
    State(state): State<ServerState>,
    Json(payload): Json<RegisterRequest>,
) -> Json<TransferResponse> {
    let peer_name = payload.name.unwrap_or_else(|| "P4 (Mobile)".to_string());
    info!("Registered peer via Web Portal: {}", peer_name);

    let device = DeviceInfo {
        id: format!("web_{}", uuid::Uuid::new_v4()),
        name: peer_name,
        device_type: crate::models::DeviceType::Android,
        ip: "Mobile Hotspot Client".to_string(),
        port: 53319,
        last_seen: chrono::Utc::now(),
    };

    let _ = state.event_tx.send(IncomingPayload::DeviceRegistered(device));

    Json(TransferResponse {
        success: true,
        message: "Connected to DropBar on Arch Linux".to_string(),
        payload_id: None,
    })
}

async fn transfer_text_handler(
    State(state): State<ServerState>,
    Json(payload): Json<TextPayload>,
) -> (StatusCode, Json<TransferResponse>) {
    info!("Received text/url payload from: {}", payload.sender_name);

    let event = IncomingPayload::Text {
        text: payload.text.clone(),
        is_url: payload.is_url,
        sender: payload.sender_name.clone(),
    };

    let _ = state.event_tx.send(event);

    (
        StatusCode::OK,
        Json(TransferResponse {
            success: true,
            message: "Payload received".to_string(),
            payload_id: None,
        }),
    )
}

async fn transfer_file_handler(
    State(state): State<ServerState>,
    mut multipart: Multipart,
) -> Result<Json<TransferResponse>, (StatusCode, Json<TransferResponse>)> {
    let mut sender_name = "P4 (Mobile)".to_string();
    let mut saved_files = Vec::new();

    while let Ok(Some(mut field)) = multipart.next_field().await {
        let field_name = field.name().unwrap_or("").to_string();

        if field_name == "sender" {
            if let Ok(text) = field.text().await {
                sender_name = text;
            }
            continue;
        }

        if field_name == "file" {
            let filename = field
                .file_name()
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("dropbar_received_{}", uuid::Uuid::new_v4()));

            let clean_filename = Path::new(&filename)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| "unnamed_file".to_string());

            let mut dest_path = state.save_dir.join(&clean_filename);

            if dest_path.exists() {
                let stem = Path::new(&clean_filename)
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy();
                let ext = Path::new(&clean_filename)
                    .extension()
                    .map(|e| format!(".{}", e.to_string_lossy()))
                    .unwrap_or_default();
                let timestamp = chrono::Utc::now().timestamp_millis();
                dest_path = state.save_dir.join(format!("{}_{}{}", stem, timestamp, ext));
            }

            let mut file = match File::create(&dest_path).await {
                Ok(f) => f,
                Err(e) => {
                    error!("Failed to create destination file: {}", e);
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(TransferResponse {
                            success: false,
                            message: format!("Failed to create file: {}", e),
                            payload_id: None,
                        }),
                    ));
                }
            };

            let mut hasher = Sha256::new();
            let mut total_bytes = 0u64;

            while let Ok(Some(chunk)) = field.chunk().await {
                hasher.update(&chunk);
                total_bytes += chunk.len() as u64;
                if let Err(e) = file.write_all(&chunk).await {
                    error!("Write error: {}", e);
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(TransferResponse {
                            success: false,
                            message: format!("Failed to write chunk: {}", e),
                            payload_id: None,
                        }),
                    ));
                }
            }

            let _ = file.flush().await;
            let hash_hex = hex::encode(hasher.finalize());

            info!("Successfully saved file: {} ({} bytes)", dest_path.display(), total_bytes);

            let event = IncomingPayload::File {
                filename: clean_filename,
                path: dest_path.clone(),
                size: total_bytes,
                sha256: hash_hex,
                sender: Some(sender_name.clone()),
            };

            let _ = state.event_tx.send(event);
            saved_files.push(dest_path);
        }
    }

    Ok(Json(TransferResponse {
        success: true,
        message: format!("Successfully transferred {} file(s)", saved_files.len()),
        payload_id: None,
    }))
}

async fn download_file_handler(
    State(state): State<ServerState>,
    AxumPath(filename): AxumPath<String>,
) -> Response {
    let clean = Path::new(&filename)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();

    let path = state.save_dir.join(&clean);
    if !path.exists() {
        return (StatusCode::NOT_FOUND, "File not found").into_response();
    }

    match tokio::fs::read(&path).await {
        Ok(bytes) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", clean).parse().unwrap(),
            );
            headers.insert(header::CONTENT_TYPE, "application/octet-stream".parse().unwrap());
            (headers, bytes).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file").into_response(),
    }
}
