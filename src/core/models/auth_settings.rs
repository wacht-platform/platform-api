use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FirstFactor {
    EmailPassword,
    UsernamePassword,
    EmailOtp,
    EmailMagicLink,
    PhoneOtp,
}

impl FromStr for FirstFactor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "email_password" => Ok(FirstFactor::EmailPassword),
            "username_password" => Ok(FirstFactor::UsernamePassword),
            "email_otp" => Ok(FirstFactor::EmailOtp),
            "email_magic_link" => Ok(FirstFactor::EmailMagicLink),
            "phone_otp" => Ok(FirstFactor::PhoneOtp),
            _ => Err(format!("Invalid first factor: {}", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SecondFactor {
    None,
    PhoneOtp,
    BackupCode,
    Authenticator,
}

impl FromStr for SecondFactor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(SecondFactor::None),
            "phone_otp" => Ok(SecondFactor::PhoneOtp),
            "backup_code" => Ok(SecondFactor::BackupCode),
            "authenticator" => Ok(SecondFactor::Authenticator),
            _ => Err(format!("Invalid second factor: {}", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SecondFactorPolicy {
    None,
    Optional,
    Enforced,
}

impl FromStr for SecondFactorPolicy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(SecondFactorPolicy::None),
            "optional" => Ok(SecondFactorPolicy::Optional),
            "enforced" => Ok(SecondFactorPolicy::Enforced),
            _ => Err(format!("Invalid second factor policy: {}", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndividualAuthSettings {
    pub enabled: bool,
    pub required: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PasswordSettings {
    pub enabled: bool,
    pub min_length: Option<u8>,
    pub require_lowercase: Option<bool>,
    pub require_uppercase: Option<bool>,
    pub require_number: Option<bool>,
    pub require_special: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VerificationPolicy {
    pub phone_number: bool,
    pub email: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AuthFactorsEnabled {
    pub sso: bool,
    pub email_password: bool,
    pub username_password: bool,
    pub email_otp: bool,
    pub email_magic_link: bool,
    pub phone_otp: bool,
    pub web3_wallet: bool,
    pub backup_code: bool,
    pub authenticator: bool,
    pub passkey: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmailSettings {
    pub enabled: bool,
    pub required: bool,
    pub verify_signup: Option<bool>,
    pub otp_verification_allowed: Option<bool>,
    pub magic_link_verification_allowed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PhoneSettings {
    pub enabled: bool,
    pub required: bool,
    pub verify_signup: Option<bool>,
    pub sms_verification_allowed: Option<bool>,
    pub whatsapp_verification_allowed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UsernameSettings {
    pub enabled: bool,
    pub required: bool,
    pub min_length: Option<u8>,
    pub max_length: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmailLinkSettings {
    pub enabled: bool,
    pub require_same_device: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PasskeySettings {
    pub enabled: bool,
    pub allow_autofill: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeploymentAuthSettings {
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub email_address: EmailSettings,
    pub phone_number: PhoneSettings,
    pub username: UsernameSettings,
    pub first_name: IndividualAuthSettings,
    pub last_name: IndividualAuthSettings,
    pub password: PasswordSettings,
    pub backup_code: IndividualAuthSettings,
    pub web3_wallet: IndividualAuthSettings,
    pub magic_link: Option<EmailLinkSettings>,
    pub passkey: Option<PasskeySettings>,
    pub auth_factors_enabled: AuthFactorsEnabled,
    pub verification_policy: VerificationPolicy,
    pub second_factor_policy: Option<SecondFactorPolicy>,
    pub first_factor: FirstFactor,
    pub second_factor: Option<SecondFactor>,
    pub alternate_first_factors: Option<Vec<FirstFactor>>,
    pub alternate_second_factors: Option<Vec<SecondFactor>>,
    pub deployment_id: i64,
}
