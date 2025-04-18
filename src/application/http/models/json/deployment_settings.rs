use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::models::{
    CountryRestrictions, CustomSigningKey, DeploymentRestrictionsSignUpMode, MultiSessionSupport,
    OauthCredentials, SecondFactorPolicy, SocialConnectionProvider,
};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PartialEmailSettings {
    pub enabled: Option<bool>,
    pub required: Option<bool>,
    pub verify_signup: Option<bool>,
    pub otp_verification_allowed: Option<bool>,
    pub magic_link_verification_allowed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PartialPhoneSettings {
    pub enabled: Option<bool>,
    pub required: Option<bool>,
    pub verify_signup: Option<bool>,
    pub sms_verification_allowed: Option<bool>,
    pub whatsapp_verification_allowed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PartialUsernameSettings {
    pub enabled: Option<bool>,
    pub required: Option<bool>,
    pub min_length: Option<u8>,
    pub max_length: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PartialPasswordSettings {
    pub enabled: Option<bool>,
    pub min_length: Option<u8>,
    pub require_lowercase: Option<bool>,
    pub require_uppercase: Option<bool>,
    pub require_number: Option<bool>,
    pub require_special: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PartialNameSettings {
    pub first_name_enabled: Option<bool>,
    pub first_name_required: Option<bool>,
    pub last_name_enabled: Option<bool>,
    pub last_name_required: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PartialEmailLinkSettings {
    pub enabled: Option<bool>,
    pub require_same_device: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PartialPasskeySettings {
    pub enabled: Option<bool>,
    pub allow_autofill: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct PartialIndividualAuthSettings {
    pub enabled: Option<bool>,
    pub required: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PartialAuthenticationFactorSettings {
    pub email_password_enabled: Option<bool>,
    pub username_password_enabled: Option<bool>,
    pub sso_enabled: Option<bool>,
    pub web3_wallet_enabled: Option<bool>,
    pub email_otp_enabled: Option<bool>,
    pub phone_otp_enabled: Option<bool>,
    pub magic_link: Option<PartialEmailLinkSettings>,
    pub passkey: Option<PartialPasskeySettings>,
    pub second_factor_authenticator_enabled: Option<bool>,
    pub second_factor_backup_code_enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SocialProviderConfig {
    pub enabled: bool,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RestrictionSettings {
    pub blocked_country_codes: Vec<String>,
    pub banned_keywords: Vec<String>,
    pub email_allowlist: Vec<String>,
    pub email_blocklist: Vec<String>,
    pub block_voip_numbers: bool,
    pub restrict_signup: bool,
    pub block_temporary_emails: bool,
    pub block_special_characters_in_email: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SocialConnectionSettings {
    #[serde(
        default,
        deserialize_with = "crate::utils::serde::enum_from_str::from_str_option"
    )]
    pub provider: Option<SocialConnectionProvider>,
    pub id: Option<String>,
    pub client_id: String,
    pub client_secret: String,
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DeploymentAuthSettingsUpdates {
    pub email: Option<PartialEmailSettings>,
    pub phone: Option<PartialPhoneSettings>,
    pub username: Option<PartialUsernameSettings>,
    pub password: Option<PartialPasswordSettings>,
    pub name: Option<PartialNameSettings>,
    pub authentication_factors: Option<PartialAuthenticationFactorSettings>,
    pub second_factor_policy: Option<SecondFactorPolicy>,
    pub backup_code: Option<PartialIndividualAuthSettings>,
    pub web3_wallet: Option<PartialIndividualAuthSettings>,
    pub multi_session_support: Option<MultiSessionSupport>,
    pub session_token_lifetime: Option<i64>,
    pub session_validity_period: Option<i64>,
    pub session_inactive_timeout: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeploymentSocialConnectionUpsert {
    pub provider: Option<SocialConnectionProvider>,
    pub enabled: Option<bool>,
    pub user_defined_scopes: Option<Vec<String>>,
    pub credentials: Option<OauthCredentials>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeploymentRestrictionsUpdates {
    pub allowlist_enabled: Option<bool>,
    pub blocklist_enabled: Option<bool>,
    pub block_subaddresses: Option<bool>,
    pub block_disposable_emails: Option<bool>,
    pub block_voip_numbers: Option<bool>,
    pub country_restrictions: Option<CountryRestrictions>,
    pub banned_keywords: Option<Vec<String>>,
    pub allowlisted_resources: Option<Vec<String>>,
    pub blocklisted_resources: Option<Vec<String>>,
    pub sign_up_mode: Option<DeploymentRestrictionsSignUpMode>,
    pub multi_session_support: Option<MultiSessionSupport>,
    pub session_token_lifetime: Option<i64>,
    pub session_validity_period: Option<i64>,
    pub session_inactive_timeout: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewDeploymentJwtTemplate {
    pub name: String,
    pub token_lifetime: i64,
    pub allowed_clock_skew: i64,
    pub custom_signing_key: Option<CustomSigningKey>,
    pub template: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PartialDeploymentJwtTemplate {
    pub name: Option<String>,
    pub token_lifetime: Option<i64>,
    pub allowed_clock_skew: Option<i64>,
    pub custom_signing_key: Option<CustomSigningKey>,
    pub template: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeploymentB2bSettingsUpdates {
    pub organizations_enabled: Option<bool>,
    pub workspaces_enabled: Option<bool>,
    pub ip_allowlist_per_org_enabled: Option<bool>,
    pub allow_users_to_create_orgs: Option<bool>,
    pub max_allowed_org_members: Option<i64>,
    pub max_allowed_workspace_members: Option<i64>,
    pub allow_org_deletion: Option<bool>,
    pub allow_workspace_deletion: Option<bool>,
    pub custom_org_role_enabled: Option<bool>,
    pub custom_workspace_role_enabled: Option<bool>,
    #[serde(
        deserialize_with = "crate::utils::serde::i64_as_string_option::deserialize",
        default
    )]
    pub default_workspace_creator_role_id: Option<i64>,
    #[serde(
        deserialize_with = "crate::utils::serde::i64_as_string_option::deserialize",
        default
    )]
    pub default_workspace_member_role_id: Option<i64>,
    #[serde(
        deserialize_with = "crate::utils::serde::i64_as_string_option::deserialize",
        default
    )]
    pub default_org_creator_role_id: Option<i64>,
    #[serde(
        deserialize_with = "crate::utils::serde::i64_as_string_option::deserialize",
        default
    )]
    pub default_org_member_role_id: Option<i64>,
    pub limit_org_creation_per_user: Option<bool>,
    pub limit_workspace_creation_per_org: Option<bool>,
    pub org_creation_per_user_count: Option<i32>,
    pub workspaces_per_org_count: Option<i32>,
}
