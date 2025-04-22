use axum::{
    Router,
    routing::{delete, get, patch, post, put},
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use super::AppState;
use crate::api;

fn health_routes() -> Router<AppState> {
    Router::new().route("/health", get(api::health::check))
}

fn project_routes() -> Router<AppState> {
    Router::new()
        .route("/projects", get(api::project::get_projects))
        .route("/project", post(api::project::create_project))
        .route("/project/{id}", delete(api::project::delete_project))
}

fn deployment_routes() -> Router<AppState> {
    let routes = Router::new()
        .route("/users", get(api::deployment::user::get_user_list))
        .route(
            "/",
            get(api::deployment::settings::get_deployment_with_settings),
        )
        .route(
            "/jwt-templates",
            get(api::deployment::settings::get_deployment_jwt_templates),
        )
        .route(
            "/jwt-templates",
            post(api::deployment::settings::create_deployment_jwt_template),
        )
        .route(
            "/jwt-templates/{id}",
            patch(api::deployment::settings::update_deployment_jwt_template),
        )
        .route(
            "/jwt-templates/{id}",
            delete(api::deployment::settings::delete_deployment_jwt_template),
        )
        .route(
            "/workspace-roles",
            get(api::deployment::b2b::get_deployment_workspace_roles),
        )
        .route(
            "/organization-roles",
            get(api::deployment::b2b::get_deployment_org_roles),
        )
        .route(
            "/organizations",
            get(api::deployment::organization::get_organization_list),
        )
        .route(
            "/settings/auth-settings",
            patch(api::deployment::settings::update_deployment_authetication_settings),
        )
        .route(
            "/settings/display-settings",
            patch(api::deployment::settings::update_deployment_display_settings),
        )
        .route(
            "/restrictions",
            patch(api::deployment::settings::update_deployment_restrictions),
        )
        .route(
            "/social-connections",
            get(api::deployment::connection::get_deployment_social_connections),
        )
        .route(
            "/social-connections",
            put(api::deployment::connection::upsert_deployment_social_connection),
        )
        .route(
            "/settings/b2b-settings",
            patch(api::deployment::b2b::update_deployment_b2b_settings),
        )
        .route(
            "/email-templates/{template_name}",
            get(api::deployment::settings::get_email_template),
        )
        .route(
            "/email-templates/{template_name}",
            patch(api::deployment::settings::update_email_template),
        )
        .route(
            "/upload/{image_type}",
            post(api::deployment::upload::upload_image),
        );

    Router::new().nest("/deployments/{deployment_id}", routes)
}

fn configure_cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}

pub fn create_router(state: AppState) -> Router {
    let cors = configure_cors();

    Router::new()
        .merge(health_routes())
        .merge(project_routes())
        .merge(deployment_routes())
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}
