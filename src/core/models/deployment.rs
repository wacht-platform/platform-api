use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{
    DeploymentAuthSettings, DeploymentDisplaySettings, DeploymentOrgSettings,
    DeploymentRestrictions,
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentMode {
    Production,
    Staging,
}

impl From<String> for DeploymentMode {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "production" => DeploymentMode::Production,
            "staging" => DeploymentMode::Staging,
            _ => panic!("Invalid deployment mode: {}", value),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Deployment {
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub maintenance_mode: bool,
    pub host: String,
    pub publishable_key: String,
    pub secret: String,
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub project_id: i64,
    pub mode: DeploymentMode,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeploymentWithSettings {
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub maintenance_mode: bool,
    pub host: String,
    pub publishable_key: String,
    pub secret: String,
    pub mode: DeploymentMode,
    pub auth_settings: Option<DeploymentAuthSettings>,
    pub display_settings: Option<DeploymentDisplaySettings>,
    pub org_settings: Option<DeploymentOrgSettings>,
    pub restrictions: Option<DeploymentRestrictions>,
}
