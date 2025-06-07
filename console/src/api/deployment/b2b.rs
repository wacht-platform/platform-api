use axum::Json;
use axum::extract::{Path, Query as QueryParams, State};

use crate::core::commands::{
    AddOrganizationMemberCommand, Command, CreateOrganizationCommand,
    CreateOrganizationRoleCommand, CreateWorkspaceCommand, DeleteOrganizationCommand,
    DeleteOrganizationRoleCommand, RemoveOrganizationMemberCommand,
    UpdateDeploymentB2bSettingsCommand, UpdateOrganizationCommand, UpdateOrganizationMemberCommand,
    UpdateOrganizationRoleCommand,
};
use crate::core::dto::{
    json::{
        b2b::{
            AddOrganizationMemberRequest, CreateOrganizationRequest, CreateOrganizationRoleRequest,
            CreateWorkspaceRequest, UpdateOrganizationMemberRequest, UpdateOrganizationRequest,
            UpdateOrganizationRoleRequest,
        },
        deployment_settings::DeploymentB2bSettingsUpdates,
    },
    query::OrganizationListQueryParams,
};
use crate::core::models::{
    Organization, OrganizationDetails, OrganizationMemberDetails, OrganizationRole, Workspace,
    WorkspaceDetails, WorkspaceWithOrganizationName,
};
use crate::core::queries::{
    DeploymentOrganizationListQuery, DeploymentWorkspaceListQuery, GetOrganizationDetailsQuery,
    GetWorkspaceDetailsQuery,
};
use crate::{
    application::{HttpState, response::ApiResult, response::PaginatedResponse},
    core::{
        models::{DeploymentOrganizationRole, DeploymentWorkspaceRole},
        queries::{GetDeploymentOrganizationRolesQuery, GetDeploymentWorkspaceRolesQuery, Query},
    },
};

