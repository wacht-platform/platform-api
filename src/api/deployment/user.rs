use crate::{
    application::{
        AppState,
        query::UserListQueryParams,
        response::{ApiResult, PaginatedResponse},
    },
    core::{
        models::UserWithIdentifiers,
        queries::{DeploymentUserListQuery, Query},
    },
};
use axum::extract::{Path, Query as QueryParams, State};

pub async fn get_user_list(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
    QueryParams(query_params): QueryParams<UserListQueryParams>,
) -> ApiResult<PaginatedResponse<UserWithIdentifiers>> {
    let limit = query_params.limit.unwrap_or(10) as i32;

    let users = DeploymentUserListQuery::new(deployment_id)
        .limit(limit + 1)
        .offset(query_params.offset.unwrap_or(0))
        .sort_key(query_params.sort_key.as_ref().map(ToString::to_string))
        .sort_order(query_params.sort_order.as_ref().map(ToString::to_string))
        .disabled(query_params.disabled.unwrap_or_default())
        .invited(query_params.invited.unwrap_or_default())
        .execute(&app_state)
        .await?;

    let has_more = users.len() > limit as usize;
    let users = if has_more {
        users[..limit as usize].to_vec()
    } else {
        users
    };

    Ok(PaginatedResponse::from(users).into())
}
