use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeploymentInvitation {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deployment_id: i64,
    pub first_name: String,
    pub last_name: String,
    pub email_address: String,
    pub expiry: DateTime<Utc>,
}
