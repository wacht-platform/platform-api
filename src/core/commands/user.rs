use chrono::{Duration, Utc};
use serde_json::json;
use std::collections::HashMap;

use crate::{
    application::{
        AppError, AppState,
        http::models::json::{CreateUserRequest, InviteUserRequest, UpdateUserRequest},
    },
    core::{
        models::{DeploymentInvitation, UserDetails, UserWithIdentifiers},
        queries::{GetDeploymentAuthSettingsQuery, Query},
    },
    utils::{
        security::{PasswordHasher, TotpGenerator},
        validation::UserValidator,
    },
};

use super::{Command, SendEmailCommand};

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
        let now = Utc::now();
        let user_id = app_state.sf.next_id()? as i64;

        let auth_settings = GetDeploymentAuthSettingsQuery::new(self.deployment_id)
            .execute(app_state)
            .await?;

        let mut tx = app_state.db_pool.begin().await?;

        UserValidator::validate_user_creation(
            &self.request.first_name,
            &self.request.last_name,
            &self.request.email_address,
            &self.request.phone_number,
            &self.request.username,
            &self.request.password,
            &auth_settings,
        )
        .map_err(|errors| {
            let error_messages: Vec<String> = errors
                .into_iter()
                .map(|e| format!("{}: {}", e.field, e.message))
                .collect();
            AppError::BadRequest(format!("Validation failed: {}", error_messages.join(", ")))
        })?;

        let hashed_password = if let Some(password) = &self.request.password {
            Some(PasswordHasher::hash_password(password)?)
        } else {
            None
        };

        let otp_secret = TotpGenerator::generate_secret()?;

        sqlx::query!(
            r#"
            INSERT INTO users (
                id, created_at, updated_at, first_name, last_name, username,
                password, schema_version, disabled, second_factor_policy,
                deployment_id, public_metadata, private_metadata, otp_secret, backup_codes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
            user_id,
            now,
            now,
            self.request.first_name,
            self.request.last_name,
            self.request.username,
            hashed_password,
            "v1",
            false,
            "optional",
            self.deployment_id,
            json!({}),
            json!({}),
            otp_secret,
            &Vec::<String>::new()
        )
        .execute(&mut *tx)
        .await?;

        let mut primary_email_address = None;
        let mut primary_phone_number = None;

        if let Some(email) = &self.request.email_address {
            let email_id = app_state.sf.next_id()? as i64;

            sqlx::query!(
                r#"
            INSERT INTO user_email_addresses (
                id, created_at, updated_at, deployment_id, user_id,
                email_address, is_primary, verified, verified_at, verification_strategy
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
                email_id,
                now,
                now,
                self.deployment_id,
                user_id,
                email,
                true,
                true,
                now,
                "otp"
            )
            .execute(&mut *tx)
            .await?;

            sqlx::query!(
                "UPDATE users SET primary_email_address_id = $1 WHERE id = $2",
                email_id,
                user_id
            )
            .execute(&mut *tx)
            .await?;

            primary_email_address = Some(email.clone());
        }

        if let Some(phone) = &self.request.phone_number {
            let phone_id = app_state.sf.next_id()? as i64;

            sqlx::query!(
                r#"
            INSERT INTO user_phone_numbers (
                id, created_at, updated_at, user_id, can_use_for_second_factor,
                phone_number, verified, verified_at, deployment_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
                phone_id,
                now,
                now,
                user_id,
                false,
                self.request.phone_number,
                true,
                now,
                self.deployment_id,
            )
            .execute(&app_state.db_pool)
            .await?;

            sqlx::query!(
                "UPDATE users SET primary_phone_number_id = $1 WHERE id = $2",
                phone_id,
                user_id
            )
            .execute(&mut *tx)
            .await?;

            primary_phone_number = Some(phone.clone());
        }

        let user = UserWithIdentifiers {
            id: user_id,
            created_at: now,
            updated_at: now,
            first_name: self.request.first_name,
            last_name: self.request.last_name,
            username: self.request.username,
            primary_email_address,
            primary_phone_number,
        };

        tx.commit().await?;

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
        let invitation_id = app_state.sf.next_id()? as i64;

        sqlx::query!(
            r#"
            INSERT INTO deployment_invitations (
                id, created_at, updated_at, deployment_id,
                first_name, last_name, email_address, expiry
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            invitation_id,
            now,
            now,
            self.deployment_id,
            self.request.first_name,
            self.request.last_name,
            self.request.email_address,
            expiry
        )
        .execute(&app_state.db_pool)
        .await?;

        let mut variables = HashMap::new();
        variables.insert("app_name".to_string(), "Your App".to_string());
        variables.insert(
            "app_logo".to_string(),
            "https://via.placeholder.com/150".to_string(),
        );
        variables.insert("first_name".to_string(), self.request.first_name.clone());
        variables.insert("last_name".to_string(), self.request.last_name.clone());
        variables.insert(
            "invitation.expires_in_days".to_string(),
            expiry_days.to_string(),
        );

        SendEmailCommand::new(
            self.deployment_id,
            "workspace_invite_template".to_string(),
            self.request.email_address.clone(),
            variables,
        )
        .execute(app_state)
        .await?;

        let invitation = DeploymentInvitation {
            id: invitation_id,
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
    type Output = DeploymentInvitation;

    async fn execute(self, app_state: &AppState) -> Result<Self::Output, AppError> {
        let now = Utc::now();

        let mut tx = app_state.db_pool.begin().await?;

        let waitlist_user = sqlx::query!(
            r#"
            SELECT id, created_at, updated_at, deployment_id,
                   first_name, last_name, email_address
            FROM deployment_waitlist_users
            WHERE id = $1 AND deployment_id = $2
            "#,
            self.waitlist_user_id,
            self.deployment_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| AppError::NotFound("Waitlist user not found".to_string()))?;

        let invitation_id = app_state.sf.next_id()? as i64;
        let expiry = now + Duration::days(7);

        let first_name = waitlist_user.first_name.unwrap_or_default();
        let last_name = waitlist_user.last_name.unwrap_or_default();
        let email_address = waitlist_user.email_address.unwrap_or_default();

        sqlx::query!(
            r#"
            INSERT INTO deployment_invitations (
                id, created_at, updated_at, deployment_id,
                first_name, last_name, email_address, expiry
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            invitation_id,
            now,
            now,
            self.deployment_id,
            first_name,
            last_name,
            email_address,
            expiry
        )
        .execute(&mut *tx)
        .await?;

        let mut variables = HashMap::new();
        variables.insert("app_name".to_string(), "Your App".to_string());
        variables.insert(
            "app_logo".to_string(),
            "https://via.placeholder.com/150".to_string(),
        );
        variables.insert("first_name".to_string(), first_name.clone());
        variables.insert("last_name".to_string(), last_name.clone());
        variables.insert("invitation.expires_in_days".to_string(), "7".to_string());

        SendEmailCommand::new(
            self.deployment_id,
            "waitlist_invite_template".to_string(),
            email_address.clone(),
            variables,
        )
        .execute(app_state)
        .await?;

        sqlx::query!(
            "DELETE FROM deployment_waitlist_users WHERE id = $1",
            self.waitlist_user_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        let invitation = DeploymentInvitation {
            id: invitation_id,
            created_at: now,
            updated_at: now,
            deployment_id: self.deployment_id,
            first_name: first_name.clone(),
            last_name: last_name.clone(),
            email_address: email_address.clone(),
            expiry,
        };

        Ok(invitation)
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

        use crate::core::queries::{GetUserDetailsQuery, Query};
        let user_details = GetUserDetailsQuery::new(self.deployment_id, self.user_id)
            .execute(app_state)
            .await?;

        Ok(user_details)
    }
}
