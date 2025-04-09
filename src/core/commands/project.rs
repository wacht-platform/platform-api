use crate::{
    application::{AppError, AppState},
    core::models::{
        AuthFactorsEnabled, Deployment, DeploymentAuthSettings, DeploymentDisplaySettings,
        DeploymentMode, DeploymentOrgSettings, EmailSettings, FirstFactor, IndividualAuthSettings,
        PasswordSettings, PhoneSettings, ProjectWithDeployments, SecondFactor, SecondFactorPolicy,
        UsernameSettings, VerificationPolicy,
    },
};
use base64::{Engine, prelude::BASE64_STANDARD};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{Command, UploadToCdnCommand};

pub struct CreateProjectCommand {
    name: String,
    logo: Vec<u8>,
    has_logo: bool,
    auth_methods: Vec<String>,
}

impl CreateProjectCommand {
    pub fn new(name: String, logo: Vec<u8>, auth_methods: Vec<String>) -> Self {
        let has_logo = !logo.is_empty();
        Self {
            name,
            logo,
            has_logo,
            auth_methods,
        }
    }

    fn generate_key(prefix: &str) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);

        let mut random_chars = Vec::with_capacity(32);
        let charset: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut num = timestamp;

        for _ in 0..32 {
            let idx = (num % charset.len() as u128) as usize;
            random_chars.push(charset[idx] as char);
            num = num.wrapping_mul(17).wrapping_add(3);
        }

        let random_part: String = random_chars.into_iter().collect();
        format!("{}_{}", prefix, random_part)
    }

    fn create_auth_settings(&self, deployment_id: i64) -> DeploymentAuthSettings {
        let email_enabled = self.auth_methods.contains(&"email".to_string());
        let phone_enabled = self.auth_methods.contains(&"phone".to_string());
        let username_enabled = self.auth_methods.contains(&"username".to_string());

        let mut first_factor = FirstFactor::EmailPassword;
        let mut alternate_first_factors: Vec<FirstFactor> = Vec::new();

        if email_enabled {
            first_factor = FirstFactor::EmailPassword;
            if phone_enabled {
                alternate_first_factors.push(FirstFactor::PhoneOtp);
            }
            if username_enabled {
                alternate_first_factors.push(FirstFactor::UsernamePassword);
            }
        } else if phone_enabled {
            first_factor = FirstFactor::PhoneOtp;
            if username_enabled {
                alternate_first_factors.push(FirstFactor::UsernamePassword);
            }
        } else if username_enabled {
            first_factor = FirstFactor::UsernamePassword;
        }

        let email_settings = EmailSettings {
            enabled: email_enabled,
            required: email_enabled,
            ..EmailSettings::default()
        };

        let phone_settings = PhoneSettings {
            enabled: phone_enabled,
            required: phone_enabled,
            ..PhoneSettings::default()
        };

        let username_settings = UsernameSettings {
            enabled: username_enabled,
            required: username_enabled,
            ..UsernameSettings::default()
        };

        let password_settings = PasswordSettings::default();
        let first_name_settings = IndividualAuthSettings::default();
        let last_name_settings = IndividualAuthSettings::default();
        let backup_code_settings = IndividualAuthSettings {
            enabled: false,
            required: None,
        };
        let web3_wallet_settings = IndividualAuthSettings {
            enabled: false,
            required: None,
        };

        let auth_factors_enabled = AuthFactorsEnabled::default()
            .with_email(email_enabled)
            .with_phone(phone_enabled)
            .with_username(username_enabled);

        let verification_policy = VerificationPolicy {
            phone_number: phone_enabled,
            email: email_enabled,
        };

        DeploymentAuthSettings {
            deployment_id,
            email_address: email_settings,
            phone_number: phone_settings,
            username: username_settings,
            first_factor: first_factor,
            alternate_first_factors: Some(alternate_first_factors),
            first_name: first_name_settings,
            last_name: last_name_settings,
            password: password_settings,
            backup_code: backup_code_settings,
            web3_wallet: web3_wallet_settings,
            auth_factors_enabled: auth_factors_enabled,
            verification_policy: verification_policy,
            second_factor_policy: Some(SecondFactorPolicy::None),
            second_factor: Some(SecondFactor::None),
            ..DeploymentAuthSettings::default()
        }
    }

    fn create_display_settings(
        &self,
        deployment_id: i64,
        frontend_host: String,
    ) -> DeploymentDisplaySettings {
        DeploymentDisplaySettings {
            deployment_id,
            app_name: self.name.clone(),
            after_sign_out_all_page_url: Some(format!("{}/sign-in", frontend_host)),
            after_sign_out_one_page_url: Some(format!("{}/account-picker", frontend_host)),
            sign_in_page_url: Some(format!("{}/sign-in", frontend_host)),
            sign_up_page_url: Some(format!("{}/sign-up", frontend_host)),
            ..DeploymentDisplaySettings::default()
        }
    }

    fn create_org_settings(&self, deployment_id: i64) -> DeploymentOrgSettings {
        DeploymentOrgSettings {
            deployment_id,
            ..DeploymentOrgSettings::default()
        }
    }
}

