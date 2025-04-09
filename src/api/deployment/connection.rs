use crate::{
    application::{ApiResult, ApiSuccess, AppState, PaginatedResponse},
    core::{
        models::DeploymentSocialConnection,
        queries::{Query, deployment::GetDeploymentSocialConnectionsQuery},
    },
};
use axum::extract::{Path, State};

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
