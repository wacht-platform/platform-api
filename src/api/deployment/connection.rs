use crate::{
    application::{ApiError, ApiErrorResponse, ApiResult, AppState, PaginatedResponse},
    core::{
        models::DeploymentSocialConnection,
        queries::{deployment::GetDeploymentSocialConnectionsQuery, Query},
    },
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
};

pub async fn get_deployment_social_connections(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
) -> ApiResult<PaginatedResponse<DeploymentSocialConnection>> {
    GetDeploymentSocialConnectionsQuery::new(deployment_id)
        .execute(&app_state)
        .await
        .map_err(|e| ApiErrorResponse {
            staus_code: StatusCode::INTERNAL_SERVER_ERROR,
            errors: vec![ApiError {
                message: e.to_string(),
                code: 500,
            }],
        })
        .map(Into::<PaginatedResponse<_>>::into)
        .map(Into::into)
}
