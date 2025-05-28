use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiKnowledgeBase {
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub title: String,
    pub description: String,
    pub status: AiKnowledgeBaseStatus,
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub deployment_id: i64,
    pub usage_count: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AiKnowledgeBaseStatus {
    Processing,
    Ready,
    Error,
}

impl From<String> for AiKnowledgeBaseStatus {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "processing" => AiKnowledgeBaseStatus::Processing,
            "ready" => AiKnowledgeBaseStatus::Ready,
            "error" => AiKnowledgeBaseStatus::Error,
            _ => AiKnowledgeBaseStatus::Processing,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiKnowledgeBaseDocument {
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub knowledge_base_id: i64,
    pub filename: String,
    pub original_filename: String,
    pub file_type: AiKnowledgeBaseDocumentType,
    pub file_size: i64,
    pub file_url: String,
    pub content: Option<String>, // Extracted text content
    pub metadata: Value, // Additional metadata like page count, etc.
    pub status: AiKnowledgeBaseDocumentStatus,
    pub processing_error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AiKnowledgeBaseDocumentType {
    Pdf,
    Markdown,
}

impl From<String> for AiKnowledgeBaseDocumentType {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "pdf" => AiKnowledgeBaseDocumentType::Pdf,
            "markdown" => AiKnowledgeBaseDocumentType::Markdown,
            _ => AiKnowledgeBaseDocumentType::Markdown,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AiKnowledgeBaseDocumentStatus {
    Uploading,
    Processing,
    Ready,
    Error,
}

impl From<String> for AiKnowledgeBaseDocumentStatus {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "uploading" => AiKnowledgeBaseDocumentStatus::Uploading,
            "processing" => AiKnowledgeBaseDocumentStatus::Processing,
            "ready" => AiKnowledgeBaseDocumentStatus::Ready,
            "error" => AiKnowledgeBaseDocumentStatus::Error,
            _ => AiKnowledgeBaseDocumentStatus::Uploading,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateAiKnowledgeBaseRequest {
    pub title: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateAiKnowledgeBaseRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<AiKnowledgeBaseStatus>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UploadKnowledgeBaseDocumentRequest {
    pub filename: String,
    pub file_type: AiKnowledgeBaseDocumentType,
    pub file_size: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiKnowledgeBaseWithDocuments {
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub title: String,
    pub description: String,
    pub status: AiKnowledgeBaseStatus,
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub deployment_id: i64,
    pub usage_count: i32,
    pub documents: Vec<AiKnowledgeBaseDocument>,
}
