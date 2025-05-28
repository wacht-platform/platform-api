use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiWorkflow {
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub description: String,
    pub status: AiWorkflowStatus,
    pub configuration: Value, // Stores the workflow definition (nodes, edges, etc.)
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub deployment_id: i64,
    pub last_run_at: Option<DateTime<Utc>>,
    pub run_count: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AiWorkflowStatus {
    Active,
    Inactive,
    Draft,
}

impl From<String> for AiWorkflowStatus {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "active" => AiWorkflowStatus::Active,
            "inactive" => AiWorkflowStatus::Inactive,
            "draft" => AiWorkflowStatus::Draft,
            _ => AiWorkflowStatus::Draft,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiWorkflowWithDetails {
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub description: String,
    pub status: AiWorkflowStatus,
    pub configuration: Value,
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub deployment_id: i64,
    pub last_run_at: Option<DateTime<Utc>>,
    pub run_count: i32,
    pub agents_count: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateAiWorkflowRequest {
    pub name: String,
    pub description: String,
    pub configuration: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateAiWorkflowRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<AiWorkflowStatus>,
    pub configuration: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiWorkflowRun {
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub workflow_id: i64,
    pub status: AiWorkflowRunStatus,
    pub input_data: Value,
    pub output_data: Option<Value>,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AiWorkflowRunStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl From<String> for AiWorkflowRunStatus {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "running" => AiWorkflowRunStatus::Running,
            "completed" => AiWorkflowRunStatus::Completed,
            "failed" => AiWorkflowRunStatus::Failed,
            "cancelled" => AiWorkflowRunStatus::Cancelled,
            _ => AiWorkflowRunStatus::Failed,
        }
    }
}