impl Command for CreateProjectCommand {
    type Output = ProjectWithDeployments;

    async fn execute(self, app_state: &AppState) -> Result<Self::Output, AppError> {
        let mut tx = app_state.db_pool.begin().await?;

        let project_id = app_state.sf.next_id()? as i64;

        let image_url: String;

        if self.has_logo {
            image_url = UploadToCdnCommand::new(
                format!("projects/{}/logo.png", project_id),
                self.logo.clone(),
            )
            .execute(app_state)
            .await?;
        } else {
            image_url = "".to_string();
        }

        let project_row = sqlx::query!(
            r#"
            INSERT INTO projects (id, name, image_url, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, created_at, updated_at, deleted_at, name, image_url
            "#,
            project_id,
            self.name,
            image_url,
            chrono::Utc::now(),
            chrono::Utc::now(),
        )
        .fetch_one(&mut *tx)
        .await?;

        let secret_key = Self::generate_key("sk");
        let host = format!("{}.backend-api.services", project_id);
        let mut publishable_key = String::from("pk_test_");

        let base64_host = BASE64_STANDARD.encode(format!("https://{}", host));
        publishable_key.push_str(&base64_host);

        let deployment_row = sqlx::query!(
            r#"
            INSERT INTO deployments (
                id,
                project_id, 
                mode, 
                host, 
                publishable_key, 
                secret, 
                maintenance_mode
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, created_at, updated_at, deleted_at, 
                     maintenance_mode, host, publishable_key, secret, 
                     project_id, mode
            "#,
            app_state.sf.next_id()? as i64,
            project_row.id,
            "staging",
            host,
            publishable_key,
            secret_key,
            false,
        )
        .fetch_one(&mut *tx)
        .await?;

        let auth_settings = self.create_auth_settings(deployment_row.id);

        sqlx::query!(
            r#"
            INSERT INTO deployment_auth_settings (
                id,
                deployment_id,
                email_address,
                phone_number,
                username,
                first_factor,
                alternate_first_factors,
                password_policy,
                first_name,
                last_name,
                password,
                backup_code,
                web3_wallet,
                auth_factors_enabled,
                verification_policy,
                second_factor_policy,
                second_factor,
                passkey,
                magic_link,
                created_at,
                updated_at
            )
            VALUES (
                $1, 
                $2, 
                $3, 
                $4,
                $5,
                $6,
                $7,
                $8,
                $9,
                $10,
                $11,
                $12,
                $13,
                $14,
                $15,
                $16,
                $17,
                $18,
                $19,
                $20,
                $21
            )
            "#,
            app_state.sf.next_id()? as i64,
            deployment_row.id,
            serde_json::to_value(&auth_settings.email_address)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            serde_json::to_value(&auth_settings.phone_number)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            serde_json::to_value(&auth_settings.username)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            auth_settings.first_factor.to_string(),
            &auth_settings
                .alternate_first_factors
                .unwrap_or_default()
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<String>>(),
            serde_json::to_value(&auth_settings.password)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            serde_json::to_value(&auth_settings.first_name)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            serde_json::to_value(&auth_settings.last_name)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            serde_json::to_value(&auth_settings.password)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            serde_json::to_value(&auth_settings.backup_code)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            serde_json::to_value(&auth_settings.web3_wallet)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            serde_json::to_value(&auth_settings.auth_factors_enabled)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            serde_json::to_value(&auth_settings.verification_policy)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            auth_settings
                .second_factor_policy
                .unwrap_or(SecondFactorPolicy::None)
                .to_string(),
            auth_settings.second_factor.map(|f| f.to_string()),
            serde_json::to_value(&auth_settings.passkey)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            serde_json::to_value(&auth_settings.magic_link)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            chrono::Utc::now(),
            chrono::Utc::now(),
        )
        .execute(&mut *tx)
        .await?;

        let display_settings = self.create_display_settings(
            deployment_row.id,
            format!("https://{}.watch.tech", project_id),
        );

        sqlx::query!(
            r#"
            INSERT INTO deployment_display_settings (
                id,
                deployment_id,
                app_name,
                primary_color,
                button_config,
                input_config,
                tos_page_url,
                sign_in_page_url,
                sign_up_page_url,
                after_sign_out_one_page_url,
                after_sign_out_all_page_url,
                favicon_image_url,
                logo_image_url,
                privacy_policy_url,
                signup_terms_statement,
                signup_terms_statement_shown,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            "#,
            app_state.sf.next_id()? as i64,
            display_settings.deployment_id,
            display_settings.app_name,
            display_settings.primary_color,
            serde_json::to_value(&display_settings.button_config)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            serde_json::to_value(&display_settings.input_config)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            display_settings.tos_page_url,
            display_settings.sign_in_page_url,
            display_settings.sign_up_page_url,
            display_settings.after_sign_out_one_page_url,
            display_settings.after_sign_out_all_page_url,
            display_settings.favicon_image_url,
            display_settings.logo_image_url,
            display_settings.privacy_policy_url,
            display_settings.signup_terms_statement,
            display_settings.signup_terms_statement_shown,
            chrono::Utc::now(),
            chrono::Utc::now(),
        )
        .execute(&mut *tx)
        .await?;

        let org_settings = self.create_org_settings(deployment_row.id);

        sqlx::query!(
            r#"
            INSERT INTO deployment_org_settings (
                id,
                deployment_id,
                enabled,
                ip_allowlist_enabled,
                max_allowed_members,
                allow_deletion,
                custom_role_enabled,
                default_role,
                created_at,
                updated_at
            )
            VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                $7,
                $8,
                $9,
                $10
            )
            "#,
            app_state.sf.next_id()? as i64,
            org_settings.deployment_id,
            org_settings.enabled,
            org_settings.ip_allowlist_enabled,
            org_settings.max_allowed_members,
            org_settings.allow_deletion,
            org_settings.custom_role_enabled,
            org_settings.default_role,
            chrono::Utc::now(),
            chrono::Utc::now(),
        )
        .execute(&mut *tx)
        .await?;

        let social_providers = [
            "google",
            "apple",
            "facebook",
            "github",
            "microsoft",
            "discord",
            "linkedin",
        ];

        let empty_credentials = json!({});

        for provider in social_providers.iter() {
            if self.auth_methods.contains(&provider.to_string()) {
                sqlx::query!(
                    r#"
                    INSERT INTO deployment_social_connections (
                        id,
                        deployment_id,
                        provider,
                        enabled,
                        credentials,
                        created_at,
                        updated_at
                    )
                    VALUES (
                        $1,
                        $2,
                        $3,
                        true,
                        $4,
                        $5,
                        $6
                    )
                    "#,
                    app_state.sf.next_id()? as i64,
                    deployment_row.id,
                    provider,
                    empty_credentials,
                    chrono::Utc::now(),
                    chrono::Utc::now(),
                )
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;

        let deployment = Deployment {
            id: deployment_row.id,
            created_at: deployment_row.created_at.unwrap_or_default(),
            updated_at: deployment_row.updated_at.unwrap_or_default(),
            deleted_at: deployment_row.deleted_at,
            maintenance_mode: deployment_row.maintenance_mode.unwrap_or_default(),
            host: deployment_row.host.unwrap_or_default(),
            publishable_key: deployment_row.publishable_key.unwrap_or_default(),
            secret: deployment_row.secret.unwrap_or_default(),
            project_id: deployment_row.project_id.unwrap_or_default(),
            mode: DeploymentMode::from(
                deployment_row
                    .mode
                    .unwrap_or_else(|| "production".to_string()),
            ),
        };

        Ok(ProjectWithDeployments {
            id: project_row.id,
            image_url: project_row.image_url,
            created_at: project_row.created_at.unwrap_or_default(),
            updated_at: project_row.updated_at.unwrap_or_default(),
            deleted_at: project_row.deleted_at,
            name: project_row.name.unwrap_or_default(),
            deployments: vec![deployment],
        })
    }
}

pub struct DeleteProjectCommand {
    id: i64,
    created_by: i64,
}

impl DeleteProjectCommand {
    pub fn new(id: i64, created_by: i64) -> Self {
        Self { id, created_by }
    }
}

impl Command for DeleteProjectCommand {
    type Output = ();

    async fn execute(self, app_state: &AppState) -> Result<Self::Output, AppError> {
        let mut tx = app_state.db_pool.begin().await?;

        let deployments = sqlx::query!(
            r#"
            SELECT id FROM deployments 
            WHERE project_id = $1 AND deleted_at IS NULL
            "#,
            self.id
        )
        .fetch_all(&mut *tx)
        .await?;

        for deployment in &deployments {
            sqlx::query!(
                r#"
                UPDATE deployment_social_connections
                SET deleted_at = $1
                WHERE deployment_id = $2 AND deleted_at IS NULL
                "#,
                chrono::Utc::now(),
                deployment.id
            )
            .execute(&mut *tx)
            .await?;
        }

        for deployment in &deployments {
            sqlx::query!(
                r#"
                UPDATE deployment_auth_settings
                SET deleted_at = $1
                WHERE deployment_id = $2 AND deleted_at IS NULL
                "#,
                chrono::Utc::now(),
                deployment.id
            )
            .execute(&mut *tx)
            .await?;
        }

        for deployment in &deployments {
            sqlx::query!(
                r#"
                UPDATE deployment_display_settings
                SET deleted_at = $1
                WHERE deployment_id = $2 AND deleted_at IS NULL
                "#,
                chrono::Utc::now(),
                deployment.id
            )
            .execute(&mut *tx)
            .await?;
        }

        for deployment in &deployments {
            sqlx::query!(
                r#"
                UPDATE deployment_org_settings
                SET deleted_at = $1
                WHERE deployment_id = $2 AND deleted_at IS NULL
                "#,
                chrono::Utc::now(),
                deployment.id
            )
            .execute(&mut *tx)
            .await?;
        }

        sqlx::query!(
            r#"
            UPDATE deployments
            SET deleted_at = $1
            WHERE project_id = $2 AND deleted_at IS NULL
            "#,
            chrono::Utc::now(),
            self.id
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
            UPDATE projects
            SET deleted_at = $1
            WHERE id = $2 AND deleted_at IS NULL
            "#,
            chrono::Utc::now(),
            self.id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }
}
