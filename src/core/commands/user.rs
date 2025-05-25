use chrono::{Duration, Utc};

use crate::{
    application::{
        AppError, AppState,
        http::models::json::{
            AddToWaitlistRequest, CreateUserRequest, InviteUserRequest, UpdateUserRequest,
        },
    },
    core::models::{
        DeploymentInvitation, DeploymentWaitlistUser, UserDetails, UserWithIdentifiers,
    },
};

use super::Command;

pub struct CreateUserCommand {
    deployment_id: i64,
    request: CreateUserRequest,
}

impl CreateUserCommand {
    pub fn new(deployment_id: i64, request: CreateUserRequest) -> Self {
        Self {
            deployment_id,
            request,
        }
    }
}

impl Command for CreateUserCommand {
    type Output = UserWithIdentifiers;

    async fn execute(self, app_state: &AppState) -> Result<Self::Output, AppError> {
        // In a real implementation, this would create a user in the database
        // For now, we'll just return a mock user
        let now = Utc::now();

        let user = UserWithIdentifiers {
            id: app_state.sf.next_id()? as i64,
            created_at: now,
            updated_at: now,
            first_name: self.request.first_name,
            last_name: self.request.last_name,
            username: self.request.username,
            primary_email_address: Some(self.request.email_address),
            primary_phone_number: self.request.phone_number,
        };

        Ok(user)
    }
}

pub struct InviteUserCommand {
    deployment_id: i64,
    request: InviteUserRequest,
}

impl InviteUserCommand {
    pub fn new(deployment_id: i64, request: InviteUserRequest) -> Self {
        Self {
            deployment_id,
            request,
        }
    }
}

impl Command for InviteUserCommand {
    type Output = DeploymentInvitation;

    async fn execute(self, app_state: &AppState) -> Result<Self::Output, AppError> {
        let now = Utc::now();
        let expiry_days = self.request.expiry_days.unwrap_or(7);
        let expiry = now + Duration::days(expiry_days);

        let invitation = DeploymentInvitation {
            id: app_state.sf.next_id()? as i64,
            created_at: now,
            updated_at: now,
            deployment_id: self.deployment_id,
            first_name: self.request.first_name,
            last_name: self.request.last_name,
            email_address: self.request.email_address,
            expiry,
        };

        Ok(invitation)
    }
}

pub struct AddToWaitlistCommand {
    deployment_id: i64,
    request: AddToWaitlistRequest,
}

impl AddToWaitlistCommand {
    pub fn new(deployment_id: i64, request: AddToWaitlistRequest) -> Self {
        Self {
            deployment_id,
            request,
        }
    }
}

impl Command for AddToWaitlistCommand {
    type Output = DeploymentWaitlistUser;

    async fn execute(self, app_state: &AppState) -> Result<Self::Output, AppError> {
        let now = Utc::now();

        let waitlist_user = DeploymentWaitlistUser {
            id: app_state.sf.next_id()? as i64,
            created_at: now,
            updated_at: now,
            deployment_id: self.deployment_id,
            first_name: self.request.first_name,
            last_name: self.request.last_name,
            email_address: self.request.email_address,
        };

        Ok(waitlist_user)
    }
}

pub struct ApproveWaitlistUserCommand {
    deployment_id: i64,
    waitlist_user_id: i64,
}

impl ApproveWaitlistUserCommand {
    pub fn new(deployment_id: i64, waitlist_user_id: i64) -> Self {
        Self {
            deployment_id,
            waitlist_user_id,
        }
    }
}

impl Command for ApproveWaitlistUserCommand {
    type Output = UserWithIdentifiers;

    async fn execute(self, app_state: &AppState) -> Result<Self::Output, AppError> {
        // In a real implementation, this would:
        // 1. Fetch the waitlist user from the database
        // 2. Create a new user with the waitlist user's information
        // 3. Delete the waitlist user
        // 4. Return the new user

        // For now, we'll just return a mock user
        let now = Utc::now();

        // Simulate fetching the waitlist user
        let waitlist_user = DeploymentWaitlistUser {
            id: self.waitlist_user_id,
            created_at: now,
            updated_at: now,
            deployment_id: self.deployment_id,
            first_name: "Waitlist".to_string(),
            last_name: "User".to_string(),
            email_address: "waitlist@example.com".to_string(),
        };

        // Create a new user from the waitlist user
        let user = UserWithIdentifiers {
            id: app_state.sf.next_id()? as i64,
            created_at: now,
            updated_at: now,
            first_name: waitlist_user.first_name,
            last_name: waitlist_user.last_name,
            username: None,
            primary_email_address: Some(waitlist_user.email_address),
            primary_phone_number: None,
        };

        Ok(user)
    }
}

pub struct UpdateUserCommand {
    deployment_id: i64,
    user_id: i64,
    request: UpdateUserRequest,
}

impl UpdateUserCommand {
    pub fn new(deployment_id: i64, user_id: i64, request: UpdateUserRequest) -> Self {
        Self {
            deployment_id,
            user_id,
            request,
        }
    }
}

impl Command for UpdateUserCommand {
    type Output = UserDetails;

