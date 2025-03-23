use std::str::FromStr;

use sqlx::query;

use crate::{
    application::{AppError, AppState},
    core::models::{
        DeploymentAuthSettings, DeploymentDisplaySettings, DeploymentMode, DeploymentOrgSettings,
        DeploymentSocialConnection, DeploymentWithSettings,
    },
};

use super::Query;

pub struct GetDeploymentWithSettingsQuery {
    deployment_id: i64,
}

impl GetDeploymentWithSettingsQuery {
    pub fn new(deployment_id: i64) -> Self {
        Self { deployment_id }
    }
}

impl Query<DeploymentWithSettings> for GetDeploymentWithSettingsQuery {
    async fn execute(&self, app_state: &AppState) -> Result<DeploymentWithSettings, AppError> {
        let row = query!(
            r#"
            SELECT 
                deployments.id, 
                deployments.created_at, 
                deployments.updated_at, 
                deployments.deleted_at,
                deployments.maintenance_mode, 
                deployments.host, 
                deployments.publishable_key, 
                deployments.secret,
                deployments.mode,
                
                deployment_auth_settings.id as "auth_settings_id?", 
                deployment_auth_settings.created_at as "auth_settings_created_at?",
                deployment_auth_settings.updated_at as "auth_settings_updated_at?", 
                deployment_auth_settings.deleted_at as "auth_settings_deleted_at?",
                deployment_auth_settings.email_address::jsonb as "email_address?", 
                deployment_auth_settings.phone_number::jsonb as "phone_number?",
                deployment_auth_settings.username::jsonb as username, 
                deployment_auth_settings.first_name::jsonb as first_name,
                deployment_auth_settings.last_name::jsonb as last_name, 
                deployment_auth_settings.password::jsonb as password,
                deployment_auth_settings.backup_code::jsonb as backup_code, 
                deployment_auth_settings.web3_wallet::jsonb as web3_wallet,
                deployment_auth_settings.password_policy::jsonb as password_policy,
                deployment_auth_settings.auth_factors_enabled::jsonb as auth_factors_enabled,
                deployment_auth_settings.verification_policy::jsonb as verification_policy,
                deployment_auth_settings.second_factor_policy::text as second_factor_policy, 
                deployment_auth_settings.first_factor::text as first_factor, 
                deployment_auth_settings.second_factor::text as second_factor,
                deployment_auth_settings.passkey::jsonb as passkey,
                deployment_auth_settings.magic_link::jsonb as magic_link,
                array_to_json(deployment_auth_settings.alternate_first_factors)::jsonb as alternate_first_factors,
                array_to_json(deployment_auth_settings.alternate_second_factors)::jsonb as alternate_second_factors,
                
                deployment_display_settings.id as "display_settings_id?", 
                deployment_display_settings.created_at as "display_settings_created_at?",
                deployment_display_settings.updated_at as "display_settings_updated_at?", 
                deployment_display_settings.deleted_at as "display_settings_deleted_at?",
                deployment_display_settings.app_name, 
                deployment_display_settings.primary_color, 
                deployment_display_settings.tos_page_url, 
                deployment_display_settings.sign_in_page_url,
                deployment_display_settings.sign_up_page_url,
                deployment_display_settings.after_sign_out_one_page_url,
                deployment_display_settings.after_sign_out_all_page_url,
                deployment_display_settings.favicon_image_url,
                deployment_display_settings.logo_image_url,
                deployment_display_settings.privacy_policy_url,
                deployment_display_settings.signup_terms_statement, 
                deployment_display_settings.signup_terms_statement_shown,
                deployment_display_settings.button_config::jsonb as button_config, 
                deployment_display_settings.input_config::jsonb as input_config,
                
                deployment_org_settings.id as "org_settings_id?", 
                deployment_org_settings.created_at as "org_settings_created_at?",
                deployment_org_settings.updated_at as "org_settings_updated_at?", 
                deployment_org_settings.deleted_at as "org_settings_deleted_at?",
                deployment_org_settings.enabled, 
                deployment_org_settings.ip_allowlist_enabled, 
                deployment_org_settings.max_allowed_members,
                deployment_org_settings.allow_deletion, 
                deployment_org_settings.custom_role_enabled, 
                deployment_org_settings.default_role
            FROM deployments
            LEFT JOIN deployment_auth_settings 
                ON deployments.id = deployment_auth_settings.deployment_id 
                AND deployment_auth_settings.deleted_at IS NULL
            LEFT JOIN deployment_display_settings 
                ON deployments.id = deployment_display_settings.deployment_id 
                AND deployment_display_settings.deleted_at IS NULL
            LEFT JOIN deployment_org_settings 
                ON deployments.id = deployment_org_settings.deployment_id 
                AND deployment_org_settings.deleted_at IS NULL
            WHERE deployments.id = $1 AND deployments.deleted_at IS NULL
            "#,
            self.deployment_id,
        )
        .fetch_one(&app_state.pool)
        .await?;

        let mode = match row.mode.as_ref().map(|s| s.to_lowercase()).as_deref() {
            Some("production") => DeploymentMode::Production,
            Some("staging") => DeploymentMode::Staging,
            Some(mode) => {
                return Err(AppError::Database(sqlx::Error::Protocol(format!(
                    "Invalid deployment mode: {}",
                    mode
                ))))
            }
            None => {
                return Err(AppError::Database(sqlx::Error::Protocol(
                    "Mode is required".into(),
                )))
            }
        };

        Ok(DeploymentWithSettings {
            id: row.id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
            maintenance_mode: row.maintenance_mode.unwrap_or_default(),
            host: row.host.unwrap_or_default(),
            publishable_key: row.publishable_key.unwrap_or_default(),
            secret: row.secret.unwrap_or_default(),
            mode,
            auth_settings: if row.auth_settings_id.is_some() {
                Some(DeploymentAuthSettings {
                    id: row.auth_settings_id.unwrap(),
                    created_at: row.auth_settings_created_at,
                    updated_at: row.auth_settings_updated_at,
                    deleted_at: row.auth_settings_deleted_at,
                    email_address: serde_json::from_value(row.email_address.unwrap_or_default())
                        .unwrap(),
                    phone_number: serde_json::from_value(row.phone_number.unwrap_or_default())
                        .unwrap(),
                    username: serde_json::from_value(row.username.unwrap_or_default()).unwrap(),
                    first_name: serde_json::from_value(row.first_name.unwrap_or_default()).unwrap(),
                    last_name: serde_json::from_value(row.last_name.unwrap_or_default()).unwrap(),
                    password: serde_json::from_value(row.password.unwrap_or_default()).unwrap(),
                    backup_code: serde_json::from_value(row.backup_code.unwrap_or_default())
                        .unwrap(),
                    web3_wallet: serde_json::from_value(row.web3_wallet.unwrap_or_default())
                        .unwrap(),
                    auth_factors_enabled: serde_json::from_value(
                        row.auth_factors_enabled.unwrap_or_default(),
                    )
                    .unwrap_or_default(),
                    verification_policy: serde_json::from_value(
                        row.verification_policy.unwrap_or_default(),
                    )
                    .unwrap(),
                    passkey: serde_json::from_value(row.passkey.unwrap_or_default()).unwrap(),
                    magic_link: serde_json::from_value(row.magic_link.unwrap_or_default()).unwrap(),
                    second_factor_policy: row
                        .second_factor_policy
                        .map(|s| FromStr::from_str(&s).unwrap()),
                    first_factor: FromStr::from_str(&row.first_factor.unwrap_or_default()).unwrap(),
                    second_factor: row.second_factor.map(|s| FromStr::from_str(&s).unwrap()),
                    alternate_first_factors: row
                        .alternate_first_factors
                        .map(|v| serde_json::from_value(v).unwrap_or_default()),
                    alternate_second_factors: row
                        .alternate_second_factors
                        .map(|v| serde_json::from_value(v).unwrap_or_default()),
                    deployment_id: self.deployment_id,
                })
            } else {
                None
            },
            display_settings: if row.display_settings_id.is_some() {
                Some(DeploymentDisplaySettings {
                    id: row.display_settings_id.unwrap(),
                    created_at: row.display_settings_created_at,
                    updated_at: row.display_settings_updated_at,
                    deleted_at: row.display_settings_deleted_at,
                    deployment_id: self.deployment_id,
                    app_name: row.app_name.unwrap_or_default(),
                    primary_color: row.primary_color.unwrap_or_default(),
                    tos_page_url: row.tos_page_url,
                    sign_in_page_url: row.sign_in_page_url,
                    sign_up_page_url: row.sign_up_page_url,
                    after_sign_out_one_page_url: row.after_sign_out_one_page_url,
                    after_sign_out_all_page_url: row.after_sign_out_all_page_url,
                    favicon_image_url: row.favicon_image_url,
                    logo_image_url: row.logo_image_url,
                    privacy_policy_url: row.privacy_policy_url,
                    signup_terms_statement: row.signup_terms_statement,
                    signup_terms_statement_shown: row
                        .signup_terms_statement_shown
                        .unwrap_or_default(),
                    button_config: serde_json::from_value(row.button_config.unwrap_or_default())
                        .unwrap(),
                    input_config: serde_json::from_value(row.input_config.unwrap_or_default())
                        .unwrap(),
                })
            } else {
                None
            },
            org_settings: if row.org_settings_id.is_some() {
                Some(DeploymentOrgSettings {
                    id: row.org_settings_id.unwrap(),
                    created_at: row.org_settings_created_at,
                    updated_at: row.org_settings_updated_at,
                    deleted_at: row.org_settings_deleted_at,
                    deployment_id: self.deployment_id,
                    enabled: row.enabled.unwrap_or_default(),
                    ip_allowlist_enabled: row.ip_allowlist_enabled.unwrap_or_default(),
                    max_allowed_members: row.max_allowed_members.unwrap_or_default(),
                    allow_deletion: row.allow_deletion.unwrap_or_default(),
                    custom_role_enabled: row.custom_role_enabled.unwrap_or_default(),
                    default_role: row.default_role.unwrap_or_default(),
                })
            } else {
                None
            },
        })
    }
}

