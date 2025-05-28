use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiAgent {
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub description: String,
    pub status: AiAgentStatus,
    pub configuration: Value,
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub deployment_id: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AiAgentStatus {
    Active,
    Inactive,
    Draft,
}

impl From<String> for AiAgentStatus {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "active" => AiAgentStatus::Active,
            "inactive" => AiAgentStatus::Inactive,
            "draft" => AiAgentStatus::Draft,
            _ => AiAgentStatus::Draft,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiAgentWithDetails {
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub description: String,
    pub status: AiAgentStatus,
    pub configuration: Value,
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub deployment_id: i64,
    pub tools_count: i32,
    pub workflows_count: i32,
    pub knowledge_bases_count: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateAiAgentRequest {
    pub name: String,
    pub description: String,
    pub configuration: Value,
    pub tool_ids: Vec<i64>,
    pub workflow_ids: Vec<i64>,
    pub knowledge_base_ids: Vec<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateAiAgentRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<AiAgentStatus>,
    pub configuration: Option<Value>,
    pub tool_ids: Option<Vec<i64>>,
    pub workflow_ids: Option<Vec<i64>>,
    pub knowledge_base_ids: Option<Vec<i64>>,
}
