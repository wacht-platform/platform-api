use std::str::FromStr;

use super::Command;
use crate::{
    application::{
        AppError, AppState, DeploymentSocialConnectionUpsert,
        http::models::json::deployment_settings::DeploymentAuthSettingsUpdates,
    },
    core::models::{DeploymentSocialConnection, SocialConnectionProvider},
};
use chrono::Utc;
use serde_json::{Map, Value, json};

pub struct UpdateDeploymentAuthSettingsCommand {
    pub deployment_id: i64,
    pub settings: DeploymentAuthSettingsUpdates,
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
        let mut direct_updates: Vec<(&str, String)> = Vec::new();
        let mut jsonb_merges: Vec<(&str, Value)> = Vec::new();

        if let Some(json_val) = build_partial_json(self.settings.email.as_ref()) {
            jsonb_merges.push(("email_address", json_val));
        }
        if let Some(json_val) = build_partial_json(self.settings.phone.as_ref()) {
            jsonb_merges.push(("phone_number", json_val));
        }
        if let Some(json_val) = build_partial_json(self.settings.username.as_ref()) {
            jsonb_merges.push(("username", json_val));
        }
        if let Some(json_val) = build_partial_json(self.settings.password.as_ref()) {
            jsonb_merges.push(("password", json_val));
        }
        if let Some(json_val) = build_partial_json(self.settings.backup_code.as_ref()) {
            jsonb_merges.push(("backup_code", json_val));
        }
        if let Some(json_val) = build_partial_json(self.settings.web3_wallet.as_ref()) {
            jsonb_merges.push(("web3_wallet", json_val));
        }

        if let Some(name_settings) = &self.settings.name {
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
        if let Some(auth_factors) = &self.settings.authentication_factors {
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

        if let Some(restrictions) = &self.settings.restrictions {
            if let Some(json_val) = build_partial_json(Some(restrictions)) {
                jsonb_merges.push(("restrictions", json_val));
            }
        }
        if let Some(session) = &self.settings.session {
            if let Some(json_val) = build_partial_json(Some(session)) {
                jsonb_merges.push(("session_settings", json_val));
            }
        }

        if let Some(policy) = self.settings.second_factor_policy {
            direct_updates.push(("second_factor_policy", policy.to_string()));
        }

        let has_direct_updates = !direct_updates.is_empty();
        let has_jsonb_merges = !jsonb_merges.is_empty();

        if !has_direct_updates && !has_jsonb_merges {
            println!(
                "No settings updates to apply for deployment_id: {}",
                self.deployment_id
            );
            return Ok(());
        }

        let mut query_builder =
            sqlx::QueryBuilder::new("UPDATE deployment_auth_settings SET updated_at = NOW() ");

        for (_, (column, value)) in direct_updates.iter().enumerate() {
            query_builder.push(", ");
            query_builder.push(*column);
            query_builder.push(" = ");
            query_builder.push_bind(value.clone());
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

        query_builder.build().execute(&app_state.db_pool).await?;

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
