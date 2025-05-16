use axum::Json;
use axum::extract::{Path, Query as QueryParams, State};

use crate::application::http::models::json::deployment_settings::DeploymentB2bSettingsUpdates;
use crate::application::query::OrganizationListQueryParams;
use crate::core::commands::{Command, UpdateDeploymentB2bSettingsCommand};
use crate::core::models::{Organization, Workspace};
use crate::core::queries::{DeploymentOrganizationListQuery, DeploymentWorkspaceListQuery};
use crate::{
    application::{AppState, response::ApiResult, response::PaginatedResponse},
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

pub async fn get_organization_list(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
    QueryParams(query_params): QueryParams<OrganizationListQueryParams>,
) -> ApiResult<PaginatedResponse<Organization>> {
    let limit = query_params.limit.unwrap_or(10);

    let organizations = DeploymentOrganizationListQuery::new(deployment_id)
        .limit(limit + 1)
        .offset(query_params.offset.unwrap_or(0))
        .sort_key(query_params.sort_key)
        .sort_order(query_params.sort_order)
        .execute(&app_state)
        .await?;

    let has_more = organizations.len() > limit as usize;
    let organizations = if has_more {
        organizations[..limit as usize].to_vec()
    } else {
        organizations
    };

    Ok(PaginatedResponse::from(organizations).into())
}

pub async fn get_workspace_list(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
    QueryParams(query_params): QueryParams<OrganizationListQueryParams>,
) -> ApiResult<PaginatedResponse<Workspace>> {
    let limit = query_params.limit.unwrap_or(10);

    let workspaces = DeploymentWorkspaceListQuery::new(deployment_id)
        .limit(limit + 1)
        .offset(query_params.offset.unwrap_or(0))
        .sort_key(query_params.sort_key)
        .sort_order(query_params.sort_order)
        .execute(&app_state)
        .await?;

    let has_more = workspaces.len() > limit as usize;
    let workspaces = if has_more {
        workspaces[..limit as usize].to_vec()
    } else {
        workspaces
    };

    Ok(PaginatedResponse::from(workspaces).into())
}
