use crate::{
    application::{deployment_settings::DeploymentAuthSettingsUpdates, AppError, AppState},
    core::models::IndividualAuthSettings,
};

use super::Command;

pub struct UpdateDeploymentAuthSettingsCommand {
    pub deployment_id: i64,
    pub settings: DeploymentAuthSettingsUpdates,
}

impl Command<()> for UpdateDeploymentAuthSettingsCommand {
    async fn execute(&self, app_state: &AppState) -> Result<(), AppError> {
        let mut query_builder =
            sqlx::QueryBuilder::new("UPDATE deployment_auth_settings SET updated_at = NOW()");

        let mut needs_comma = false;

        if let Some(email) = &self.settings.email {
            if needs_comma {
                query_builder.push(", ");
            }
            query_builder
                .push(", email_address = ")
                .push_bind(serde_json::to_value(email).unwrap());
            needs_comma = true;
        }

        if let Some(phone) = &self.settings.phone {
            if needs_comma {
                query_builder.push(", ");
            }
            query_builder
                .push(", phone_number = ")
                .push_bind(serde_json::to_value(phone).unwrap());
            needs_comma = true;
        }

        if let Some(username) = &self.settings.username {
            if needs_comma {
                query_builder.push(", ");
            }
            query_builder
                .push(", username = ")
                .push_bind(serde_json::to_value(username).unwrap());
            needs_comma = true;
        }

        if let Some(name) = &self.settings.name {
            if needs_comma {
                query_builder.push(", ");
            }
            query_builder.push(", first_name = ").push_bind(
                serde_json::to_value(&IndividualAuthSettings {
                    enabled: name.first_name_enabled,
                    required: Some(name.first_name_required),
                })
                .unwrap(),
            );
            needs_comma = true;

            query_builder.push(", last_name = ").push_bind(
                serde_json::to_value(&IndividualAuthSettings {
                    enabled: name.last_name_enabled,
                    required: Some(name.last_name_required),
                })
                .unwrap(),
            );
        }

        if let Some(password) = &self.settings.password {
            if needs_comma {
                query_builder.push(", ");
            }
            query_builder
                .push(", password = ")
                .push_bind(serde_json::to_value(password).unwrap());
            needs_comma = true;
        }

        if let Some(auth_factors) = &self.settings.authentication_factors {
            if needs_comma {
                query_builder.push(", ");
            }

            // Build a JSON object with only the changed fields
            let mut auth_factors_json = serde_json::Map::new();

            if let Some(sso) = auth_factors.sso_enabled {
                auth_factors_json.insert("sso".to_string(), serde_json::Value::Bool(sso));
            }

            if let Some(web3) = auth_factors.web3_wallet_enabled {
                auth_factors_json.insert("web3_wallet".to_string(), serde_json::Value::Bool(web3));
            }

            if auth_factors.magic_link.is_some() {
                auth_factors_json.insert(
                    "email_magic_link".to_string(),
                    serde_json::Value::Bool(true),
                );
            }

            if let Some(email_otp) = auth_factors.email_otp_enabled {
                auth_factors_json
                    .insert("email_otp".to_string(), serde_json::Value::Bool(email_otp));
            }

            if let Some(phone_otp) = auth_factors.phone_otp_enabled {
                auth_factors_json
                    .insert("phone_otp".to_string(), serde_json::Value::Bool(phone_otp));
            }

            if auth_factors.passkey.is_some() {
                auth_factors_json.insert("passkey".to_string(), serde_json::Value::Bool(true));
            }

            if let Some(backup_code) = auth_factors.second_factor_backup_code_enabled {
                auth_factors_json.insert(
                    "backup_code".to_string(),
                    serde_json::Value::Bool(backup_code),
                );
            }

            if let Some(authenticator) = auth_factors.second_factor_authenticator_enabled {
                auth_factors_json.insert(
                    "authenticator".to_string(),
                    serde_json::Value::Bool(authenticator),
                );
            }

            // Only update if we have fields to update
            if !auth_factors_json.is_empty() {
                query_builder
                    .push(
                        ", auth_factors_enabled = COALESCE(auth_factors_enabled, '{}'::jsonb) || ",
                    )
                    .push_bind(serde_json::Value::Object(auth_factors_json));
            }

            if let Some(backup_code_enabled) = auth_factors.second_factor_backup_code_enabled {
                query_builder.push(", backup_code = ").push_bind(
                    serde_json::to_value(&IndividualAuthSettings {
                        enabled: backup_code_enabled,
                        required: None,
                    })
                    .unwrap(),
                );
            }

            if let Some(web3_enabled) = auth_factors.web3_wallet_enabled {
                query_builder.push(", web3_wallet = ").push_bind(
                    serde_json::to_value(&IndividualAuthSettings {
                        enabled: web3_enabled,
                        required: None,
                    })
                    .unwrap(),
                );
            }

            if let Some(magic_link) = &auth_factors.magic_link {
                query_builder
                    .push(", magic_link = ")
                    .push_bind(serde_json::to_value(magic_link).unwrap());
            }

            if let Some(passkey) = &auth_factors.passkey {
                query_builder
                    .push(", passkey = ")
                    .push_bind(serde_json::to_value(passkey).unwrap());
            }
        }

        query_builder
            .push(" WHERE deployment_id = ")
            .push_bind(self.deployment_id);

        query_builder.build().execute(&app_state.pool).await?;

        Ok(())
    }
}
