use crate::{
    application::{AppError, AppState},
    core::models::{
        AuthFactorsEnabled, Deployment, DeploymentAuthSettings, DeploymentB2bSettings,
        DeploymentB2bSettingsWithRoles, DeploymentDisplaySettings, DeploymentMode,
        DeploymentOrganizationRole, DeploymentRestrictions, DeploymentWorkspaceRole, EmailSettings,
        FirstFactor, IndividualAuthSettings, OauthCredentials, PasswordSettings, PhoneSettings,
        ProjectWithDeployments, SecondFactorPolicy, SocialConnectionProvider, UsernameSettings,
        VerificationPolicy,
    },
};
use base64::{Engine, prelude::BASE64_STANDARD};
use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

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

    fn create_b2b_settings(&self, deployment_id: i64) -> DeploymentB2bSettingsWithRoles {
        DeploymentB2bSettingsWithRoles {
            settings: DeploymentB2bSettings {
                deployment_id,
                ..DeploymentB2bSettings::default()
            },
            default_workspace_creator_role: DeploymentWorkspaceRole::admin(),
            default_workspace_member_role: DeploymentWorkspaceRole::member(),
            default_org_creator_role: DeploymentOrganizationRole::admin(),
            default_org_member_role: DeploymentOrganizationRole::member(),
        }
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
            first_factor,
            first_name: first_name_settings,
            last_name: last_name_settings,
            password: password_settings,
            auth_factors_enabled,
            verification_policy,
            second_factor_policy: SecondFactorPolicy::None,
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
            after_sign_out_all_page_url: format!("{}/sign-in", frontend_host),
            after_sign_out_one_page_url: format!("{}/account-picker", frontend_host),
            sign_in_page_url: format!("{}/sign-in", frontend_host),
            sign_up_page_url: format!("{}/sign-up", frontend_host),
            ..DeploymentDisplaySettings::default()
        }
    }

    fn create_restrictions(&self, deployment_id: i64) -> DeploymentRestrictions {
        DeploymentRestrictions {
            deployment_id,
            allowlist_enabled: false,
            blocklist_enabled: false,
            block_subaddresses: false,
            block_disposable_emails: false,
            block_voip_numbers: false,
            country_restrictions: Default::default(),
            banned_keywords: Default::default(),
            allowlisted_resources: Default::default(),
            blocklisted_resources: Default::default(),
            sign_up_mode: Default::default(),
            ..Default::default()
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
        let backend_host = format!("{}.backend-api.services", project_id);
        let frontend_host = format!("{}.watch.tech", project_id);
        let mut publishable_key = String::from("pk_test_");

        let base64_backend_host = BASE64_STANDARD.encode(format!("https://{}", backend_host));
        publishable_key.push_str(&base64_backend_host);

        let deployment_row = sqlx::query!(
            r#"
            INSERT INTO deployments (
                id,
                project_id, 
                mode, 
                backend_host, 
                frontend_host, 
                publishable_key, 
                secret, 
                maintenance_mode,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, created_at, updated_at, deleted_at, 
                     maintenance_mode, backend_host, frontend_host, publishable_key, secret, 
                     project_id, mode
            "#,
            app_state.sf.next_id()? as i64,
            project_row.id,
            "staging",
            backend_host,
            frontend_host,
            publishable_key,
            secret_key,
            false,
            chrono::Utc::now(),
            chrono::Utc::now(),
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
                first_name,
                last_name,
                password,
                auth_factors_enabled,
                verification_policy,
                second_factor_policy,
                passkey,
                magic_link,
                multi_session_support,
                session_token_lifetime,
                session_validity_period,
                session_inactive_timeout,
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
                $20
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
            serde_json::to_value(&auth_settings.first_name)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            serde_json::to_value(&auth_settings.last_name)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            serde_json::to_value(&auth_settings.password)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            serde_json::to_value(&auth_settings.auth_factors_enabled)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            serde_json::to_value(&auth_settings.verification_policy)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            auth_settings.second_factor_policy.to_string(),
            serde_json::to_value(&auth_settings.passkey)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            serde_json::to_value(&auth_settings.magic_link)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            serde_json::to_value(&auth_settings.multi_session_support)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            auth_settings.session_token_lifetime,
            auth_settings.session_validity_period,
            auth_settings.session_inactive_timeout,
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

        let restrictions = self.create_restrictions(deployment_row.id);

        sqlx::query!(
            r#"
            INSERT INTO deployment_restrictions (
                id,
                deployment_id,
                allowlist_enabled,
                blocklist_enabled,
                block_subaddresses,
                block_disposable_emails,
                block_voip_numbers,
                country_restrictions,
                banned_keywords,
                allowlisted_resources,
                blocklisted_resources,
                sign_up_mode,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
            app_state.sf.next_id()? as i64,
            restrictions.deployment_id,
            restrictions.allowlist_enabled,
            restrictions.blocklist_enabled,
            restrictions.block_subaddresses,
            restrictions.block_disposable_emails,
            restrictions.block_voip_numbers,
            serde_json::to_value(&restrictions.country_restrictions)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            &restrictions.banned_keywords,
            &restrictions.allowlisted_resources,
            &restrictions.blocklisted_resources,
            restrictions.sign_up_mode.to_string(),
            chrono::Utc::now(),
            chrono::Utc::now(),
        )
        .execute(&mut *tx)
        .await?;

        let mut b2b_settings = self.create_b2b_settings(deployment_row.id);

        let default_workspace_creator_role = sqlx::query!(
            r#"
            INSERT INTO deployment_workspace_roles (
                id,
                deployment_id,
                name,
                permissions,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6)

            RETURNING id
            "#,
            app_state.sf.next_id()? as i64,
            deployment_row.id,
            b2b_settings.default_workspace_creator_role.name,
            &b2b_settings.default_workspace_creator_role.permissions,
            chrono::Utc::now(),
            chrono::Utc::now(),
        )
        .fetch_one(&mut *tx)
        .await?;

        let default_workspace_member_role = sqlx::query!(
            r#"
            INSERT INTO deployment_workspace_roles (
                id,
                deployment_id,
                name,
                permissions,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6)

            RETURNING id
            "#,
            app_state.sf.next_id()? as i64,
            deployment_row.id,
            b2b_settings.default_workspace_member_role.name,
            &b2b_settings.default_workspace_member_role.permissions,
            chrono::Utc::now(),
            chrono::Utc::now(),
        )
        .fetch_one(&mut *tx)
        .await?;

        let default_org_creator_role = sqlx::query!(
            r#"
            INSERT INTO deployment_organization_roles (
                id,
                deployment_id,
                name,
                permissions,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6)

            RETURNING id
            "#,
            app_state.sf.next_id()? as i64,
            deployment_row.id,
            b2b_settings.default_org_creator_role.name,
            &b2b_settings.default_org_creator_role.permissions,
            chrono::Utc::now(),
            chrono::Utc::now(),
        )
        .fetch_one(&mut *tx)
        .await?;

        let default_org_member_role = sqlx::query!(
            r#"
            INSERT INTO deployment_organization_roles (
                id,
                deployment_id,
                name,
                permissions,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6)

            RETURNING id
            "#,
            app_state.sf.next_id()? as i64,
            deployment_row.id,
            b2b_settings.default_org_member_role.name,
            &b2b_settings.default_org_member_role.permissions,
            chrono::Utc::now(),
            chrono::Utc::now(),
        )
        .fetch_one(&mut *tx)
        .await?;

        b2b_settings.default_workspace_creator_role.id = default_workspace_creator_role.id;
        b2b_settings.default_workspace_member_role.id = default_workspace_member_role.id;
        b2b_settings.default_org_creator_role.id = default_org_creator_role.id;
        b2b_settings.default_org_member_role.id = default_org_member_role.id;

        sqlx::query!(
            r#"
            INSERT INTO deployment_b2b_settings (
                id,
                deployment_id,
                organizations_enabled,
                workspaces_enabled,
                ip_allowlist_per_org_enabled,
                max_allowed_org_members,
                max_allowed_workspace_members,
                allow_org_deletion,
                allow_workspace_deletion,
                custom_org_role_enabled,
                custom_workspace_role_enabled,
                default_workspace_creator_role_id,
                default_workspace_member_role_id,
                default_org_creator_role_id,
                default_org_member_role_id,
                limit_org_creation_per_user,
                limit_workspace_creation_per_org,
                org_creation_per_user_count,
                workspaces_per_org_count,
                allow_users_to_create_orgs,
                max_orgs_per_user,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23)
            "#,
            app_state.sf.next_id()? as i64,
            deployment_row.id,
            b2b_settings.settings.organizations_enabled,
            b2b_settings.settings.workspaces_enabled,
            b2b_settings.settings.ip_allowlist_per_org_enabled,
            b2b_settings.settings.max_allowed_org_members,
            b2b_settings.settings.max_allowed_workspace_members,
            b2b_settings.settings.allow_org_deletion,
            b2b_settings.settings.allow_workspace_deletion,
            b2b_settings.settings.custom_org_role_enabled,
            b2b_settings.settings.custom_workspace_role_enabled,
            b2b_settings.default_workspace_creator_role.id,
            b2b_settings.default_workspace_member_role.id,
            b2b_settings.default_org_creator_role.id,
            b2b_settings.default_org_member_role.id,
            b2b_settings.settings.limit_org_creation_per_user,
            b2b_settings.settings.limit_workspace_creation_per_org,
            b2b_settings.settings.org_creation_per_user_count,
            b2b_settings.settings.workspaces_per_org_count,
            b2b_settings.settings.allow_users_to_create_orgs,
            b2b_settings.settings.max_orgs_per_user,
            chrono::Utc::now(),
            chrono::Utc::now(),
        )
        .execute(&mut *tx)
        .await?;

        let social_providers = [
            "google_oauth",
            "apple_oauth",
            "facebook_oauth",
            "github_oauth",
            "microsoft_oauth",
            "discord_oauth",
            "linkedin_oauth",
        ];

        let empty_credentials = serde_json::to_value(OauthCredentials::default())
            .map_err(|e| AppError::Serialization(e.to_string()))?;

        for provider in social_providers.iter() {
            if self.auth_methods.contains(&provider.to_string())
                && SocialConnectionProvider::from_str(*provider).is_ok()
            {
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
            created_at: deployment_row.created_at,
            updated_at: deployment_row.updated_at,
            deleted_at: deployment_row.deleted_at,
            maintenance_mode: deployment_row.maintenance_mode,
            backend_host: deployment_row.backend_host,
            frontend_host: deployment_row.frontend_host,
            publishable_key: deployment_row.publishable_key,
            secret: deployment_row.secret,
            project_id: deployment_row.project_id,
            mode: DeploymentMode::from(deployment_row.mode),
        };

        Ok(ProjectWithDeployments {
            id: project_row.id,
            image_url: project_row.image_url,
            created_at: project_row.created_at,
            updated_at: project_row.updated_at,
            deleted_at: project_row.deleted_at,
            name: project_row.name,
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
                UPDATE deployment_b2b_settings
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
