use serde::{Deserialize, Serialize};

use crate::core::models::{
    EmailLinkSettings, EmailSettings, PasskeySettings, PasswordSettings, PhoneSettings,
    SocialConnectionProvider, UsernameSettings,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct NameSettings {
    pub first_name_enabled: bool,
    pub first_name_required: bool,
    pub last_name_enabled: bool,
    pub last_name_required: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticationFactorSettings {
    pub sso_enabled: Option<bool>,
    pub web3_wallet_enabled: Option<bool>,
    pub magic_link: Option<EmailLinkSettings>,
    pub email_otp_enabled: Option<bool>,
    pub phone_otp_enabled: Option<bool>,
    pub passkey: Option<PasskeySettings>,
    pub second_factor_phone_otp_enabled: Option<bool>,
    pub second_factor_authenticator_enabled: Option<bool>,
    pub second_factor_backup_code_enabled: Option<bool>,
}

// Social Login Settings
#[derive(Debug, Serialize, Deserialize)]
pub struct SocialLoginSettings {
    pub google: Option<SocialProviderConfig>,
    pub github: Option<SocialProviderConfig>,
    pub facebook: Option<SocialProviderConfig>,
    pub apple: Option<SocialProviderConfig>,
    pub microsoft: Option<SocialProviderConfig>,
    pub linkedin: Option<SocialProviderConfig>,
    pub discord: Option<SocialProviderConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SocialProviderConfig {
    pub enabled: bool,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RestrictionSettings {
    pub blocked_countries: Vec<String>,
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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeploymentAuthSettingsUpdates {
    pub email: Option<EmailSettings>,
    pub phone: Option<PhoneSettings>,
    pub username: Option<UsernameSettings>,
    pub password: Option<PasswordSettings>,
    pub name: Option<NameSettings>,
    pub authentication_factors: Option<AuthenticationFactorSettings>,
    pub social_login: Option<SocialLoginSettings>,
    pub restrictions: Option<RestrictionSettings>,
    pub social_connections: Option<Vec<SocialConnectionSettings>>,
}
