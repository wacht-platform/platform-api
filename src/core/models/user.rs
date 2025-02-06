use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::SecondFactorPolicy;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStrategy {
    Otp,
    OauthGoogle,
    OauthGithub,
    OauthMicrosoft,
    OauthFacebook,
    OauthLinkedin,
    OauthDiscord,
    OauthApple,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SchemaVersion {
    V1,
    V2,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserEmailAddress {
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deployment_id: i64,
    pub user_id: i64,
    pub email: String,
    pub is_primary: bool,
    pub verified: bool,
    pub verified_at: DateTime<Utc>,
    pub verification_strategy: VerificationStrategy,
    pub social_connection_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub password: String,
    pub schema_version: SchemaVersion,
    pub disabled: bool,
    pub primary_email_address_id: Option<i64>,
    pub primary_phone_number_id: Option<i64>,
    pub second_factor_policy: SecondFactorPolicy,
    pub active_organization_membership_id: Option<i64>,
    pub active_workspace_membership_id: Option<i64>,
    pub deployment_id: i64,
    pub public_metadata: Value,
    pub private_metadata: Value,
    pub otp_secret: String,
    pub backup_codes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserWithIdentifiers {
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub first_name: String,
    pub last_name: String,
    pub username: Option<String>,
    pub primary_email_address: Option<String>,
    pub primary_phone_number: Option<String>,
}
