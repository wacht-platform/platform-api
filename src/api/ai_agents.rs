use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
};
use serde::Deserialize;

use crate::{
    application::AppState,
    core::{
        commands::{Command, CreateAiAgentCommand, DeleteAiAgentCommand, UpdateAiAgentCommand},
        models::{AiAgentWithDetails, CreateAiAgentRequest, UpdateAiAgentRequest},
        queries::{GetAiAgentByIdQuery, GetAiAgentsQuery, Query},
    },
};

use crate::application::response::{ApiResult, PaginatedResponse};

#[derive(Deserialize)]
pub struct GetAiAgentsParams {
    pub deployment_id: i64,
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub search: Option<String>,
}

pub async fn get_ai_agents(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
) -> ApiResult<PaginatedResponse<AiAgentWithDetails>> {
    let agents = GetAiAgentsQuery::new(deployment_id, 0, 50, None)
        .execute(&app_state)
        .await?;

    Ok(PaginatedResponse {
        data: agents,
        has_more: false,
    }
    .into())
}

pub async fn get_ai_agent_by_id(
    State(app_state): State<AppState>,
    Path((deployment_id, agent_id)): Path<(i64, i64)>,
) -> ApiResult<AiAgentWithDetails> {
    let agent = GetAiAgentByIdQuery::new(deployment_id, agent_id)
        .execute(&app_state)
        .await?;

    Ok(agent.into())
}

pub async fn create_ai_agent(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
    Json(request): Json<CreateAiAgentRequest>,
) -> ApiResult<AiAgentWithDetails> {
    let agent = CreateAiAgentCommand::new(deployment_id, request)
        .execute(&app_state)
        .await?;

    Ok(agent.into())
}

pub async fn update_ai_agent(
    State(app_state): State<AppState>,
    Path((deployment_id, agent_id)): Path<(i64, i64)>,
    Json(request): Json<UpdateAiAgentRequest>,
) -> ApiResult<AiAgentWithDetails> {
    let agent = UpdateAiAgentCommand::new(deployment_id, agent_id, request)
        .execute(&app_state)
        .await?;

    Ok(agent.into())
}

pub async fn delete_ai_agent(
    State(app_state): State<AppState>,
    Path((deployment_id, agent_id)): Path<(i64, i64)>,
) -> ApiResult<()> {
    DeleteAiAgentCommand::new(deployment_id, agent_id)
        .execute(&app_state)
        .await?;

    Ok(().into())
}
