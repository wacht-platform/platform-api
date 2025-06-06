use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::{
    application::AppState,
    core::services::clickhouse::{RecentSignup},
};

pub fn analytics_routes() -> Router<AppState> {
    Router::new()
        .route("/deployment/:deployment_id/analytics/stats", get(get_analytics_stats))
        .route("/deployment/:deployment_id/analytics/recent-signups", get(get_recent_signups))
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
}

#[derive(Debug, Serialize)]
struct RecentSignupsResponse {
    signups: Vec<RecentSignup>,
}

async fn get_analytics_stats(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<AnalyticsStatsResponse>, StatusCode> {
    let clickhouse = &app_state.clickhouse_service;
    
    let unique_signins = clickhouse.get_unique_signins(deployment_id, query.from, query.to).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let signups = clickhouse.get_signups(deployment_id, query.from, query.to).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let organizations_created = clickhouse.get_organizations_created(deployment_id, query.from, query.to).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let workspaces_created = clickhouse.get_workspaces_created(deployment_id, query.from, query.to).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let total_signups = clickhouse.get_total_signups(deployment_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AnalyticsStatsResponse {
        unique_signins,
        signups,
        organizations_created,
        workspaces_created,
        total_signups,
    }))
}

async fn get_recent_signups(
    State(app_state): State<AppState>,
    Path(deployment_id): Path<i64>,
    Query(query): Query<RecentSignupsQuery>,
) -> Result<Json<RecentSignupsResponse>, StatusCode> {
    let limit = query.limit.unwrap_or(10);
    
    let signups = app_state.clickhouse_service
        .get_recent_signups(deployment_id, limit)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(RecentSignupsResponse { signups }))
}
