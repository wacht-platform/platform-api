use crate::{
    application::{AppError, AppState},
    core::models::{
        AuthFactorsEnabled, DarkModeSettings, Deployment, DeploymentAuthSettings,
        DeploymentB2bSettings, DeploymentB2bSettingsWithRoles, DeploymentDisplaySettings,
        DeploymentEmailTemplate, DeploymentKeyPair, DeploymentMode, DeploymentOrganizationRole,
        DeploymentRestrictions, DeploymentSmsTemplate, DeploymentWorkspaceRole, EmailSettings,
        FirstFactor, IndividualAuthSettings, LightModeSettings, OauthCredentials, PasswordSettings,
        PhoneSettings, ProjectWithDeployments, SecondFactorPolicy, SocialConnectionProvider,
        UsernameSettings, VerificationPolicy,
    },
    utils::name::generate_random_name,
};
use base64::{Engine, prelude::BASE64_STANDARD};
use redis::AsyncCommands;
use std::str::FromStr;

use super::{Command, UploadToCdnCommand};

pub struct CreateProjectWithStagingDeploymentCommand {
    name: String,
    logo: Vec<u8>,
    has_logo: bool,
    auth_methods: Vec<String>,
}

impl CreateProjectWithStagingDeploymentCommand {
    pub fn new(name: String, logo: Vec<u8>, auth_methods: Vec<String>) -> Self {
        let has_logo = !logo.is_empty();
        Self {
            name,
            logo,
            has_logo,
            auth_methods,
        }
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

    fn create_key_pair(&self, deployment_id: i64) -> Result<DeploymentKeyPair, AppError> {
        let pair = rcgen::KeyPair::generate().map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(DeploymentKeyPair {
            id: 0,
            deployment_id,
            public_key: pair.public_key_pem(),
            private_key: pair.serialize_pem(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
        })
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
            dark_mode_settings: DarkModeSettings::default(),
            light_mode_settings: LightModeSettings::default(),
            organization_profile_url: format!("{}/organization", frontend_host),
            create_organization_url: format!("{}/create-organization", frontend_host),
            user_profile_url: format!("{}/me", frontend_host),
            use_initials_for_organization_profile_image: true,
            use_initials_for_user_profile_image: true,
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

    fn create_sms_templates(&self, deployment_id: i64) -> DeploymentSmsTemplate {
        DeploymentSmsTemplate {
            deployment_id,
            ..Default::default()
        }
    }

    fn create_email_templates(&self, deployment_id: i64) -> DeploymentEmailTemplate {
        DeploymentEmailTemplate {
            deployment_id,
            ..Default::default()
        }
    }
}

impl Command for CreateProjectWithStagingDeploymentCommand {
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

        let random_name = generate_random_name();
        let count: i64 = app_state
            .redis_client
            .get_multiplexed_tokio_connection()
            .await?
            .incr(format!("project_count:{}", random_name), 1)
            .await?;

        let hostname = format!("{}-{}", random_name, count);

        let backend_host = format!("{}.backend-api.services", hostname);
        let frontend_host = format!("{}.wacht.tech", hostname);
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
                maintenance_mode,
                mail_from_host,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, created_at, updated_at, deleted_at, 
                     maintenance_mode, backend_host, frontend_host, publishable_key, project_id, mode, mail_from_host
            "#,
            app_state.sf.next_id()? as i64,
            project_row.id,
            "staging",
            backend_host,
            frontend_host,
            publishable_key,
            false,
            "dev.wacht.services",
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
            format!("https://{}.wacht.tech", hostname),
        );

        sqlx::query!(
            r#"
            INSERT INTO deployment_display_settings (
                id,
                deployment_id,
                app_name,
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
                light_mode_settings,
                dark_mode_settings,
                after_logo_click_url,
                organization_profile_url,
                create_organization_url,
                default_user_profile_image_url,
                default_organization_profile_image_url,
                use_initials_for_user_profile_image,
                use_initials_for_organization_profile_image,
                after_signup_redirect_url,
                after_signin_redirect_url,
                user_profile_url,
                after_create_organization_redirect_url,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28)
            "#,
            app_state.sf.next_id()? as i64,
            display_settings.deployment_id,
            display_settings.app_name,
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
            serde_json::to_value(&display_settings.light_mode_settings)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            serde_json::to_value(&display_settings.dark_mode_settings)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            display_settings.after_logo_click_url,
            display_settings.organization_profile_url,
            display_settings.create_organization_url,
            display_settings.default_user_profile_image_url,
            display_settings.default_organization_profile_image_url,
            display_settings.use_initials_for_user_profile_image,
            display_settings.use_initials_for_organization_profile_image,
            display_settings.after_signup_redirect_url,
            display_settings.after_signin_redirect_url,
            display_settings.user_profile_url,
            display_settings.after_create_organization_redirect_url,
            chrono::Utc::now(),
            chrono::Utc::now(),
        )
        .execute(&mut *tx)
        .await.unwrap();

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

        let sms_templates = self.create_sms_templates(deployment_row.id);

        sqlx::query!(
            r#"
            INSERT INTO deployment_sms_templates (
                id,
                deployment_id,
                reset_password_code_template,
                verification_code_template,
                password_change_template,
                password_remove_template,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            app_state.sf.next_id()? as i64,
            sms_templates.deployment_id,
            sms_templates.reset_password_code_template,
            sms_templates.verification_code_template,
            sms_templates.password_change_template,
            sms_templates.password_remove_template,
            chrono::Utc::now(),
            chrono::Utc::now(),
        )
        .execute(&mut *tx)
        .await?;

        let key_pair = self.create_key_pair(deployment_row.id)?;

        sqlx::query!(
            r#"
            INSERT INTO deployment_key_pairs (
                id,
                deployment_id,
                public_key,
                private_key,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            app_state.sf.next_id()? as i64,
            deployment_row.id,
            key_pair.public_key,
            key_pair.private_key,
            chrono::Utc::now(),
            chrono::Utc::now(),
        )
        .execute(&mut *tx)
        .await?;

        let email_templates = self.create_email_templates(deployment_row.id);

        sqlx::query!(
            r#"
            INSERT INTO deployment_email_templates (
                id,
                deployment_id,
                organization_invite_template,
                verification_code_template,
                reset_password_code_template,
                primary_email_change_template,
                password_change_template,
                password_remove_template,
                sign_in_from_new_device_template,
                magic_link_template,
                waitlist_signup_template,
                waitlist_invite_template,
                workspace_invite_template,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
            app_state.sf.next_id()? as i64,
            email_templates.deployment_id,
            serde_json::to_value(&email_templates.organization_invite_template)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            serde_json::to_value(&email_templates.verification_code_template)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            serde_json::to_value(&email_templates.reset_password_code_template)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            serde_json::to_value(&email_templates.primary_email_change_template)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            serde_json::to_value(&email_templates.password_change_template)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            serde_json::to_value(&email_templates.password_remove_template)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            serde_json::to_value(&email_templates.sign_in_from_new_device_template)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            serde_json::to_value(&email_templates.magic_link_template)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            serde_json::to_value(&email_templates.waitlist_signup_template)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            serde_json::to_value(&email_templates.waitlist_invite_template)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            serde_json::to_value(&email_templates.workspace_invite_template)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            chrono::Utc::now(),
            chrono::Utc::now(),
        )
        .execute(&mut *tx)
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
            project_id: deployment_row.project_id,
            mode: DeploymentMode::from(deployment_row.mode),
            mail_from_host: deployment_row.mail_from_host,
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
