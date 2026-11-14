use crate::models::{DeviceInfo, TextPayload, TransferResponse};
use reqwest::multipart::{Form, Part};
use std::path::Path;
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use tracing::info;

#[derive(Clone)]
pub struct DropBarClient {
    client: reqwest::Client,
    local_name: String,
}

impl DropBarClient {
    pub fn new(local_name: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .unwrap_or_default(),
            local_name: local_name.into(),
        }
    }

    pub async fn ping(&self, target: &DeviceInfo) -> Result<DeviceInfo, reqwest::Error> {
        let url = format!("http://{}:{}/api/v1/ping", target.ip, target.port);
        let resp = self.client.get(&url).send().await?;
        resp.json::<DeviceInfo>().await
    }

    pub async fn send_text(
        &self,
        target: &DeviceInfo,
        text: &str,
        is_url: bool,
    ) -> Result<TransferResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("http://{}:{}/api/v1/transfer/text", target.ip, target.port);
        let payload = TextPayload {
            text: text.to_string(),
            is_url,
            sender_name: self.local_name.clone(),
        };

        let resp = self.client.post(&url).json(&payload).send().await?;
        let result = resp.json::<TransferResponse>().await?;
        Ok(result)
    }

    pub async fn send_file(
        &self,
        target: &DeviceInfo,
        file_path: &Path,
    ) -> Result<TransferResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("http://{}:{}/api/v1/transfer/file", target.ip, target.port);
        let filename = file_path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "file.bin".to_string());

        let file = File::open(file_path).await?;
        let stream = ReaderStream::new(file);
        let body = reqwest::Body::wrap_stream(stream);

        let part = Part::stream(body).file_name(filename.clone());
        let form = Form::new()
            .text("sender", self.local_name.clone())
            .part("file", part);

        info!("Sending file '{}' to {}:{}", filename, target.name, url);
        let resp = self.client.post(&url).multipart(form).send().await?;
        let result = resp.json::<TransferResponse>().await?;
        Ok(result)
    }
}

