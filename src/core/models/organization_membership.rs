use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::OrganizationRole;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrganizationMembership {
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub organization_id: i64,
    pub user_id: i64,
    pub roles: Vec<OrganizationRole>,
}
