use crate::{
    application::{AppError, AppState},
    core::{commands::Command, models::OrganizationMemberDetails},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AddOrganizationMemberCommand {
    pub deployment_id: i64,
    pub organization_id: i64,
    pub user_id: i64,
    pub role_ids: Vec<i64>,
}

impl AddOrganizationMemberCommand {
    pub fn new(deployment_id: i64, organization_id: i64, user_id: i64, role_ids: Vec<i64>) -> Self {
        Self {
            deployment_id,
            organization_id,
            user_id,
            role_ids,
        }
    }
}

impl Command for AddOrganizationMemberCommand {
    type Output = OrganizationMemberDetails;

    async fn execute(self, app_state: &AppState) -> Result<Self::Output, AppError> {
        // Check if user exists
        let user_exists = sqlx::query!(
            "SELECT id FROM users WHERE id = $1",
            self.user_id
        )
        .fetch_optional(&app_state.db_pool)
        .await?;

        if user_exists.is_none() {
            return Err(AppError::NotFound("User not found".to_string()));
        }

        // Check if organization exists
        let org_exists = sqlx::query!(
            "SELECT id FROM organizations WHERE deployment_id = $1 AND id = $2",
            self.deployment_id,
            self.organization_id
        )
        .fetch_optional(&app_state.db_pool)
        .await?;

        if org_exists.is_none() {
            return Err(AppError::NotFound("Organization not found".to_string()));
        }

        // Check if user is already a member
        let existing_membership = sqlx::query!(
            "SELECT id FROM organization_memberships WHERE organization_id = $1 AND user_id = $2",
            self.organization_id,
            self.user_id
        )
        .fetch_optional(&app_state.db_pool)
        .await?;

        if existing_membership.is_some() {
            return Err(AppError::BadRequest("User is already a member of this organization".to_string()));
        }

        // Create membership
        let membership = sqlx::query!(
            r#"
            INSERT INTO organization_memberships (id, organization_id, user_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, created_at, updated_at
            "#,
            app_state.sf.next_id()? as i64,
            self.organization_id,
            self.user_id,
            chrono::Utc::now(),
            chrono::Utc::now()
        )
        .fetch_one(&app_state.db_pool)
        .await?;

        // Add role associations
        for role_id in &self.role_ids {
            sqlx::query!(
                r#"
                INSERT INTO organization_membership_roles (organization_membership_id, organization_role_id, organization_id)
                VALUES ($1, $2, $3)
                "#,
                membership.id,
                role_id,
                self.organization_id
            )
            .execute(&app_state.db_pool)
            .await?;
        }

        // Update organization member count
        sqlx::query!(
            "UPDATE organizations SET member_count = member_count + 1 WHERE id = $1",
            self.organization_id
        )
        .execute(&app_state.db_pool)
        .await?;

        // Fetch and return the complete member details
        let member_details = sqlx::query!(
            r#"
            SELECT
                om.id, om.created_at, om.updated_at,
                om.organization_id, om.user_id,
                u.first_name, u.last_name, u.username,
                u.created_at as user_created_at,
                e.email_address as "primary_email_address?",
                p.phone_number as "primary_phone_number?"
            FROM organization_memberships om
            JOIN users u ON om.user_id = u.id
            LEFT JOIN user_email_addresses e ON u.primary_email_address_id = e.id
            LEFT JOIN user_phone_numbers p ON u.primary_phone_number_id = p.id
            WHERE om.id = $1
            "#,
            membership.id
        )
        .fetch_one(&app_state.db_pool)
        .await?;

        Ok(OrganizationMemberDetails {
            id: member_details.id,
            created_at: member_details.created_at,
            updated_at: member_details.updated_at,
            organization_id: member_details.organization_id,
            user_id: member_details.user_id,
            roles: vec![], // TODO: Fetch actual roles
            first_name: member_details.first_name,
            last_name: member_details.last_name,
            username: if member_details.username.is_empty() {
                None
            } else {
                Some(member_details.username)
            },
            primary_email_address: member_details.primary_email_address,
            primary_phone_number: member_details.primary_phone_number,
            user_created_at: member_details.user_created_at,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateOrganizationMemberCommand {
    pub deployment_id: i64,
    pub organization_id: i64,
    pub membership_id: i64,
    pub role_ids: Vec<i64>,
}

impl UpdateOrganizationMemberCommand {
    pub fn new(deployment_id: i64, organization_id: i64, membership_id: i64, role_ids: Vec<i64>) -> Self {
        Self {
            deployment_id,
            organization_id,
            membership_id,
            role_ids,
        }
    }
}

impl Command for UpdateOrganizationMemberCommand {
    type Output = ();

    async fn execute(self, app_state: &AppState) -> Result<Self::Output, AppError> {
        // Check if membership exists
        let membership_exists = sqlx::query!(
            "SELECT id FROM organization_memberships WHERE id = $1 AND organization_id = $2",
            self.membership_id,
            self.organization_id
        )
        .fetch_optional(&app_state.db_pool)
        .await?;

        if membership_exists.is_none() {
            return Err(AppError::NotFound("Organization membership not found".to_string()));
        }

        // Remove existing role associations
        sqlx::query!(
            "DELETE FROM organization_membership_roles WHERE organization_membership_id = $1",
            self.membership_id
        )
        .execute(&app_state.db_pool)
        .await?;

        // Add new role associations
        for role_id in &self.role_ids {
            sqlx::query!(
                r#"
                INSERT INTO organization_membership_roles (organization_membership_id, organization_role_id, organization_id)
                VALUES ($1, $2, $3)
                "#,
                self.membership_id,
                role_id,
                self.organization_id
            )
            .execute(&app_state.db_pool)
            .await?;
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveOrganizationMemberCommand {
    pub deployment_id: i64,
    pub organization_id: i64,
    pub membership_id: i64,
}

impl RemoveOrganizationMemberCommand {
    pub fn new(deployment_id: i64, organization_id: i64, membership_id: i64) -> Self {
        Self {
            deployment_id,
            organization_id,
            membership_id,
        }
    }
}

impl Command for RemoveOrganizationMemberCommand {
    type Output = ();

    async fn execute(self, app_state: &AppState) -> Result<Self::Output, AppError> {
        // Check if membership exists
        let membership_exists = sqlx::query!(
            "SELECT id FROM organization_memberships WHERE id = $1 AND organization_id = $2",
            self.membership_id,
            self.organization_id
        )
        .fetch_optional(&app_state.db_pool)
        .await?;

        if membership_exists.is_none() {
            return Err(AppError::NotFound("Organization membership not found".to_string()));
        }

        // Delete membership (this should cascade to role associations)
        sqlx::query!(
            "DELETE FROM organization_memberships WHERE id = $1",
            self.membership_id
        )
        .execute(&app_state.db_pool)
        .await?;

        // Update organization member count
        sqlx::query!(
            "UPDATE organizations SET member_count = member_count - 1 WHERE id = $1",
            self.organization_id
        )
        .execute(&app_state.db_pool)
        .await?;

        Ok(())
    }
}
