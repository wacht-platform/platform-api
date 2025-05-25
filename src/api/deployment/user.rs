use crate::{
    application::{
        AppState,
        http::models::json::{
            AddEmailRequest, AddPhoneRequest, AddToWaitlistRequest, CreateUserRequest,
            InviteUserRequest, UpdateEmailRequest, UpdatePhoneRequest, UpdateUserRequest,
        },
        query::{ActiveUserListQueryParams, InvitationsWaitlistQueryParams},
        response::{ApiResult, PaginatedResponse},
    },
    core::{
        commands::{
            AddToWaitlistCommand, AddUserEmailCommand, AddUserPhoneCommand,
            ApproveWaitlistUserCommand, Command, CreateUserCommand, DeleteUserEmailCommand,
            DeleteUserPhoneCommand, DeleteUserSocialConnectionCommand, InviteUserCommand,
            UpdateUserCommand, UpdateUserEmailCommand, UpdateUserPhoneCommand,
        },
        models::{
            DeploymentInvitation, DeploymentWaitlistUser, UserDetails, UserEmailAddress,
            UserPhoneNumber, UserWithIdentifiers,
        },
        queries::{
            DeploymentActiveUserListQuery, DeploymentInvitationQuery, DeploymentWaitlistQuery,
            GetUserDetailsQuery, Query,
        },
    },
};
use axum::{
    Json,
    extract::{Path, Query as QueryParams, State},
};

pub async fn get_active_user_list(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
    QueryParams(query_params): QueryParams<ActiveUserListQueryParams>,
) -> ApiResult<PaginatedResponse<UserWithIdentifiers>> {
    let limit = query_params.limit.unwrap_or(10) as i32;

    let users = DeploymentActiveUserListQuery::new(deployment_id)
        .limit(limit + 1)
        .offset(query_params.offset.unwrap_or(0))
        .sort_key(query_params.sort_key.as_ref().map(ToString::to_string))
        .sort_order(query_params.sort_order.as_ref().map(ToString::to_string))
        .execute(&app_state)
        .await
        .unwrap();

    let has_more = users.len() > limit as usize;
    let users = if has_more {
        users[..limit as usize].to_vec()
    } else {
        users
    };

    Ok(PaginatedResponse::from(users).into())
}

pub async fn get_invited_user_list(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
    QueryParams(query_params): QueryParams<InvitationsWaitlistQueryParams>,
) -> ApiResult<PaginatedResponse<DeploymentInvitation>> {
    let limit = query_params.limit.unwrap_or(10) as i32;

    let users = DeploymentInvitationQuery::new(deployment_id)
        .limit(limit + 1)
        .offset(query_params.offset.unwrap_or(0))
        .sort_key(query_params.sort_key.as_ref().map(ToString::to_string))
        .sort_order(query_params.sort_order.as_ref().map(ToString::to_string))
        .execute(&app_state)
        .await
        .unwrap();

    let has_more = users.len() > limit as usize;
    let users = if has_more {
        users[..limit as usize].to_vec()
    } else {
        users
    };

    Ok(PaginatedResponse::from(users).into())
}

pub async fn get_user_waitlist(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
    QueryParams(query_params): QueryParams<InvitationsWaitlistQueryParams>,
) -> ApiResult<PaginatedResponse<DeploymentWaitlistUser>> {
    let limit = query_params.limit.unwrap_or(10) as i32;

    let users = DeploymentWaitlistQuery::new(deployment_id)
        .limit(limit + 1)
        .offset(query_params.offset.unwrap_or(0))
        .sort_key(query_params.sort_key.as_ref().map(ToString::to_string))
        .sort_order(query_params.sort_order.as_ref().map(ToString::to_string))
        .execute(&app_state)
        .await
        .unwrap();

    let has_more = users.len() > limit as usize;
    let users = if has_more {
        users[..limit as usize].to_vec()
    } else {
        users
    };

    Ok(PaginatedResponse::from(users).into())
}

pub async fn create_user(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
    Json(request): Json<CreateUserRequest>,
) -> ApiResult<UserWithIdentifiers> {
    let user = CreateUserCommand::new(deployment_id, request)
        .execute(&app_state)
        .await?;

    Ok(user.into())
}

