use std::str::FromStr;

use crate::{
    application::{AppError, AppState},
    core::models::{
        DeploymentAuthSettings, DeploymentDisplaySettings, DeploymentJwtTemplate, DeploymentMode,
        DeploymentOrgSettings, DeploymentRestrictions, DeploymentRestrictionsSignUpMode,
        DeploymentSocialConnection, DeploymentWithSettings,
    },
};
use sqlx::query;

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
                deployment_auth_settings.auth_factors_enabled::jsonb as auth_factors_enabled,
                deployment_auth_settings.verification_policy::jsonb as verification_policy,
                deployment_auth_settings.second_factor_policy::text as second_factor_policy, 
                deployment_auth_settings.first_factor::text as first_factor, 
                deployment_auth_settings.second_factor::text as second_factor,
                deployment_auth_settings.passkey::jsonb as passkey,
                deployment_auth_settings.magic_link::jsonb as magic_link,
                deployment_auth_settings.multi_session_support::jsonb as multi_session_support,
                deployment_auth_settings.session_token_lifetime,
                deployment_auth_settings.session_validity_period,
                deployment_auth_settings.session_inactive_timeout,
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
                deployment_org_settings.default_role,

                deployment_restrictions.id as "restrictions_id?",
                deployment_restrictions.created_at as "restrictions_created_at?",
                deployment_restrictions.updated_at as "restrictions_updated_at?",
                deployment_restrictions.deleted_at as "restrictions_deleted_at?",
                deployment_restrictions.allowlist_enabled,
                deployment_restrictions.blocklist_enabled,
                deployment_restrictions.block_subaddresses,
                deployment_restrictions.block_disposable_emails,
                deployment_restrictions.block_voip_numbers,
                deployment_restrictions.country_restrictions,
                deployment_restrictions.banned_keywords,
                deployment_restrictions.allowlisted_resources,
                deployment_restrictions.blocklisted_resources,
                deployment_restrictions.sign_up_mode
                
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
            LEFT JOIN deployment_restrictions
                ON deployments.id = deployment_restrictions.deployment_id
                AND deployment_restrictions.deleted_at IS NULL
            WHERE deployments.id = $1 AND deployments.deleted_at IS NULL
            "#,
            self.deployment_id,
        )
        .fetch_one(&app_state.db_pool)
        .await?;

        let mode = match row.mode.as_str() {
            "production" => DeploymentMode::Production,
            "staging" => DeploymentMode::Staging,
            _ => {
                return Err(AppError::Database(sqlx::Error::Protocol(format!(
                    "Invalid deployment mode: {}",
                    row.mode
                ))));
            }
        };

        Ok(DeploymentWithSettings {
            id: row.id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
            maintenance_mode: row.maintenance_mode,
            host: row.host,
            publishable_key: row.publishable_key,
            secret: row.secret,
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
                    username: serde_json::from_value(row.username).unwrap(),
                    first_name: serde_json::from_value(row.first_name).unwrap(),
                    last_name: serde_json::from_value(row.last_name).unwrap(),
                    password: serde_json::from_value(row.password).unwrap(),
                    auth_factors_enabled: serde_json::from_value(row.auth_factors_enabled).unwrap(),
                    verification_policy: serde_json::from_value(row.verification_policy).unwrap(),
                    passkey: serde_json::from_value(row.passkey).unwrap(),
                    magic_link: serde_json::from_value(row.magic_link).unwrap(),
                    second_factor_policy: FromStr::from_str(&row.second_factor_policy).unwrap(),
                    first_factor: FromStr::from_str(&row.first_factor).unwrap(),
                    second_factor: FromStr::from_str(&row.second_factor).unwrap(),
                    alternate_first_factors: row
                        .alternate_first_factors
                        .map(|v| serde_json::from_value(v).unwrap_or_default()),
                    alternate_second_factors: row
                        .alternate_second_factors
                        .map(|v| serde_json::from_value(v).unwrap_or_default()),
                    deployment_id: self.deployment_id,
                    multi_session_support: serde_json::from_value(row.multi_session_support)
                        .unwrap(),
                    session_token_lifetime: row.session_token_lifetime,
                    session_validity_period: row.session_validity_period,
                    session_inactive_timeout: row.session_inactive_timeout,
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
                    app_name: row.app_name,
                    primary_color: row.primary_color,
                    tos_page_url: row.tos_page_url,
                    sign_in_page_url: row.sign_in_page_url,
                    sign_up_page_url: row.sign_up_page_url,
                    after_sign_out_one_page_url: row.after_sign_out_one_page_url,
                    after_sign_out_all_page_url: row.after_sign_out_all_page_url,
                    favicon_image_url: row.favicon_image_url,
                    logo_image_url: row.logo_image_url,
                    privacy_policy_url: row.privacy_policy_url,
                    signup_terms_statement: row.signup_terms_statement,
                    signup_terms_statement_shown: row.signup_terms_statement_shown,
                    button_config: serde_json::from_value(row.button_config).unwrap(),
                    input_config: serde_json::from_value(row.input_config).unwrap(),
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
                    enabled: row.enabled,
                    ip_allowlist_enabled: row.ip_allowlist_enabled,
                    max_allowed_members: row.max_allowed_members,
                    allow_deletion: row.allow_deletion,
                    custom_role_enabled: row.custom_role_enabled,
                    default_role: row.default_role,
                })
            } else {
                None
            },
            restrictions: if row.restrictions_id.is_some() {
                Some(DeploymentRestrictions {
                    id: row.restrictions_id.unwrap(),
                    created_at: row.restrictions_created_at,
                    updated_at: row.restrictions_updated_at,
                    deleted_at: row.restrictions_deleted_at,
                    deployment_id: self.deployment_id,
                    allowlist_enabled: row.allowlist_enabled,
                    blocklist_enabled: row.blocklist_enabled,
                    block_subaddresses: row.block_subaddresses,
                    block_disposable_emails: row.block_disposable_emails,
                    block_voip_numbers: row.block_voip_numbers,
                    country_restrictions: serde_json::from_value(row.country_restrictions).unwrap(),
                    banned_keywords: row.banned_keywords,
                    allowlisted_resources: row.allowlisted_resources,
                    blocklisted_resources: row.blocklisted_resources,
                    sign_up_mode: DeploymentRestrictionsSignUpMode::from_str(&row.sign_up_mode)
                        .unwrap(),
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
                credentials
            FROM deployment_social_connections
            WHERE deployment_id = $1 AND deleted_at IS NULL
            "#,
            self.deployment_id,
        )
        .fetch_all(&app_state.db_pool)
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
                credentials: row
                    .credentials
                    .map(|v| serde_json::from_value(v).unwrap_or_default()),
            })
            .collect())
    }
}

