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
        .route("/users", get(api::deployment::user::get_active_user_list))
        .route("/users", post(api::deployment::user::create_user))
        .route(
            "/users/{user_id}/details",
            get(api::deployment::user::get_user_details),
        )
        .route(
            "/users/{user_id}",
            patch(api::deployment::user::update_user),
        )
        .route(
            "/users/{user_id}/emails",
            post(api::deployment::user::add_user_email),
        )
        .route(
            "/users/{user_id}/emails/{email_id}",
            patch(api::deployment::user::update_user_email),
        )
        .route(
            "/users/{user_id}/emails/{email_id}",
            delete(api::deployment::user::delete_user_email),
        )
        .route(
            "/users/{user_id}/phones",
            post(api::deployment::user::add_user_phone),
        )
        .route(
            "/users/{user_id}/phones/{phone_id}",
            patch(api::deployment::user::update_user_phone),
        )
        .route(
            "/users/{user_id}/phones/{phone_id}",
            delete(api::deployment::user::delete_user_phone),
        )
        .route(
            "/users/{user_id}/social-connections/{connection_id}",
            delete(api::deployment::user::delete_user_social_connection),
        )
        .route(
            "/invited-users",
            get(api::deployment::user::get_invited_user_list),
        )
        .route("/invited-users", post(api::deployment::user::invite_user))
        .route(
            "/user-waitlist",
            get(api::deployment::user::get_user_waitlist),
        )
        .route(
            "/user-waitlist",
            post(api::deployment::user::add_to_waitlist),
        )
        .route(
            "/user-waitlist/{waitlist_user_id}/approve",
            post(api::deployment::user::approve_waitlist_user),
        )
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
        .route("/workspaces", get(api::deployment::b2b::get_workspace_list))
        .route(
            "/workspaces/{workspace_id}",
            get(api::deployment::b2b::get_workspace_details),
        )
        .route(
            "/workspace-roles",
            get(api::deployment::b2b::get_deployment_workspace_roles),
        )
        .route(
            "/organizations",
            get(api::deployment::b2b::get_organization_list)
                .post(api::deployment::b2b::create_organization),
        )
        .route(
            "/organizations/{organization_id}",
            get(api::deployment::b2b::get_organization_details)
                .patch(api::deployment::b2b::update_organization)
                .delete(api::deployment::b2b::delete_organization),
        )
        .route(
            "/organizations/{organization_id}/workspaces",
            post(api::deployment::b2b::create_workspace_for_organization),
        )
        .route(
            "/organizations/{organization_id}/members",
            post(api::deployment::b2b::add_organization_member),
        )
        .route(
            "/organizations/{organization_id}/members/{membership_id}",
            patch(api::deployment::b2b::update_organization_member)
                .delete(api::deployment::b2b::remove_organization_member),
        )
        .route(
            "/organizations/{organization_id}/roles",
            post(api::deployment::b2b::create_organization_role),
        )
        .route(
            "/organizations/{organization_id}/roles/{role_id}",
            patch(api::deployment::b2b::update_organization_role)
                .delete(api::deployment::b2b::delete_organization_role),
        )
        .route(
            "/organization-roles",
            get(api::deployment::b2b::get_deployment_org_roles),
        )
        .route(
            "/settings/auth-settings",
            patch(api::deployment::settings::update_deployment_authetication_settings),
        )
        .route(
            "/settings/display-settings",
            patch(api::deployment::settings::update_deployment_ui_settings),
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
