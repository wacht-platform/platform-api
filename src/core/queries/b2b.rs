use sqlx::{Row, query, query_as};

use crate::{
    application::{AppError, AppState},
    core::models::{DeploymentOrganizationRole, DeploymentWorkspaceRole, Organization, Workspace},
};

use super::Query;

pub struct GetDeploymentWorkspaceRolesQuery {
    deployment_id: i64,
}

impl GetDeploymentWorkspaceRolesQuery {
    pub fn new(deployment_id: i64) -> Self {
        Self { deployment_id }
    }
}

impl Query for GetDeploymentWorkspaceRolesQuery {
    type Output = Vec<DeploymentWorkspaceRole>;

    async fn execute(&self, app_state: &AppState) -> Result<Self::Output, AppError> {
        let rows = query_as!(
            DeploymentWorkspaceRole,
            r#"
            SELECT * FROM workspace_roles WHERE deployment_id = $1"#,
            self.deployment_id
        )
        .fetch_all(&app_state.db_pool)
        .await?;

        Ok(rows)
    }
}

pub struct GetDeploymentOrganizationRolesQuery {
    deployment_id: i64,
}

impl GetDeploymentOrganizationRolesQuery {
    pub fn new(deployment_id: i64) -> Self {
        Self { deployment_id }
    }
}

impl Query for GetDeploymentOrganizationRolesQuery {
    type Output = Vec<DeploymentOrganizationRole>;

    async fn execute(&self, app_state: &AppState) -> Result<Self::Output, AppError> {
        let rows = query_as!(
            DeploymentOrganizationRole,
            r#"SELECT * FROM organization_roles WHERE deployment_id = $1"#,
            self.deployment_id
        )
        .fetch_all(&app_state.db_pool)
        .await?;

        Ok(rows)
    }
}

pub struct DeploymentOrganizationListQuery {
    offset: i64,
    sort_key: Option<String>,
    sort_order: Option<String>,
    limit: i32,
    deployment_id: i64,
}

impl DeploymentOrganizationListQuery {
    pub fn new(id: i64) -> Self {
        Self {
            offset: 0,
            sort_key: None,
            sort_order: None,
            limit: 10,
            deployment_id: id,
        }
    }

    pub fn offset(&self, offset: i64) -> Self {
        Self {
            offset,
            sort_key: self.sort_key.clone(),
            sort_order: self.sort_order.clone(),
            limit: self.limit,
            deployment_id: self.deployment_id,
        }
    }

    pub fn limit(&self, limit: i32) -> Self {
        Self {
            offset: self.offset,
            sort_key: self.sort_key.clone(),
            sort_order: self.sort_order.clone(),
            limit,
            deployment_id: self.deployment_id,
        }
    }

    pub fn sort_key(&self, sort_key: Option<String>) -> Self {
        Self {
            offset: self.offset,
            sort_key,
            sort_order: self.sort_order.clone(),
            limit: self.limit,
            deployment_id: self.deployment_id,
        }
    }

    pub fn sort_order(&self, sort_order: Option<String>) -> Self {
        Self {
            offset: self.offset,
            sort_key: self.sort_key.clone(),
            sort_order,
            limit: self.limit,
            deployment_id: self.deployment_id,
        }
    }
}

impl Query for DeploymentOrganizationListQuery {
    type Output = Vec<Organization>;

    async fn execute(&self, app_state: &AppState) -> Result<Self::Output, AppError> {
        let mut query_str = String::from(
            r#"
            SELECT
                o.id, o.created_at, o.updated_at, o.deleted_at,
                o.name, o.image_url, o.description, o.member_count,
                o.public_metadata, o.private_metadata
            FROM organizations o
            WHERE o.deployment_id = $1 AND o.deleted_at IS NULL
            "#,
        );

        let sort_key = self.sort_key.as_deref().unwrap_or("created_at");
        let sort_order = self.sort_order.as_deref().unwrap_or("desc");
        query_str.push_str(&format!(" ORDER BY o.{} {}", sort_key, sort_order));

        query_str.push_str(" OFFSET $2 LIMIT $3");

        let rows = query(&query_str)
            .bind(self.deployment_id)
            .bind(self.offset)
            .bind(self.limit)
            .fetch_all(&app_state.db_pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| Organization {
                id: row.get("id"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                deleted_at: row.get("deleted_at"),
                name: row.get("name"),
                image_url: row.get("image_url"),
                description: row.get("description"),
                member_count: row.get("member_count"),
                public_metadata: row.get("public_metadata"),
                private_metadata: row.get("private_metadata"),
            })
            .collect())
    }
}

pub struct DeploymentWorkspaceListQuery {
    offset: i64,
    sort_key: Option<String>,
    sort_order: Option<String>,
    limit: i32,
    deployment_id: i64,
}

impl DeploymentWorkspaceListQuery {
    pub fn new(id: i64) -> Self {
        Self {
            offset: 0,
            sort_key: None,
            sort_order: None,
            limit: 10,
            deployment_id: id,
        }
    }

    pub fn offset(&self, offset: i64) -> Self {
        Self {
            offset,
            sort_key: self.sort_key.clone(),
            sort_order: self.sort_order.clone(),
            limit: self.limit,
            deployment_id: self.deployment_id,
        }
    }

    pub fn limit(&self, limit: i32) -> Self {
        Self {
            offset: self.offset,
            sort_key: self.sort_key.clone(),
            sort_order: self.sort_order.clone(),
            limit,
            deployment_id: self.deployment_id,
        }
    }

    pub fn sort_key(&self, sort_key: Option<String>) -> Self {
        Self {
            offset: self.offset,
            sort_key,
            sort_order: self.sort_order.clone(),
            limit: self.limit,
            deployment_id: self.deployment_id,
        }
    }

    pub fn sort_order(&self, sort_order: Option<String>) -> Self {
        Self {
            offset: self.offset,
            sort_key: self.sort_key.clone(),
            sort_order,
            limit: self.limit,
            deployment_id: self.deployment_id,
        }
    }
}

impl Query for DeploymentWorkspaceListQuery {
    type Output = Vec<Workspace>;

    async fn execute(&self, app_state: &AppState) -> Result<Self::Output, AppError> {
        let mut query_str = String::from(
            r#"
            SELECT
                w.id, w.created_at, w.updated_at, w.deleted_at,
                w.name, w.image_url, w.description, w.member_count,
                w.public_metadata, w.private_metadata
            FROM workspaces w
            WHERE w.deployment_id = $1 AND w.deleted_at IS NULL
            "#,
        );

        let sort_key = self.sort_key.as_deref().unwrap_or("created_at");
        let sort_order = self.sort_order.as_deref().unwrap_or("desc");
        query_str.push_str(&format!(" ORDER BY o.{} {}", sort_key, sort_order));

        query_str.push_str(" OFFSET $2 LIMIT $3");

        let rows = query(&query_str)
            .bind(self.deployment_id)
            .bind(self.offset)
            .bind(self.limit)
            .fetch_all(&app_state.db_pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| Workspace {
                id: row.get("id"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                deleted_at: row.get("deleted_at"),
                name: row.get("name"),
                image_url: row.get("image_url"),
                description: row.get("description"),
                member_count: row.get("member_count"),
                public_metadata: row.get("public_metadata"),
                private_metadata: row.get("private_metadata"),
            })
            .collect())
    }
}
