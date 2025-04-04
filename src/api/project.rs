use axum::{
    extract::{Multipart, State},
    http::StatusCode,
};

use crate::{
    application::{ApiError, ApiErrorResponse, AppState},
    core::{
        models::ProjectWithDeployments,
        queries::{GetProjectsWithDeploymentQuery, Query},
    },
};

use crate::application::{ApiResult, PaginatedResponse};

pub async fn get_projects(
    State(app_state): State<AppState>,
) -> ApiResult<PaginatedResponse<ProjectWithDeployments>> {
    let projects = GetProjectsWithDeploymentQuery::new(0)
        .execute(&app_state)
        .await
        .unwrap();

    Ok(PaginatedResponse {
        data: projects,
        has_more: false,
    }
    .into())
}

pub async fn create_project(
    // State(app_state): State<AppState>,
    mut multipart: Multipart,
) -> ApiResult<()> {
    // let project = Project::create(&app_state, payload).await?;
    let mut name = String::new();
    let mut logo_buffer: Vec<u8> = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        let field_name = field.name().unwrap_or_default().to_string();
        let content_type = field.content_type().unwrap_or_default().to_string();
        let value = field.bytes().await.unwrap().to_vec();

        if field_name == "name" {
            name = field_name;
        } else if field_name == "logo" && content_type == "image/png" {
            logo_buffer = value;
        }
    }

    println!("name: {}", name);
    println!("logo_buffer: {:?}", logo_buffer);

    Ok(().into())
}
