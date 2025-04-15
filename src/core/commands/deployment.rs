use std::str::FromStr;

use super::Command;
use crate::{
    application::{
        AppError, AppState, DeploymentRestrictionsUpdates, DeploymentSocialConnectionUpsert,
        http::models::json::deployment_settings::DeploymentAuthSettingsUpdates,
    },
    core::models::{DeploymentSocialConnection, SocialConnectionProvider},
};
use chrono::Utc;
use serde_json::{Map, Value, json};

pub struct UpdateDeploymentAuthSettingsCommand {
    pub deployment_id: i64,
    pub updates: DeploymentAuthSettingsUpdates,
}

impl UpdateDeploymentAuthSettingsCommand {
    pub fn new(deployment_id: i64, updates: DeploymentAuthSettingsUpdates) -> Self {
        Self {
            deployment_id,
            updates,
        }
    }
}

fn build_partial_json<T: serde::Serialize>(data: Option<&T>) -> Option<Value> {
    data.and_then(|d| match serde_json::to_value(d) {
        Ok(Value::Object(map)) => {
            let filtered_map: Map<String, Value> =
                map.into_iter().filter(|(_, v)| !v.is_null()).collect();
            if filtered_map.is_empty() {
                None
            } else {
                Some(Value::Object(filtered_map))
            }
        }
        Ok(_) => None,
        Err(_) => None,
    })
}

impl Command for UpdateDeploymentAuthSettingsCommand {
    type Output = ();

