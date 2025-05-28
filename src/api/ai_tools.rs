use axum::extract::{Json, Path, State};

use crate::{
    application::AppState,
    core::{
        commands::{Command, CreateAiToolCommand, DeleteAiToolCommand, UpdateAiToolCommand},
        models::{AiTool, CreateAiToolRequest, UpdateAiToolRequest},
        queries::{GetAiToolByIdQuery, GetAiToolsQuery, Query},
    },
};

use crate::application::response::{ApiResult, PaginatedResponse};

pub async fn get_ai_tools(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
) -> ApiResult<PaginatedResponse<AiTool>> {
    let tools = GetAiToolsQuery::new(deployment_id, 0, 50, None)
        .execute(&app_state)
        .await?;

    Ok(PaginatedResponse {
        data: tools,
        has_more: false,
    }
    .into())
}

pub async fn get_ai_tool_by_id(
    State(app_state): State<AppState>,
    Path((deployment_id, tool_id)): Path<(i64, i64)>,
) -> ApiResult<AiTool> {
    let tool = GetAiToolByIdQuery::new(deployment_id, tool_id)
        .execute(&app_state)
        .await?;

    Ok(tool.into())
}

pub async fn create_ai_tool(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
    Json(request): Json<CreateAiToolRequest>,
) -> ApiResult<AiTool> {
    let tool = CreateAiToolCommand::new(deployment_id, request)
        .execute(&app_state)
        .await?;

    Ok(tool.into())
}

pub async fn update_ai_tool(
    State(app_state): State<AppState>,
    Path((deployment_id, tool_id)): Path<(i64, i64)>,
    Json(request): Json<UpdateAiToolRequest>,
) -> ApiResult<AiTool> {
    let tool = UpdateAiToolCommand::new(deployment_id, tool_id, request)
        .execute(&app_state)
        .await?;

    Ok(tool.into())
}

pub async fn delete_ai_tool(
    State(app_state): State<AppState>,
    Path((deployment_id, tool_id)): Path<(i64, i64)>,
) -> ApiResult<()> {
    DeleteAiToolCommand::new(deployment_id, tool_id)
        .execute(&app_state)
        .await?;

    Ok(().into())
}