pub async fn get_deployment_workspace_roles(
    State(app_state): State<HttpState>,
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
    State(app_state): State<HttpState>,
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
    State(app_state): State<HttpState>,
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
    State(app_state): State<HttpState>,
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
    State(app_state): State<HttpState>,
    Path(deployment_id): Path<i64>,
    QueryParams(query_params): QueryParams<OrganizationListQueryParams>,
) -> ApiResult<PaginatedResponse<WorkspaceWithOrganizationName>> {
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

pub async fn get_organization_details(
    State(app_state): State<HttpState>,
    Path((deployment_id, organization_id)): Path<(i64, i64)>,
) -> ApiResult<OrganizationDetails> {
    GetOrganizationDetailsQuery::new(deployment_id, organization_id)
        .execute(&app_state)
        .await
        .map(Into::into)
        .map_err(Into::into)
}

pub async fn get_workspace_details(
    State(app_state): State<HttpState>,
    Path((deployment_id, workspace_id)): Path<(i64, i64)>,
) -> ApiResult<WorkspaceDetails> {
    GetWorkspaceDetailsQuery::new(deployment_id, workspace_id)
        .execute(&app_state)
        .await
        .map(Into::into)
        .map_err(Into::into)
}

pub async fn create_organization(
    State(app_state): State<HttpState>,
    Path(deployment_id): Path<i64>,
    Json(request): Json<CreateOrganizationRequest>,
) -> ApiResult<Organization> {
    CreateOrganizationCommand::new(
        deployment_id,
        request.name,
        request.description,
        request.image_url,
        request.public_metadata,
        request.private_metadata,
    )
    .execute(&app_state)
    .await
    .map(Into::into)
    .map_err(Into::into)
}

pub async fn create_workspace_for_organization(
    State(app_state): State<HttpState>,
    Path((deployment_id, organization_id)): Path<(i64, i64)>,
    Json(request): Json<CreateWorkspaceRequest>,
) -> ApiResult<Workspace> {
    CreateWorkspaceCommand::new(
        deployment_id,
        organization_id,
        request.name,
        request.description,
        request.image_url,
        request.public_metadata,
        request.private_metadata,
    )
    .execute(&app_state)
    .await
    .map(Into::into)
    .map_err(Into::into)
}

pub async fn update_organization(
    State(app_state): State<HttpState>,
    Path((deployment_id, organization_id)): Path<(i64, i64)>,
    Json(request): Json<UpdateOrganizationRequest>,
) -> ApiResult<Organization> {
    UpdateOrganizationCommand::new(
        deployment_id,
        organization_id,
        request.name,
        request.description,
        request.image_url,
        request.public_metadata,
        request.private_metadata,
    )
    .execute(&app_state)
    .await
    .map(Into::into)
    .map_err(Into::into)
}

pub async fn delete_organization(
    State(app_state): State<HttpState>,
    Path((deployment_id, organization_id)): Path<(i64, i64)>,
) -> ApiResult<()> {
    DeleteOrganizationCommand::new(deployment_id, organization_id)
        .execute(&app_state)
        .await
        .map(Into::into)
        .map_err(Into::into)
}

// Organization Member Management

pub async fn add_organization_member(
    State(app_state): State<HttpState>,
    Path((deployment_id, organization_id)): Path<(i64, i64)>,
    Json(request): Json<AddOrganizationMemberRequest>,
) -> ApiResult<OrganizationMemberDetails> {
    AddOrganizationMemberCommand::new(
        deployment_id,
        organization_id,
        request.user_id,
        request.role_ids,
    )
    .execute(&app_state)
    .await
    .map(Into::into)
    .map_err(Into::into)
}

pub async fn update_organization_member(
    State(app_state): State<HttpState>,
    Path((deployment_id, organization_id, membership_id)): Path<(i64, i64, i64)>,
    Json(request): Json<UpdateOrganizationMemberRequest>,
) -> ApiResult<()> {
    UpdateOrganizationMemberCommand::new(
        deployment_id,
        organization_id,
        membership_id,
        request.role_ids,
    )
    .execute(&app_state)
    .await
    .map(Into::into)
    .map_err(Into::into)
}

pub async fn remove_organization_member(
    State(app_state): State<HttpState>,
    Path((deployment_id, organization_id, membership_id)): Path<(i64, i64, i64)>,
) -> ApiResult<()> {
    RemoveOrganizationMemberCommand::new(deployment_id, organization_id, membership_id)
        .execute(&app_state)
        .await
        .map(Into::into)
        .map_err(Into::into)
}

// Organization Role Management

pub async fn create_organization_role(
    State(app_state): State<HttpState>,
    Path((deployment_id, organization_id)): Path<(i64, i64)>,
    Json(request): Json<CreateOrganizationRoleRequest>,
) -> ApiResult<OrganizationRole> {
    CreateOrganizationRoleCommand::new(
        deployment_id,
        organization_id,
        request.name,
        request.permissions,
    )
    .execute(&app_state)
    .await
    .map(Into::into)
    .map_err(Into::into)
}

pub async fn update_organization_role(
    State(app_state): State<HttpState>,
    Path((deployment_id, organization_id, role_id)): Path<(i64, i64, i64)>,
    Json(request): Json<UpdateOrganizationRoleRequest>,
) -> ApiResult<OrganizationRole> {
    UpdateOrganizationRoleCommand::new(
        deployment_id,
        organization_id,
        role_id,
        request.name,
        request.permissions,
    )
    .execute(&app_state)
    .await
    .map(Into::into)
    .map_err(Into::into)
}

pub async fn delete_organization_role(
    State(app_state): State<HttpState>,
    Path((deployment_id, organization_id, role_id)): Path<(i64, i64, i64)>,
) -> ApiResult<()> {
    DeleteOrganizationRoleCommand::new(deployment_id, organization_id, role_id)
        .execute(&app_state)
        .await
        .map(Into::into)
        .map_err(Into::into)
}
