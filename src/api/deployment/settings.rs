use crate::{
    application::{
        ApiResult, AppState, DeploymentAuthSettingsUpdates, DeploymentRestrictionsUpdates,
    },
    core::{
        commands::{
            Command, UpdateDeploymentAuthSettingsCommand, UpdateDeploymentRestrictionsCommand,
        },
        models::{DeploymentJwtTemplate, DeploymentWithSettings},
        queries::{
            Query,
            deployment::{GetDeploymentJwtTemplatesQuery, GetDeploymentWithSettingsQuery},
        },
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
    UpdateDeploymentAuthSettingsCommand::new(deployment_id, settings)
        .execute(&app_state)
        .await
        .map(Into::into)
        .map_err(Into::into)
}

pub async fn update_deployment_restrictions(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
    Json(updates): Json<DeploymentRestrictionsUpdates>,
) -> ApiResult<()> {
    UpdateDeploymentRestrictionsCommand::new(deployment_id, updates)
        .execute(&app_state)
        .await
        .map(Into::into)
        .map_err(Into::into)
}

pub async fn get_deployment_jwt_templates(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
) -> ApiResult<Vec<DeploymentJwtTemplate>> {
    GetDeploymentJwtTemplatesQuery::new(deployment_id)
        .execute(&app_state)
        .await
        .map(Into::into)
        .map_err(Into::into)
}
