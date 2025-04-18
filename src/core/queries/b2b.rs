use sqlx::query_as;

use crate::{
    application::{AppError, AppState},
    core::models::{DeploymentOrganizationRole, DeploymentWorkspaceRole},
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
            SELECT * FROM deployment_workspace_roles WHERE deployment_id = $1"#,
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
            r#"SELECT * FROM deployment_organization_roles WHERE deployment_id = $1"#,
            self.deployment_id
        )
        .fetch_all(&app_state.db_pool)
        .await?;

        Ok(rows)
    }
}