pub struct GetDeploymentSocialConnectionsQuery {
    deployment_id: i64,
}

impl GetDeploymentSocialConnectionsQuery {
    pub fn new(deployment_id: i64) -> Self {
        Self { deployment_id }
    }
}

impl Query<Vec<DeploymentSocialConnection>> for GetDeploymentSocialConnectionsQuery {
    async fn execute(
        &self,
        app_state: &AppState,
    ) -> Result<Vec<DeploymentSocialConnection>, AppError> {
        let row = query!(
            r#"
            SELECT 
                id,
                created_at,
                updated_at,
                deleted_at,
                deployment_id,
                provider,
                enabled,
                user_defined_scopes,
                credentials
            FROM deployment_social_connections
            WHERE deployment_id = $1 AND deleted_at IS NULL
            "#,
            self.deployment_id,
        )
        .fetch_all(&app_state.pool)
        .await?;

        Ok(row
            .into_iter()
            .map(|row| DeploymentSocialConnection {
                id: row.id,
                created_at: row.created_at,
                updated_at: row.updated_at,
                deleted_at: row.deleted_at,
                deployment_id: row.deployment_id,
                provider: row.provider.map(|s| FromStr::from_str(&s).unwrap()),
                enabled: row.enabled,
                user_defined_scopes: row.user_defined_scopes,
                credentials: row.credentials.map(|v| serde_json::from_value(v).unwrap()),
            })
            .collect())
    }
}
