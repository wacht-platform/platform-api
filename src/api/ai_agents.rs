use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::{
    application::{response::ApiResult, AppState},
    core::models::AiAgentWithDetails,
};

#[derive(Deserialize)]
pub struct GetAgentsQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub search: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub description: Option<String>,
    pub configuration: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct UpdateAgentRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub configuration: Option<serde_json::Value>,
}

pub async fn get_ai_agents(
    State(_app_state): State<AppState>,
    Path(_deployment_id): Path<i64>,
    Query(_query): Query<GetAgentsQuery>,
) -> ApiResult<Vec<AiAgentWithDetails>> {
    // TODO: Implement
    Ok(vec![].into())
}

pub async fn create_ai_agent(
    State(_app_state): State<AppState>,
    Path(_deployment_id): Path<i64>,
    Json(_request): Json<CreateAgentRequest>,
) -> ApiResult<()> {
    // TODO: Implement
    Err((StatusCode::NOT_IMPLEMENTED, "Not implemented yet".to_string()).into())
}

pub async fn get_ai_agent_by_id(
    State(_app_state): State<AppState>,
    Path((_deployment_id, _agent_id)): Path<(i64, i64)>,
) -> ApiResult<()> {
    // TODO: Implement
    Err((StatusCode::NOT_IMPLEMENTED, "Not implemented yet".to_string()).into())
}

pub async fn update_ai_agent(
    State(_app_state): State<AppState>,
    Path((_deployment_id, _agent_id)): Path<(i64, i64)>,
    Json(_request): Json<UpdateAgentRequest>,
) -> ApiResult<()> {
    // TODO: Implement
    Err((StatusCode::NOT_IMPLEMENTED, "Not implemented yet".to_string()).into())
}

pub async fn delete_ai_agent(
    State(_app_state): State<AppState>,
    Path((_deployment_id, _agent_id)): Path<(i64, i64)>,
) -> ApiResult<()> {
    // TODO: Implement
    Err((StatusCode::NOT_IMPLEMENTED, "Not implemented yet".to_string()).into())
}