    async fn execute(self, app_state: &AppState) -> Result<Self::Output, AppError> {
        let mut text_updates: Vec<(&str, String)> = Vec::new();
        let mut int_updates: Vec<(&str, i64)> = Vec::new();
        let mut jsonb_merges: Vec<(&str, Value)> = Vec::new();

        if let Some(json_val) = build_partial_json(self.updates.email.as_ref()) {
            jsonb_merges.push(("email_address", json_val));
        }
        if let Some(json_val) = build_partial_json(self.updates.phone.as_ref()) {
            jsonb_merges.push(("phone_number", json_val));
        }
        if let Some(json_val) = build_partial_json(self.updates.username.as_ref()) {
            jsonb_merges.push(("username", json_val));
        }
        if let Some(json_val) = build_partial_json(self.updates.password.as_ref()) {
            jsonb_merges.push(("password", json_val));
        }
        if let Some(json_val) = build_partial_json(self.updates.backup_code.as_ref()) {
            jsonb_merges.push(("backup_code", json_val));
        }
        if let Some(json_val) = build_partial_json(self.updates.web3_wallet.as_ref()) {
            jsonb_merges.push(("web3_wallet", json_val));
        }

        if let Some(name_settings) = &self.updates.name {
            let mut first_name_partial = Map::new();
            if let Some(enabled) = name_settings.first_name_enabled {
                first_name_partial.insert("enabled".to_string(), json!(enabled));
            }
            if let Some(required) = name_settings.first_name_required {
                first_name_partial.insert("required".to_string(), json!(required));
            }
            if !first_name_partial.is_empty() {
                jsonb_merges.push(("first_name", Value::Object(first_name_partial)));
            }

            let mut last_name_partial = Map::new();
            if let Some(enabled) = name_settings.last_name_enabled {
                last_name_partial.insert("enabled".to_string(), json!(enabled));
            }
            if let Some(required) = name_settings.last_name_required {
                last_name_partial.insert("required".to_string(), json!(required));
            }
            if !last_name_partial.is_empty() {
                jsonb_merges.push(("last_name", Value::Object(last_name_partial)));
            }
        }

        let mut auth_factors_enabled_updates = Map::new();
        let mut process_auth_factors = false;
        if let Some(auth_factors) = &self.updates.authentication_factors {
            process_auth_factors = true;

            if let Some(json_val) = build_partial_json(auth_factors.magic_link.as_ref()) {
                jsonb_merges.push(("magic_link", json_val));
                if let Some(ml) = &auth_factors.magic_link {
                    if let Some(enabled) = ml.enabled {
                        auth_factors_enabled_updates
                            .insert("email_magic_link".to_string(), json!(enabled));
                    }
                }
            }
            if let Some(json_val) = build_partial_json(auth_factors.passkey.as_ref()) {
                jsonb_merges.push(("passkey", json_val));
                if let Some(pk) = &auth_factors.passkey {
                    if let Some(enabled) = pk.enabled {
                        auth_factors_enabled_updates.insert("passkey".to_string(), json!(enabled));
                    }
                }
            }

            if let Some(email_password) = auth_factors.email_password_enabled {
                auth_factors_enabled_updates
                    .insert("email_password".to_string(), json!(email_password));
            }
            if let Some(username_password) = auth_factors.username_password_enabled {
                auth_factors_enabled_updates
                    .insert("username_password".to_string(), json!(username_password));
            }
            if let Some(sso) = auth_factors.sso_enabled {
                auth_factors_enabled_updates.insert("sso".to_string(), json!(sso));
            }
            if let Some(web3_enabled) = auth_factors.web3_wallet_enabled {
                auth_factors_enabled_updates.insert("web3_wallet".to_string(), json!(web3_enabled));
            }
            if let Some(email_otp) = auth_factors.email_otp_enabled {
                auth_factors_enabled_updates.insert("email_otp".to_string(), json!(email_otp));
            }
            if let Some(phone_otp) = auth_factors.phone_otp_enabled {
                auth_factors_enabled_updates.insert("phone_otp".to_string(), json!(phone_otp));
            }

            if let Some(enabled) = auth_factors.second_factor_authenticator_enabled {
                auth_factors_enabled_updates.insert("authenticator".to_string(), json!(enabled));
            }
            if let Some(enabled) = auth_factors.second_factor_backup_code_enabled {
                auth_factors_enabled_updates.insert("backup_code".to_string(), json!(enabled));
            }
        }

        if process_auth_factors && !auth_factors_enabled_updates.is_empty() {
            jsonb_merges.push((
                "auth_factors_enabled",
                Value::Object(auth_factors_enabled_updates),
            ));
        }

        if let Some(policy) = self.updates.second_factor_policy {
            text_updates.push(("second_factor_policy", policy.to_string()));
        }

        if let Some(session) = &self.updates.multi_session_support {
            jsonb_merges.push(("multi_session_support", serde_json::to_value(session)?));
        }

        if let Some(session_token_lifetime) = &self.updates.session_token_lifetime {
            int_updates.push(("session_token_lifetime", *session_token_lifetime));
        }

        if let Some(session_validity_period) = &self.updates.session_validity_period {
            int_updates.push(("session_validity_period", *session_validity_period));
        }

        if let Some(session_inactive_timeout) = &self.updates.session_inactive_timeout {
            int_updates.push(("session_inactive_timeout", *session_inactive_timeout));
        }

        let has_text_updates = !text_updates.is_empty();
        let has_int_updates = !int_updates.is_empty();
        let has_jsonb_merges = !jsonb_merges.is_empty();

        if !has_text_updates && !has_int_updates && !has_jsonb_merges {
            println!(
                "No settings updates to apply for deployment_id: {}",
                self.deployment_id
            );
            return Ok(());
        }

        let mut query_builder =
            sqlx::QueryBuilder::new("UPDATE deployment_auth_settings SET updated_at = NOW() ");

        for (_, (column, value)) in text_updates.iter().enumerate() {
            query_builder.push(", ");
            query_builder.push(*column);
            query_builder.push(" = ");
            query_builder.push_bind(value);
        }

        for (_, (column, value)) in int_updates.iter().enumerate() {
            query_builder.push(", ");
            query_builder.push(*column);
            query_builder.push(" = ");
            query_builder.push_bind(value);
        }

        for (column, json_val) in jsonb_merges {
            query_builder.push(", ");
            let fragment = format!("{col} = COALESCE({col}, '{{}}'::jsonb) || ", col = column);
            query_builder.push(fragment);
            query_builder.push_bind(json_val);
            query_builder.push("::jsonb");
        }

        query_builder.push(" WHERE deployment_id = ");
        query_builder.push_bind(self.deployment_id);

        query_builder
            .build()
            .execute(&app_state.db_pool)
            .await
            .unwrap();

        Ok(())
    }
}

pub struct UpsertDeploymentSocialConnectionCommand {
    pub deployment_id: i64,
    pub connection: DeploymentSocialConnectionUpsert,
}

