use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::{
    application::{response::ApiResult, AppState},
    core::models::AiWorkflowWithDetails,
};

#[derive(Deserialize)]
pub struct GetWorkflowsQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub search: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub description: Option<String>,
    pub configuration: Option<serde_json::Value>,
    pub workflow_definition: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct UpdateWorkflowRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub configuration: Option<serde_json::Value>,
    pub workflow_definition: Option<serde_json::Value>,
}

pub async fn get_ai_workflows(
    State(_app_state): State<AppState>,
    Path(_deployment_id): Path<i64>,
    Query(_query): Query<GetWorkflowsQuery>,
) -> ApiResult<Vec<AiWorkflowWithDetails>> {
    // TODO: Implement
    Ok(vec![].into())
}

pub async fn create_ai_workflow(
    State(_app_state): State<AppState>,
    Path(_deployment_id): Path<i64>,
    Json(_request): Json<CreateWorkflowRequest>,
) -> ApiResult<()> {
    // TODO: Implement
    Err((StatusCode::NOT_IMPLEMENTED, "Not implemented yet".to_string()).into())
}

pub async fn get_ai_workflow_by_id(
    State(_app_state): State<AppState>,
    Path((_deployment_id, _workflow_id)): Path<(i64, i64)>,
) -> ApiResult<()> {
    // TODO: Implement
    Err((StatusCode::NOT_IMPLEMENTED, "Not implemented yet".to_string()).into())
}

pub async fn update_ai_workflow(
    State(_app_state): State<AppState>,
    Path((_deployment_id, _workflow_id)): Path<(i64, i64)>,
    Json(_request): Json<UpdateWorkflowRequest>,
) -> ApiResult<()> {
    // TODO: Implement
    Err((StatusCode::NOT_IMPLEMENTED, "Not implemented yet".to_string()).into())
}

pub async fn delete_ai_workflow(
    State(_app_state): State<AppState>,
    Path((_deployment_id, _workflow_id)): Path<(i64, i64)>,
) -> ApiResult<()> {
    // TODO: Implement
    Err((StatusCode::NOT_IMPLEMENTED, "Not implemented yet".to_string()).into())
}
