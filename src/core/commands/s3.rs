use aws_sdk_s3::primitives::{ByteStream, SdkBody};

use crate::application::{AppError, AppState};

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
            .unwrap();

        Ok(format!("https://cdn.wacht.services/{}", self.file_path))
    }
}
