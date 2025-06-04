use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiTool {
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub description: Option<String>,
    pub tool_type: AiToolType,
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub deployment_id: i64,
    pub configuration: AiToolConfiguration,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiToolWithDetails {
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub description: Option<String>,
    pub tool_type: AiToolType,
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub deployment_id: i64,
    pub configuration: AiToolConfiguration,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AiToolType {
    Api,
    KnowledgeBase,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum AiToolConfiguration {
    Api(ApiToolConfiguration),
    KnowledgeBase(KnowledgeBaseToolConfiguration),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiToolConfiguration {
    pub endpoint: String,
    pub method: HttpMethod,
    pub headers: Vec<HttpParameter>,
    pub query_parameters: Vec<HttpParameter>,
    pub body_parameters: Vec<HttpParameter>,
    pub authorization: Option<AuthorizationConfiguration>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KnowledgeBaseToolConfiguration {
    #[serde(with = "crate::utils::serde::i64_as_string")]
    pub knowledge_base_id: i64,
    pub search_settings: KnowledgeBaseSearchSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KnowledgeBaseSearchSettings {
    pub max_results: Option<u32>,
    pub similarity_threshold: Option<f32>,
    pub include_metadata: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HttpParameter {
    pub name: String,
    pub value_type: ParameterValueType,
    pub required: bool,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ParameterValueType {
    Hardcoded { value: String },
    FromChat { lookup_key: String },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthorizationConfiguration {
    pub authorize_as_user: bool,
    pub jwt_template_id: Option<i64>,
    pub custom_headers: Vec<HttpParameter>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

impl From<String> for AiToolType {
    fn from(tool_type: String) -> Self {
        match tool_type.as_str() {
            "api" => AiToolType::Api,
            "knowledge_base" => AiToolType::KnowledgeBase,
            _ => AiToolType::Api,
        }
    }
}

impl From<AiToolType> for String {
    fn from(tool_type: AiToolType) -> Self {
        match tool_type {
            AiToolType::Api => "api".to_string(),
            AiToolType::KnowledgeBase => "knowledge_base".to_string(),
        }
    }
}

impl From<String> for HttpMethod {
    fn from(method: String) -> Self {
        match method.to_uppercase().as_str() {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            "PUT" => HttpMethod::PUT,
            "DELETE" => HttpMethod::DELETE,
            "PATCH" => HttpMethod::PATCH,
            _ => HttpMethod::GET,
        }
    }
}

impl From<HttpMethod> for String {
    fn from(method: HttpMethod) -> Self {
        match method {
            HttpMethod::GET => "GET".to_string(),
            HttpMethod::POST => "POST".to_string(),
            HttpMethod::PUT => "PUT".to_string(),
            HttpMethod::DELETE => "DELETE".to_string(),
            HttpMethod::PATCH => "PATCH".to_string(),
        }
    }
}

impl Default for KnowledgeBaseSearchSettings {
    fn default() -> Self {
        Self {
            max_results: Some(10),
            similarity_threshold: Some(0.7),
            include_metadata: true,
        }
    }
}

impl Default for AiToolConfiguration {
    fn default() -> Self {
        Self::Api(ApiToolConfiguration {
            endpoint: "".to_string(),
            method: HttpMethod::GET,
            headers: Vec::new(),
            query_parameters: Vec::new(),
            body_parameters: Vec::new(),
            authorization: None,
        })
    }
}

impl Default for ApiToolConfiguration {
    fn default() -> Self {
        Self {
            endpoint: "".to_string(),
            method: HttpMethod::GET,
            headers: Vec::new(),
            query_parameters: Vec::new(),
            body_parameters: Vec::new(),
            authorization: None,
        }
    }
}

impl Default for KnowledgeBaseToolConfiguration {
    fn default() -> Self {
        Self {
            knowledge_base_id: 0,
            search_settings: KnowledgeBaseSearchSettings::default(),
        }
    }
}
