use crate::{
    application::{AppError, AppState},
    core::commands::Command,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteOrganizationCommand {
    pub deployment_id: i64,
    pub organization_id: i64,
}

impl DeleteOrganizationCommand {
    pub fn new(deployment_id: i64, organization_id: i64) -> Self {
        Self {
            deployment_id,
            organization_id,
        }
    }
}

impl Command for DeleteOrganizationCommand {
    type Output = ();

    async fn execute(self, app_state: &AppState) -> Result<Self::Output, AppError> {
        // First check if organization exists and belongs to deployment
        let exists = sqlx::query!(
            "SELECT id FROM organizations WHERE deployment_id = $1 AND id = $2",
            self.deployment_id,
            self.organization_id
        )
        .fetch_optional(&app_state.db_pool)
        .await?;

        if exists.is_none() {
            return Err(AppError::NotFound("Organization not found".to_string()));
        }

        // Delete organization (this should cascade to related tables)
        sqlx::query!(
            "DELETE FROM organizations WHERE deployment_id = $1 AND id = $2",
            self.deployment_id,
            self.organization_id
        )
        .execute(&app_state.db_pool)
        .await?;

        Ok(())
    }
}