pub async fn get_user_details(
    State(app_state): State<AppState>,
    Path((deployment_id, user_id)): Path<(i64, i64)>,
) -> ApiResult<UserDetails> {
    let user_details = GetUserDetailsQuery::new(deployment_id, user_id)
        .execute(&app_state)
        .await?;

    Ok(user_details.into())
}

pub async fn invite_user(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
    Json(request): Json<InviteUserRequest>,
) -> ApiResult<DeploymentInvitation> {
    let invitation = InviteUserCommand::new(deployment_id, request)
        .execute(&app_state)
        .await?;

    Ok(invitation.into())
}

pub async fn add_to_waitlist(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
    Json(request): Json<AddToWaitlistRequest>,
) -> ApiResult<DeploymentWaitlistUser> {
    let waitlist_user = AddToWaitlistCommand::new(deployment_id, request)
        .execute(&app_state)
        .await?;

    Ok(waitlist_user.into())
}

pub async fn approve_waitlist_user(
    State(app_state): State<AppState>,
    Path((deployment_id, waitlist_user_id)): Path<(i64, i64)>,
) -> ApiResult<UserWithIdentifiers> {
    let user = ApproveWaitlistUserCommand::new(deployment_id, waitlist_user_id)
        .execute(&app_state)
        .await?;

    Ok(user.into())
}

pub async fn update_user(
    State(app_state): State<AppState>,
    Path((deployment_id, user_id)): Path<(i64, i64)>,
    Json(request): Json<UpdateUserRequest>,
) -> ApiResult<UserDetails> {
    let user_details = UpdateUserCommand::new(deployment_id, user_id, request)
        .execute(&app_state)
        .await?;

    Ok(user_details.into())
}

pub async fn add_user_email(
    State(app_state): State<AppState>,
    Path((deployment_id, user_id)): Path<(i64, i64)>,
    Json(request): Json<AddEmailRequest>,
) -> ApiResult<UserEmailAddress> {
    let email = AddUserEmailCommand::new(deployment_id, user_id, request)
        .execute(&app_state)
        .await?;

    Ok(email.into())
}

pub async fn update_user_email(
    State(app_state): State<AppState>,
    Path((deployment_id, user_id, email_id)): Path<(i64, i64, i64)>,
    Json(request): Json<UpdateEmailRequest>,
) -> ApiResult<UserEmailAddress> {
    let email = UpdateUserEmailCommand::new(deployment_id, user_id, email_id, request)
        .execute(&app_state)
        .await?;

    Ok(email.into())
}

pub async fn delete_user_email(
    State(app_state): State<AppState>,
    Path((_, user_id, email_id)): Path<(i64, i64, i64)>,
) -> ApiResult<()> {
    DeleteUserEmailCommand::new(user_id, email_id)
        .execute(&app_state)
        .await?;

    Ok(().into())
}

pub async fn add_user_phone(
    State(app_state): State<AppState>,
    Path((deployment_id, user_id)): Path<(i64, i64)>,
    Json(request): Json<AddPhoneRequest>,
) -> ApiResult<UserPhoneNumber> {
    let phone = AddUserPhoneCommand::new(deployment_id, user_id, request)
        .execute(&app_state)
        .await
        .unwrap();

    Ok(phone.into())
}

pub async fn update_user_phone(
    State(app_state): State<AppState>,
    Path((_, user_id, phone_id)): Path<(i64, i64, i64)>,
    Json(request): Json<UpdatePhoneRequest>,
) -> ApiResult<UserPhoneNumber> {
    let phone = UpdateUserPhoneCommand::new(user_id, phone_id, request)
        .execute(&app_state)
        .await?;

    Ok(phone.into())
}

pub async fn delete_user_phone(
    State(app_state): State<AppState>,
    Path((_, user_id, phone_id)): Path<(i64, i64, i64)>,
) -> ApiResult<()> {
    DeleteUserPhoneCommand::new(user_id, phone_id)
        .execute(&app_state)
        .await?;

    Ok(().into())
}

pub async fn delete_user_social_connection(
    State(app_state): State<AppState>,
    Path((_, user_id, connection_id)): Path<(i64, i64, i64)>,
) -> ApiResult<()> {
    DeleteUserSocialConnectionCommand::new(user_id, connection_id)
        .execute(&app_state)
        .await?;

    Ok(().into())
}
