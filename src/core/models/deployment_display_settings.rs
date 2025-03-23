use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ButtonConfig {
    pub background_color: String,
    pub text_color: String,
    pub border_radius: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputConfig {
    pub placeholder: String,
    pub text_color: String,
    pub border_color: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeploymentDisplaySettings {
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deployment_id: i64,
    pub app_name: String,
    pub primary_color: String,
    pub tos_page_url: Option<String>,
    pub sign_in_page_url: Option<String>,
    pub sign_up_page_url: Option<String>,
    pub after_sign_out_one_page_url: Option<String>,
    pub after_sign_out_all_page_url: Option<String>,
    pub favicon_image_url: Option<String>,
    pub logo_image_url: Option<String>,
    pub privacy_policy_url: Option<String>,
    pub signup_terms_statement: Option<String>,
    pub signup_terms_statement_shown: bool,
    pub button_config: ButtonConfig,
    pub input_config: InputConfig,
}
