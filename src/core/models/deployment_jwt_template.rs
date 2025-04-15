use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CustomSigningKey {
    pub key: String,
    pub algorithm: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeploymentJwtTemplate {
    pub id: i64,
    pub name: String,
    pub token_lifetime: i64,
    pub allowed_clock_skew: i64,
    pub custom_signing_key: Option<CustomSigningKey>,
    pub template: Value,
    pub deployment_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}
