use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiTool {
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub description: Option<String>,
    pub tool_type: AiToolType,
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub deployment_id: i64,
    pub configuration: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiToolWithDetails {
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub description: Option<String>,
    pub tool_type: AiToolType,
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub deployment_id: i64,
    pub configuration: serde_json::Value,
    pub usage_count: i64,
    pub last_used: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AiToolType {
    Api,
    Function,
    Database,
    External,
}

impl From<String> for AiToolType {
    fn from(tool_type: String) -> Self {
        match tool_type.as_str() {
            "api" => AiToolType::Api,
            "function" => AiToolType::Function,
            "database" => AiToolType::Database,
            "external" => AiToolType::External,
            _ => AiToolType::Api,
        }
    }
}

impl From<AiToolType> for String {
    fn from(tool_type: AiToolType) -> Self {
        match tool_type {
            AiToolType::Api => "api".to_string(),
            AiToolType::Function => "function".to_string(),
            AiToolType::Database => "database".to_string(),
            AiToolType::External => "external".to_string(),
        }
    }
}


