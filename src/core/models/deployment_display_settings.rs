use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ButtonConfig {
    pub background_color: String,
    pub text_color: String,
    pub border_radius: i32,
}

impl Default for ButtonConfig {
    fn default() -> Self {
        Self {
            background_color: "#0066FF".to_string(),
            text_color: "#FFFFFF".to_string(),
            border_radius: 4,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputConfig {
    pub placeholder: String,
    pub text_color: String,
    pub border_color: String,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            placeholder: "".to_string(),
            text_color: "#000000".to_string(),
            border_color: "#E5E7EB".to_string(),
        }
    }
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
    pub tos_page_url: String,
    pub sign_in_page_url: String,
    pub sign_up_page_url: String,
    pub after_sign_out_one_page_url: String,
    pub after_sign_out_all_page_url: String,
    pub favicon_image_url: String,
    pub logo_image_url: String,
    pub privacy_policy_url: String,
    pub signup_terms_statement: String,
    pub signup_terms_statement_shown: bool,
    pub button_config: ButtonConfig,
    pub input_config: InputConfig,
}

impl Default for DeploymentDisplaySettings {
    fn default() -> Self {
        Self {
            id: 0,
            created_at: None,
            updated_at: None,
            deleted_at: None,
            deployment_id: 0,
            app_name: "".to_string(),
            primary_color: "#0066FF".to_string(),
            tos_page_url: "".to_string(),
            sign_in_page_url: "".to_string(),
            sign_up_page_url: "".to_string(),
            after_sign_out_one_page_url: "".to_string(),
            after_sign_out_all_page_url: "".to_string(),
            favicon_image_url: "".to_string(),
            logo_image_url: "".to_string(),
            privacy_policy_url: "".to_string(),
            signup_terms_statement: "I agree to the Terms of Service and Privacy Policy"
                .to_string(),
            signup_terms_statement_shown: true,
            button_config: ButtonConfig::default(),
            input_config: InputConfig::default(),
        }
    }
}
