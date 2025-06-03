use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::{
    application::{response::ApiResult, AppState},
    core::models::AiToolWithDetails,
};

#[derive(Deserialize)]
pub struct GetToolsQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub search: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateToolRequest {
    pub name: String,
    pub description: Option<String>,
    pub tool_type: String,
    pub configuration: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct UpdateToolRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tool_type: Option<String>,
    pub status: Option<String>,
    pub configuration: Option<serde_json::Value>,
}

pub async fn get_ai_tools(
    State(_app_state): State<AppState>,
    Path(_deployment_id): Path<i64>,
    Query(_query): Query<GetToolsQuery>,
) -> ApiResult<Vec<AiToolWithDetails>> {
    // TODO: Implement
    Ok(vec![].into())
}

pub async fn create_ai_tool(
    State(_app_state): State<AppState>,
    Path(_deployment_id): Path<i64>,
    Json(_request): Json<CreateToolRequest>,
) -> ApiResult<()> {
    // TODO: Implement
    Err((StatusCode::NOT_IMPLEMENTED, "Not implemented yet".to_string()).into())
}

pub async fn get_ai_tool_by_id(
    State(_app_state): State<AppState>,
    Path((_deployment_id, _tool_id)): Path<(i64, i64)>,
) -> ApiResult<()> {
    // TODO: Implement
    Err((StatusCode::NOT_IMPLEMENTED, "Not implemented yet".to_string()).into())
}

pub async fn update_ai_tool(
    State(_app_state): State<AppState>,
    Path((_deployment_id, _tool_id)): Path<(i64, i64)>,
    Json(_request): Json<UpdateToolRequest>,
) -> ApiResult<()> {
    // TODO: Implement
    Err((StatusCode::NOT_IMPLEMENTED, "Not implemented yet".to_string()).into())
}

pub async fn delete_ai_tool(
    State(_app_state): State<AppState>,
    Path((_deployment_id, _tool_id)): Path<(i64, i64)>,
) -> ApiResult<()> {
    // TODO: Implement
    Err((StatusCode::NOT_IMPLEMENTED, "Not implemented yet".to_string()).into())
}
