use axum::extract::State;

use crate::{
    application::AppState,
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
    State(app_state): State<AppState>,
    Json(payload): Json<Project>,
) -> ApiResult<Project> {
    let project = Project::create(&app_state, payload).await?;
    Ok(project)
}
