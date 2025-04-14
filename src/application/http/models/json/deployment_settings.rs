use serde::{Deserialize, Serialize};

use crate::core::models::{OauthCredentials, SecondFactorPolicy, SocialConnectionProvider};

// Define partial update structs corresponding to core models
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
pub struct SessionSettings {
    pub max_session_age: Option<i64>,
    pub inactivity_timeout: Option<i64>,
    pub allow_multi_account_session: Option<bool>,
    pub default_jwt_template: Option<String>,
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
    pub restrictions: Option<RestrictionSettings>,
    pub session: Option<SessionSettings>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub second_factor_policy: Option<SecondFactorPolicy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backup_code: Option<PartialIndividualAuthSettings>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web3_wallet: Option<PartialIndividualAuthSettings>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeploymentSocialConnectionUpsert {
    pub provider: Option<SocialConnectionProvider>,
    pub enabled: Option<bool>,
    pub user_defined_scopes: Option<Vec<String>>,
    pub credentials: Option<OauthCredentials>,
}
