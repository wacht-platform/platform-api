use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::OrganizationPermission;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrganizationRole {
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub name: String,
    pub permissions: Vec<OrganizationPermission>,
}
