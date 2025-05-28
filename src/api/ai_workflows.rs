use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
};

use crate::{
    application::AppState,
    core::{
        commands::{
            Command, CreateAiWorkflowCommand, DeleteAiWorkflowCommand, UpdateAiWorkflowCommand,
        },
        models::{AiWorkflowWithDetails, CreateAiWorkflowRequest, UpdateAiWorkflowRequest},
        queries::{GetAiWorkflowByIdQuery, GetAiWorkflowsQuery, Query},
    },
};

use crate::application::response::{ApiResult, PaginatedResponse};

pub async fn get_ai_workflows(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
) -> ApiResult<PaginatedResponse<AiWorkflowWithDetails>> {
    let workflows = GetAiWorkflowsQuery::new(deployment_id, 0, 50, None)
        .execute(&app_state)
        .await?;

    Ok(PaginatedResponse {
        data: workflows,
        has_more: false,
    }
    .into())
}

pub async fn get_ai_workflow_by_id(
    State(app_state): State<AppState>,
    Path((deployment_id, workflow_id)): Path<(i64, i64)>,
) -> ApiResult<AiWorkflowWithDetails> {
    let workflow = GetAiWorkflowByIdQuery::new(deployment_id, workflow_id)
        .execute(&app_state)
        .await?;

    Ok(workflow.into())
}

pub async fn create_ai_workflow(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
    Json(request): Json<CreateAiWorkflowRequest>,
) -> ApiResult<AiWorkflowWithDetails> {
    let workflow = CreateAiWorkflowCommand::new(deployment_id, request)
        .execute(&app_state)
        .await?;

    Ok(workflow.into())
}

pub async fn update_ai_workflow(
    State(app_state): State<AppState>,
    Path((deployment_id, workflow_id)): Path<(i64, i64)>,
    Json(request): Json<UpdateAiWorkflowRequest>,
) -> ApiResult<AiWorkflowWithDetails> {
    let workflow = UpdateAiWorkflowCommand::new(deployment_id, workflow_id, request)
        .execute(&app_state)
        .await?;

    Ok(workflow.into())
}

pub async fn delete_ai_workflow(
    State(app_state): State<AppState>,
    Path((deployment_id, workflow_id)): Path<(i64, i64)>,
) -> ApiResult<()> {
    DeleteAiWorkflowCommand::new(deployment_id, workflow_id)
        .execute(&app_state)
        .await?;

    Ok(().into())
}
