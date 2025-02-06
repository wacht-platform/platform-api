use crate::{
    application::{ApiResult, AppState, OrganizationListQueryParams, PaginatedResponse},
    core::{
        models::Organization,
        queries::{DeploymentOrganizationListQuery, Query},
    },
};
use axum::extract::{Path, Query as QueryParams, State};

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
        .await
        .unwrap();

    let has_more = organizations.len() > limit as usize;
    let organizations = if has_more {
        organizations[..limit as usize].to_vec()
    } else {
        organizations
    };

    Ok(PaginatedResponse {
        data: organizations,
        has_more,
    }
    .into())
}
