use crate::application::{AppError, AppState};
use aws_sdk_s3::primitives::{ByteStream, SdkBody};
use serde_json::json;

use super::Command;

pub struct UploadToCdnCommand {
    pub file_path: String,
    pub body: Vec<u8>,
}

impl UploadToCdnCommand {
    pub fn new(file_path: String, body: Vec<u8>) -> Self {
        Self { file_path, body }
    }
}

impl Command for UploadToCdnCommand {
    type Output = String;

    async fn execute(self, app_state: &AppState) -> Result<Self::Output, AppError> {
        app_state
            .s3_client
            .put_object()
            .bucket(std::env::var("AWS_CDN_BUCKET").expect("AWS_CDN_BUCKET must be set"))
            .key(&self.file_path)
            .body(ByteStream::new(SdkBody::from(self.body)))
            .send()
            .await
            .map_err(|e| AppError::S3(e.to_string()))?;

        let _ = ureq::post("https://api.cloudflare.com/client/v4/zones/90930ab39928937ca4d0c4aba3b03126/purge_cache")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", std::env::var("CLOUDFLARE_API_KEY").expect("CLOUDFLARE_API_KEY must be set")))
            .send_json(json!({
                "files": [
                    format!("https://cdn.wacht.services/{}", self.file_path)
                ]
            }));

        Ok(format!("https://cdn.wacht.services/{}", self.file_path))
    }
}