impl UpsertDeploymentSocialConnectionCommand {
    pub fn new(deployment_id: i64, connection: DeploymentSocialConnectionUpsert) -> Self {
        Self {
            deployment_id,
            connection,
        }
    }
}
impl Command for UpsertDeploymentSocialConnectionCommand {
    type Output = DeploymentSocialConnection;

    async fn execute(self, app_state: &AppState) -> Result<Self::Output, AppError> {
        let result = sqlx::query!(
            r#"
            INSERT INTO deployment_social_connections (id, created_at, updated_at, deployment_id, provider, enabled, credentials)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (deployment_id, provider) DO UPDATE SET updated_at = NOW(), enabled = EXCLUDED.enabled, credentials = EXCLUDED.credentials RETURNING *
            "#,
            app_state.sf.next_id()? as i64,
            Utc::now(),
            Utc::now(),
            self.deployment_id,
            self.connection.provider.map(|p| String::from(p)),
            self.connection.enabled,
            serde_json::to_value(self.connection.credentials).unwrap(),
        )
        .fetch_one(&app_state.db_pool)
        .await?;

        let connection = DeploymentSocialConnection {
            id: result.id,
            created_at: result.created_at,
            updated_at: result.updated_at,
            deleted_at: result.deleted_at,
            deployment_id: result.deployment_id,
            provider: SocialConnectionProvider::from_str(&result.provider.unwrap()).ok(),
            enabled: result.enabled,
            credentials: serde_json::from_value(result.credentials.unwrap()).unwrap_or(None),
        };

        Ok(connection)
    }
}

pub struct UpdateDeploymentRestrictionsCommand {
    pub deployment_id: i64,
    pub updates: DeploymentRestrictionsUpdates,
}

impl UpdateDeploymentRestrictionsCommand {
    pub fn new(deployment_id: i64, updates: DeploymentRestrictionsUpdates) -> Self {
        Self {
            deployment_id,
            updates,
        }
    }
}

impl Command for UpdateDeploymentRestrictionsCommand {
    type Output = ();

    async fn execute(self, app_state: &AppState) -> Result<Self::Output, AppError> {
        let mut query_builder =
            sqlx::QueryBuilder::new("UPDATE deployment_restrictions SET updated_at = NOW() ");

        if let Some(allowlist_enabled) = self.updates.allowlist_enabled {
            query_builder.push(", allowlist_enabled = ");
            query_builder.push_bind(allowlist_enabled);
        }

        if let Some(blocklist_enabled) = self.updates.blocklist_enabled {
            query_builder.push(", blocklist_enabled = ");
            query_builder.push_bind(blocklist_enabled);
        }

        if let Some(block_subaddresses) = self.updates.block_subaddresses {
            query_builder.push(", block_subaddresses = ");
            query_builder.push_bind(block_subaddresses);
        }

        if let Some(block_disposable_emails) = self.updates.block_disposable_emails {
            query_builder.push(", block_disposable_emails = ");
            query_builder.push_bind(block_disposable_emails);
        }

        if let Some(block_voip_numbers) = self.updates.block_voip_numbers {
            query_builder.push(", block_voip_numbers = ");
            query_builder.push_bind(block_voip_numbers);
        }

        if let Some(country_restrictions) = self.updates.country_restrictions {
            query_builder.push(", country_restrictions = ");
            query_builder.push_bind(serde_json::to_value(country_restrictions)?);
        }

        if let Some(banned_keywords) = self.updates.banned_keywords {
            query_builder.push(", banned_keywords = ");
            query_builder.push_bind(banned_keywords);
        }

        if let Some(allowlisted_resources) = self.updates.allowlisted_resources {
            query_builder.push(", allowlisted_resources = ");
            query_builder.push_bind(allowlisted_resources);
        }

        if let Some(blocklisted_resources) = self.updates.blocklisted_resources {
            query_builder.push(", blocklisted_resources = ");
            query_builder.push_bind(blocklisted_resources);
        }

        if let Some(sign_up_mode) = self.updates.sign_up_mode {
            query_builder.push(", sign_up_mode = ");
            query_builder.push_bind(sign_up_mode.to_string());
        }

        query_builder.push(" WHERE deployment_id = ");
        query_builder.push_bind(self.deployment_id);

        query_builder.build().execute(&app_state.db_pool).await?;

        Ok(().into())
    }
}
