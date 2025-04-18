use axum::Json;
use axum::extract::{Path, State};

use crate::application::http::models::json::deployment_settings::DeploymentB2bSettingsUpdates;
use crate::core::commands::{Command, UpdateDeploymentB2bSettingsCommand};
use crate::{
    application::{ApiResult, AppState, PaginatedResponse},
    core::{
        models::{DeploymentOrganizationRole, DeploymentWorkspaceRole},
        queries::{GetDeploymentOrganizationRolesQuery, GetDeploymentWorkspaceRolesQuery, Query},
    },
};

pub async fn get_deployment_workspace_roles(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
) -> ApiResult<PaginatedResponse<DeploymentWorkspaceRole>> {
    GetDeploymentWorkspaceRolesQuery::new(deployment_id)
        .execute(&app_state)
        .await
        .map(PaginatedResponse::from)
        .map(Into::into)
        .map_err(Into::into)
}

pub async fn get_deployment_org_roles(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
) -> ApiResult<PaginatedResponse<DeploymentOrganizationRole>> {
    GetDeploymentOrganizationRolesQuery::new(deployment_id)
        .execute(&app_state)
        .await
        .map(PaginatedResponse::from)
        .map(Into::into)
        .map_err(Into::into)
}

pub async fn update_deployment_b2b_settings(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
    Json(settings): Json<DeploymentB2bSettingsUpdates>,
) -> ApiResult<()> {
    UpdateDeploymentB2bSettingsCommand::new(deployment_id, settings)
        .execute(&app_state)
        .await
        .map(Into::into)
        .map_err(Into::into)
}
