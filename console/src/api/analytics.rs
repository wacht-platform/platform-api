use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{application::HttpState, core::services::clickhouse::RecentSignup};

pub fn analytics_routes() -> Router<HttpState> {
    Router::new()
        .route(
            "/deployment/{deployment_id}/analytics/stats",
            get(get_analytics_stats),
        )
        .route(
            "/deployment/{deployment_id}/analytics/recent-signups",
            get(get_recent_signups),
        )
}

#[derive(Debug, Deserialize)]
struct AnalyticsQuery {
    from: DateTime<Utc>,
    to: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct RecentSignupsQuery {
    limit: Option<i32>,
}

#[derive(Debug, Serialize)]
struct AnalyticsStatsResponse {
    unique_signins: i64,
    signups: i64,
    organizations_created: i64,
    workspaces_created: i64,
    total_signups: i64,
    // Percentage changes compared to previous period
    unique_signins_change: Option<f64>,
    signups_change: Option<f64>,
    organizations_created_change: Option<f64>,
    workspaces_created_change: Option<f64>,
}

#[derive(Debug, Serialize)]
struct RecentSignupsResponse {
    signups: Vec<RecentSignup>,
}

async fn get_analytics_stats(
    State(app_state): State<HttpState>,
    Path(deployment_id): Path<i64>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<AnalyticsStatsResponse>, StatusCode> {
    let clickhouse = &app_state.clickhouse_service;

    // Calculate the duration of the current period
    let duration = query.to.signed_duration_since(query.from);

    // Calculate previous period (same duration, shifted back by that duration)
    // Today vs Yesterday, This Week vs Last Week, This Month vs Last Month
    let previous_from = query.from - duration;
    let previous_to = query.to - duration;

    // Get current period stats
    let unique_signins = clickhouse
        .get_unique_signins(deployment_id, query.from, query.to)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let signups = clickhouse
        .get_signups(deployment_id, query.from, query.to)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let organizations_created = clickhouse
        .get_organizations_created(deployment_id, query.from, query.to)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let workspaces_created = clickhouse
        .get_workspaces_created(deployment_id, query.from, query.to)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_signups = clickhouse
        .get_total_signups(deployment_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get previous period stats for comparison
    let previous_signins = clickhouse
        .get_unique_signins(deployment_id, previous_from, previous_to)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let previous_signups = clickhouse
        .get_signups(deployment_id, previous_from, previous_to)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let previous_orgs = clickhouse
        .get_organizations_created(deployment_id, previous_from, previous_to)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let previous_workspaces = clickhouse
        .get_workspaces_created(deployment_id, previous_from, previous_to)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Calculate percentage changes
    let calculate_change = |current: i64, previous: i64| -> Option<f64> {
        if previous == 0 {
            if current > 0 { Some(100.0) } else { None }
        } else {
            Some(((current - previous) as f64 / previous as f64) * 100.0)
        }
    };

    Ok(Json(AnalyticsStatsResponse {
        unique_signins,
        signups,
        organizations_created,
        workspaces_created,
        total_signups,
        unique_signins_change: calculate_change(unique_signins, previous_signins),
        signups_change: calculate_change(signups, previous_signups),
        organizations_created_change: calculate_change(organizations_created, previous_orgs),
        workspaces_created_change: calculate_change(workspaces_created, previous_workspaces),
    }))
}

async fn get_recent_signups(
    State(app_state): State<HttpState>,
    Path(deployment_id): Path<i64>,
    Query(query): Query<RecentSignupsQuery>,
) -> Result<Json<RecentSignupsResponse>, StatusCode> {
    let limit = query.limit.unwrap_or(10);

    let signups = app_state
        .clickhouse_service
        .get_recent_signups(deployment_id, limit)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(RecentSignupsResponse { signups }))
}
