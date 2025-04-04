use axum::{
    Router,
    routing::{get, patch, post},
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
}

fn deployment_routes() -> Router<AppState> {
    let routes = Router::new()
        .route("/users", get(api::deployment::user::get_user_list))
        .route(
            "/organizations",
            get(api::deployment::organization::get_organization_list),
        )
        .route(
            "/settings",
            get(api::deployment::settings::get_deployment_with_settings),
        )
        .route(
            "/settings/auth-settings",
            patch(api::deployment::settings::update_deployment_authetication_settings),
        )
        .route(
            "/social-connections",
            get(api::deployment::connection::get_deployment_social_connections),
        );

    Router::new().nest("/deployment/{deployment_id}", routes)
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