    async fn execute(self, app_state: &AppState) -> Result<Self::Output, AppError> {
        // Update the user with provided fields using compile-time verified queries
        match (
            &self.request.first_name,
            &self.request.last_name,
            &self.request.username,
            &self.request.public_metadata,
            &self.request.private_metadata,
        ) {
            (
                Some(first_name),
                Some(last_name),
                Some(username),
                Some(public_metadata),
                Some(private_metadata),
            ) => {
                sqlx::query!(
                    r#"
                    UPDATE users
                    SET updated_at = NOW(), first_name = $1, last_name = $2, username = $3,
                        public_metadata = $4, private_metadata = $5
                    WHERE deployment_id = $6 AND id = $7
                    "#,
                    first_name,
                    last_name,
                    username,
                    public_metadata,
                    private_metadata,
                    self.deployment_id,
                    self.user_id
                )
                .execute(&app_state.db_pool)
                .await?;
            }
            (Some(first_name), Some(last_name), Some(username), None, None) => {
                sqlx::query!(
                    r#"
                    UPDATE users
                    SET updated_at = NOW(), first_name = $1, last_name = $2, username = $3
                    WHERE deployment_id = $4 AND id = $5
                    "#,
                    first_name,
                    last_name,
                    username,
                    self.deployment_id,
                    self.user_id
                )
                .execute(&app_state.db_pool)
                .await?;
            }
            (Some(first_name), Some(last_name), None, None, None) => {
                sqlx::query!(
                    r#"
                    UPDATE users
                    SET updated_at = NOW(), first_name = $1, last_name = $2
                    WHERE deployment_id = $3 AND id = $4
                    "#,
                    first_name,
                    last_name,
                    self.deployment_id,
                    self.user_id
                )
                .execute(&app_state.db_pool)
                .await?;
            }
            (Some(first_name), None, None, None, None) => {
                sqlx::query!(
                    r#"
                    UPDATE users
                    SET updated_at = NOW(), first_name = $1
                    WHERE deployment_id = $2 AND id = $3
                    "#,
                    first_name,
                    self.deployment_id,
                    self.user_id
                )
                .execute(&app_state.db_pool)
                .await?;
            }
            (None, Some(last_name), None, None, None) => {
                sqlx::query!(
                    r#"
                    UPDATE users
                    SET updated_at = NOW(), last_name = $1
                    WHERE deployment_id = $2 AND id = $3
                    "#,
                    last_name,
                    self.deployment_id,
                    self.user_id
                )
                .execute(&app_state.db_pool)
                .await?;
            }
            (None, None, Some(username), None, None) => {
                sqlx::query!(
                    r#"
                    UPDATE users
                    SET updated_at = NOW(), username = $1
                    WHERE deployment_id = $2 AND id = $3
                    "#,
                    username,
                    self.deployment_id,
                    self.user_id
                )
                .execute(&app_state.db_pool)
                .await?;
            }
            (None, None, None, Some(public_metadata), None) => {
                sqlx::query!(
                    r#"
                    UPDATE users
                    SET updated_at = NOW(), public_metadata = $1
                    WHERE deployment_id = $2 AND id = $3
                    "#,
                    public_metadata,
                    self.deployment_id,
                    self.user_id
                )
                .execute(&app_state.db_pool)
                .await?;
            }
            (None, None, None, None, Some(private_metadata)) => {
                sqlx::query!(
                    r#"
                    UPDATE users
                    SET updated_at = NOW(), private_metadata = $1
                    WHERE deployment_id = $2 AND id = $3
                    "#,
                    private_metadata,
                    self.deployment_id,
                    self.user_id
                )
                .execute(&app_state.db_pool)
                .await?;
            }
            (None, None, None, Some(public_metadata), Some(private_metadata)) => {
                sqlx::query!(
                    r#"
                    UPDATE users
                    SET updated_at = NOW(), public_metadata = $1, private_metadata = $2
                    WHERE deployment_id = $3 AND id = $4
                    "#,
                    public_metadata,
                    private_metadata,
                    self.deployment_id,
                    self.user_id
                )
                .execute(&app_state.db_pool)
                .await?;
            }
            (
                Some(first_name),
                Some(last_name),
                None,
                Some(public_metadata),
                Some(private_metadata),
            ) => {
                sqlx::query!(
                    r#"
                    UPDATE users
                    SET updated_at = NOW(), first_name = $1, last_name = $2,
                        public_metadata = $3, private_metadata = $4
                    WHERE deployment_id = $5 AND id = $6
                    "#,
                    first_name,
                    last_name,
                    public_metadata,
                    private_metadata,
                    self.deployment_id,
                    self.user_id
                )
                .execute(&app_state.db_pool)
                .await?;
            }
            (None, None, None, None, None) => {
                // Just update the timestamp
                sqlx::query!(
                    r#"
                    UPDATE users
                    SET updated_at = NOW()
                    WHERE deployment_id = $1 AND id = $2
                    "#,
                    self.deployment_id,
                    self.user_id
                )
                .execute(&app_state.db_pool)
                .await?;
            }
            _ => {
                // Handle other combinations by building a more complex query
                // For now, we'll handle the most common cases above and fall back to a simple update
                sqlx::query!(
                    r#"
                    UPDATE users
                    SET updated_at = NOW()
                    WHERE deployment_id = $1 AND id = $2
                    "#,
                    self.deployment_id,
                    self.user_id
                )
                .execute(&app_state.db_pool)
                .await?;
            }
        }

        // Return the updated user details by fetching them
        use crate::core::queries::{GetUserDetailsQuery, Query};
        let user_details = GetUserDetailsQuery::new(self.deployment_id, self.user_id)
            .execute(app_state)
            .await?;

        Ok(user_details)
    }
}
