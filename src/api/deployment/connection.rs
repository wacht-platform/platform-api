use crate::{
    application::{
        ApiResult, ApiSuccess, AppState, DeploymentSocialConnectionUpsert, PaginatedResponse,
    },
    core::{
        commands::{Command, UpsertDeploymentSocialConnectionCommand},
        models::DeploymentSocialConnection,
        queries::{Query, deployment::GetDeploymentSocialConnectionsQuery},
    },
};
use axum::{
    Json,
    extract::{Path, State},
};

pub async fn get_deployment_social_connections(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
) -> ApiResult<PaginatedResponse<DeploymentSocialConnection>> {
    GetDeploymentSocialConnectionsQuery::new(deployment_id)
        .execute(&app_state)
        .await
        .map(Into::<PaginatedResponse<_>>::into)
        .map(ApiSuccess::from)
        .map_err(Into::into)
}

pub async fn upsert_deployment_social_connection(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
    Json(payload): Json<DeploymentSocialConnectionUpsert>,
) -> ApiResult<DeploymentSocialConnection> {
    UpsertDeploymentSocialConnectionCommand::new(deployment_id, payload)
        .execute(&app_state)
        .await
        .map(Into::<DeploymentSocialConnection>::into)
        .map(ApiSuccess::from)
        .map_err(Into::into)
}