pub struct GetDeploymentJwtTemplatesQuery {
    deployment_id: i64,
}

impl GetDeploymentJwtTemplatesQuery {
    pub fn new(deployment_id: i64) -> Self {
        Self { deployment_id }
    }
}

impl Query<Vec<DeploymentJwtTemplate>> for GetDeploymentJwtTemplatesQuery {
    async fn execute(&self, app_state: &AppState) -> Result<Vec<DeploymentJwtTemplate>, AppError> {
        let row = query!(
            r#"
            SELECT 
                id,
                created_at,
                updated_at,
                deleted_at,
                deployment_id,
                name,
                token_lifetime,
                allowed_clock_skew,
                custom_signing_key,
                template
            FROM deployment_jwt_templates
            WHERE deployment_id = $1 AND deleted_at IS NULL
            "#,
            self.deployment_id,
        )
        .fetch_all(&app_state.db_pool)
        .await?;

        let templates = row
            .into_iter()
            .map(|row| DeploymentJwtTemplate {
                id: row.id,
                created_at: row.created_at,
                updated_at: row.updated_at,
                deleted_at: row.deleted_at,
                deployment_id: row.deployment_id,
                name: row.name,
                token_lifetime: row.token_lifetime,
                allowed_clock_skew: row.allowed_clock_skew,
                custom_signing_key: row
                    .custom_signing_key
                    .map(|v| serde_json::from_value(v).unwrap_or_default()),
                template: row.template,
            })
            .collect();

        Ok(templates)
    }
}
