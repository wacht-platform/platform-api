use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiTool {
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub description: String,
    pub tool_type: AiToolType,
    pub status: AiToolStatus,
    pub configuration: Value, // Stores tool-specific configuration
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub deployment_id: i64,
    pub usage_count: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AiToolType {
    Api,
    Function,
    Database,
    External,
}

impl From<String> for AiToolType {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "api" => AiToolType::Api,
            "function" => AiToolType::Function,
            "database" => AiToolType::Database,
            "external" => AiToolType::External,
            _ => AiToolType::Function,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AiToolStatus {
    Active,
    Inactive,
    Draft,
}

impl From<String> for AiToolStatus {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "active" => AiToolStatus::Active,
            "inactive" => AiToolStatus::Inactive,
            "draft" => AiToolStatus::Draft,
            _ => AiToolStatus::Draft,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateAiToolRequest {
    pub name: String,
    pub description: String,
    pub tool_type: AiToolType,
    pub configuration: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateAiToolRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tool_type: Option<AiToolType>,
    pub status: Option<AiToolStatus>,
    pub configuration: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiToolExecution {
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub tool_id: i64,
    #[serde(with = "crate::utils::serde::i64_as_string_option", default)]
    pub agent_id: Option<i64>,
    #[serde(with = "crate::utils::serde::i64_as_string_option", default)]
    pub workflow_run_id: Option<i64>,
    pub input_data: Value,
    pub output_data: Option<Value>,
    pub status: AiToolExecutionStatus,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub execution_time_ms: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AiToolExecutionStatus {
    Running,
    Completed,
    Failed,
}

impl From<String> for AiToolExecutionStatus {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "running" => AiToolExecutionStatus::Running,
            "completed" => AiToolExecutionStatus::Completed,
            "failed" => AiToolExecutionStatus::Failed,
            _ => AiToolExecutionStatus::Failed,
        }
    }
}
