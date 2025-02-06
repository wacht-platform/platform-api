use crate::{
    application::{ApiError, ApiErrorResponse, ApiResult, AppState},
    core::{
        models::DeploymentWithSettings,
        queries::{deployment::GetDeploymentWithSettingsQuery, Query},
    },
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
};

pub async fn get_deployment_with_settings(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
) -> ApiResult<DeploymentWithSettings> {
    GetDeploymentWithSettingsQuery::new(deployment_id)
        .execute(&app_state)
        .await
        .map_err(|e| ApiErrorResponse {
            staus_code: StatusCode::INTERNAL_SERVER_ERROR,
            errors: vec![ApiError {
                message: e.to_string(),
                code: 500,
            }],
        })
        .map(Into::into)
}

pub async fn update_deployment_settings(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
    body: DeploymentWithSettings,
) -> ApiResult<()> {
    // TODO: Implement update logic
    Ok(().into())
}
