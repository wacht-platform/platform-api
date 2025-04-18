use std::collections::BTreeMap;

use sqlx::{Row, query};

use crate::{
    application::{AppError, AppState},
    core::models::{Deployment, ProjectWithDeployments},
};

use super::Query;

pub struct GetProjectsWithDeploymentQuery {
    oid: i64,
}

impl GetProjectsWithDeploymentQuery {
    pub fn new(oid: i64) -> Self {
        GetProjectsWithDeploymentQuery { oid }
    }
}

impl GetProjectsWithDeploymentQuery {
    fn create_deployment_from_row(row: &sqlx::postgres::PgRow) -> Deployment {
        Deployment {
            id: row
                .get::<Option<i64>, _>("deployment_id")
                .unwrap_or_default(),
            created_at: row
                .get::<Option<_>, _>("deployment_created_at")
                .unwrap_or_default(),
            updated_at: row
                .get::<Option<_>, _>("deployment_updated_at")
                .unwrap_or_default(),
            deleted_at: row.get("deployment_deleted_at"),
            maintenance_mode: row
                .get::<Option<bool>, _>("deployment_maintenance_mode")
                .unwrap_or_default(),
            backend_host: row
                .get::<Option<String>, _>("deployment_backend_host")
                .unwrap_or_default(),
            frontend_host: row
                .get::<Option<String>, _>("deployment_frontend_host")
                .unwrap_or_default(),
            publishable_key: row
                .get::<Option<String>, _>("deployment_publishable_key")
                .unwrap_or_default(),
            secret: row
                .get::<Option<String>, _>("deployment_secret")
                .unwrap_or_default(),
            project_id: row
                .get::<Option<i64>, _>("deployment_project_id")
                .unwrap_or_default(),
            mode: row
                .get::<Option<String>, _>("deployment_mode")
                .unwrap_or_default()
                .into(),
        }
    }
}
impl Query for GetProjectsWithDeploymentQuery {
    type Output = Vec<ProjectWithDeployments>;

    async fn execute(&self, app_state: &AppState) -> Result<Self::Output, AppError> {
        let rows = query(
            r#"
            SELECT
                p.id, p.created_at, p.updated_at, p.deleted_at, p.name, p.image_url,
                d.id as deployment_id, d.created_at as deployment_created_at,
                d.updated_at as deployment_updated_at, d.deleted_at as deployment_deleted_at,
                d.maintenance_mode as deployment_maintenance_mode, d.backend_host as deployment_backend_host,
                d.frontend_host as deployment_frontend_host,
                d.publishable_key as deployment_publishable_key, d.secret as deployment_secret,
                d.project_id as deployment_project_id, d.mode as deployment_mode
            FROM projects p
            LEFT JOIN deployments d ON p.id = d.project_id AND d.deleted_at IS NULL
            WHERE p.deleted_at IS NULL
            ORDER BY p.id DESC
            "#,
        )
        .fetch_all(&app_state.db_pool)
        .await?;

        let mut projects_map: BTreeMap<i64, ProjectWithDeployments> = BTreeMap::new();

        for row in rows {
            let project_id = row.get("id");

            if let Some(project) = projects_map.get_mut(&project_id) {
                if row.get::<Option<i64>, _>("deployment_id").is_some() {
                    project
                        .deployments
                        .push(Self::create_deployment_from_row(&row));
                }
            } else {
                let mut deployments = Vec::new();
                if row.get::<Option<i64>, _>("deployment_id").is_some() {
                    deployments.push(Self::create_deployment_from_row(&row));
                }

                projects_map.insert(
                    project_id,
                    ProjectWithDeployments {
                        id: project_id,
                        image_url: row.get("image_url"),
                        created_at: row.get("created_at"),
                        updated_at: row.get("updated_at"),
                        deleted_at: row.get("deleted_at"),
                        name: row.get("name"),
                        deployments,
                    },
                );
            }
        }

        Ok(projects_map.values().cloned().collect())
    }
}
