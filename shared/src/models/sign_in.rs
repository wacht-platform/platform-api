use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SignIn {
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub session_id: i64,
    pub user_id: i64,
    pub expired: bool,
    pub expired_at: Option<String>,
}
