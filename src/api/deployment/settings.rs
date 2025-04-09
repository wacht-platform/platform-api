use crate::{
    application::{ApiResult, AppState, DeploymentAuthSettingsUpdates},
    core::{
        commands::{Command, UpdateDeploymentAuthSettingsCommand},
        models::DeploymentWithSettings,
        queries::{Query, deployment::GetDeploymentWithSettingsQuery},
    },
};
use axum::{
    Json,
    extract::{Path, State},
};

pub async fn get_deployment_with_settings(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
) -> ApiResult<DeploymentWithSettings> {
    GetDeploymentWithSettingsQuery::new(deployment_id)
        .execute(&app_state)
        .await
        .map(Into::into)
        .map_err(Into::into)
}

pub async fn update_deployment_authetication_settings(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
    Json(settings): Json<DeploymentAuthSettingsUpdates>,
) -> ApiResult<()> {
    let command = UpdateDeploymentAuthSettingsCommand {
        deployment_id,
        settings,
    };

    command.execute(&app_state).await?;

    Ok(().into())
}
