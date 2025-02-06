use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SocialConnectionProvider {
    XOauth,
    GithubOauth,
    GitlabOauth,
    GoogleOauth,
    FacebookOauth,
    MicrosoftOauth,
    LinkedinOauth,
    DiscordOauth,
    AppleOauth,
}

impl FromStr for SocialConnectionProvider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "x_oauth" => Ok(SocialConnectionProvider::XOauth),
            "github_oauth" => Ok(SocialConnectionProvider::GithubOauth),
            "gitlab_oauth" => Ok(SocialConnectionProvider::GitlabOauth),
            "google_oauth" => Ok(SocialConnectionProvider::GoogleOauth),
            "facebook_oauth" => Ok(SocialConnectionProvider::FacebookOauth),
            "microsoft_oauth" => Ok(SocialConnectionProvider::MicrosoftOauth),
            "linkedin_oauth" => Ok(SocialConnectionProvider::LinkedinOauth),
            "discord_oauth" => Ok(SocialConnectionProvider::DiscordOauth),
            "apple_oauth" => Ok(SocialConnectionProvider::AppleOauth),
            _ => Err(format!("Invalid social connection provider: {}", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct OauthCredentials {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeploymentSocialConnection {
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deployment_id: Option<i64>,
    pub provider: Option<SocialConnectionProvider>,
    pub enabled: Option<bool>,
    pub user_defined_scopes: Option<Vec<String>>,
    pub credentials: Option<OauthCredentials>,
}
