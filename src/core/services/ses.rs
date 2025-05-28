use crate::application::AppError;
use crate::core::models::{DnsRecord, EmailVerificationRecords};
use aws_sdk_sesv2::{Client as SesClient, types::MailFromDomainStatus};

#[derive(Clone)]
pub struct SesService {
    client: SesClient,
}

impl SesService {
    pub fn new(client: SesClient) -> Self {
        Self { client }
    }

    pub async fn create_email_identity(&self, domain: &str) -> Result<(), AppError> {
        // Create the email identity for the domain
        self.client
            .create_email_identity()
            .email_identity(domain)
            .send()
            .await
            .map_err(|e| {
                AppError::External(format!("Failed to create SES email identity: {}", e))
            })?;

        Ok(())
    }

    pub async fn set_mail_from_domain(
        &self,
        domain: &str,
        mail_from_domain: &str,
    ) -> Result<(), AppError> {
        // Set the MAIL FROM domain for the identity
        self.client
            .put_email_identity_mail_from_attributes()
            .email_identity(domain)
            .mail_from_domain(mail_from_domain)
            .behavior_on_mx_failure(aws_sdk_sesv2::types::BehaviorOnMxFailure::UseDefaultValue)
            .send()
            .await
            .map_err(|e| {
                AppError::External(format!("Failed to set SES MAIL FROM domain: {}", e))
            })?;

        Ok(())
    }

    pub async fn delete_email_identity(&self, domain: &str) -> Result<(), AppError> {
        self.client
            .delete_email_identity()
            .email_identity(domain)
            .send()
            .await
            .map_err(|e| {
                AppError::External(format!("Failed to delete SES email identity: {}", e))
            })?;

        Ok(())
    }

    pub async fn get_email_identity_status(
        &self,
        domain: &str,
    ) -> Result<Option<String>, AppError> {
        let response = self
            .client
            .get_email_identity()
            .email_identity(domain)
            .send()
            .await;

        match response {
            Ok(output) => {
                // Check if the identity is verified
                if output.verified_for_sending_status() {
                    Ok(Some("verified".to_string()))
                } else {
                    Ok(Some("pending".to_string()))
                }
            }
            Err(e) => {
                // If the identity doesn't exist, return None
                if e.to_string().contains("NotFoundException") {
                    Ok(None)
                } else {
                    Err(AppError::External(format!(
                        "Failed to get SES email identity status: {}",
                        e
                    )))
                }
            }
        }
    }

    pub async fn get_mail_from_domain_status(
        &self,
        domain: &str,
    ) -> Result<Option<MailFromDomainStatus>, AppError> {
        let response = self
            .client
            .get_email_identity()
            .email_identity(domain)
            .send()
            .await;

        match response {
            Ok(output) => {
                if let Some(attrs) = output.mail_from_attributes() {
                    Ok(Some(attrs.mail_from_domain_status().clone()))
                } else {
                    Ok(None)
                }
            }
            Err(e) => {
                if e.to_string().contains("NotFoundException") {
                    Ok(None)
                } else {
                    Err(AppError::External(format!(
                        "Failed to get SES MAIL FROM domain status: {}",
                        e
                    )))
                }
            }
        }
    }

    pub async fn create_domain_identity_with_mail_from(
        &self,
        domain: &str,
    ) -> Result<(), AppError> {
        // Create the email identity
        self.create_email_identity(domain).await?;

        // Set up MAIL FROM domain (using mail.{domain} as the MAIL FROM subdomain)
        let mail_from_domain = format!("mail.{}", domain);
        self.set_mail_from_domain(domain, &mail_from_domain).await?;

        Ok(())
    }

    /// Get DNS records required for SES email verification from AWS SES API
    pub async fn get_email_verification_records(
        &self,
        domain: &str,
    ) -> Result<EmailVerificationRecords, AppError> {
        let mut records = EmailVerificationRecords::default();

        // Try to get existing identity to see if it has verification tokens
        let identity_response = self
            .client
            .get_email_identity()
            .email_identity(domain)
            .send()
            .await;

        match identity_response {
            Ok(identity) => {
                // If identity exists, get verification tokens
                if let Some(verification_info) = identity.verification_info() {
                    if let Some(verification_token) = verification_info.soa_record() {
                        records.ses_verification.push(DnsRecord {
                            name: format!("_amazonses.{}", domain),
                            record_type: "TXT".to_string(),
                            value: format!("{:?}", verification_token), // Use debug format for now
                            ttl: Some(1800),
                            verified: identity.verified_for_sending_status(),
                            verification_attempted_at: None,
                            last_verified_at: None,
                        });
                    }
                }

                // Get DKIM attributes if available
                if let Some(dkim_attributes) = identity.dkim_attributes() {
                    let tokens = dkim_attributes.tokens();
                    for (_i, token) in tokens.iter().enumerate() {
                        records.dkim_records.push(DnsRecord {
                            name: format!("{}._domainkey.{}", token, domain),
                            record_type: "CNAME".to_string(),
                            value: format!("{}.dkim.amazonses.com", token),
                            ttl: Some(1800),
                            verified: dkim_attributes.status()
                                == Some(&aws_sdk_sesv2::types::DkimStatus::Success),
                            verification_attempted_at: None,
                            last_verified_at: None,
                        });
                    }
                }

                // Get MAIL FROM attributes if available
                if let Some(mail_from_attrs) = identity.mail_from_attributes() {
                    let mail_from_domain = mail_from_attrs.mail_from_domain();
                    let verified = mail_from_attrs.mail_from_domain_status()
                        == &aws_sdk_sesv2::types::MailFromDomainStatus::Success;

                    records.mail_from_verification.push(DnsRecord {
                        name: mail_from_domain.to_string(),
                        record_type: "MX".to_string(),
                        value: "10 feedback-smtp.us-east-1.amazonses.com".to_string(),
                        ttl: Some(1800),
                        verified,
                        verification_attempted_at: None,
                        last_verified_at: None,
                    });

                    records.mail_from_verification.push(DnsRecord {
                        name: mail_from_domain.to_string(),
                        record_type: "TXT".to_string(),
                        value: "v=spf1 include:amazonses.com ~all".to_string(),
                        ttl: Some(1800),
                        verified,
                        verification_attempted_at: None,
                        last_verified_at: None,
                    });
                }
            }
            Err(_) => {
                // Identity doesn't exist, create it and get verification tokens
                self.create_domain_identity_with_mail_from(domain).await?;

                // After creating, try to get the verification info again
                let new_identity_response = self
                    .client
                    .get_email_identity()
                    .email_identity(domain)
                    .send()
                    .await
                    .map_err(|e| {
                        AppError::External(format!(
                            "Failed to get newly created SES identity: {}",
                            e
                        ))
                    })?;

                // Add verification records based on the newly created identity
                if let Some(verification_info) = new_identity_response.verification_info() {
                    if let Some(verification_token) = verification_info.soa_record() {
                        records.ses_verification.push(DnsRecord {
                            name: format!("_amazonses.{}", domain),
                            record_type: "TXT".to_string(),
                            value: format!("{:?}", verification_token), // Use debug format for now
                            ttl: Some(1800),
                            verified: false,
                            verification_attempted_at: None,
                            last_verified_at: None,
                        });
                    }
                }

                // Add default DKIM records (will be populated after DKIM is enabled)
                for i in 1..=3 {
                    records.dkim_records.push(DnsRecord {
                        name: format!("dkim{}._domainkey.{}", i, domain),
                        record_type: "CNAME".to_string(),
                        value: format!("dkim{}.{}.dkim.amazonses.com", i, domain),
                        ttl: Some(1800),
                        verified: false,
                        verification_attempted_at: None,
                        last_verified_at: None,
                    });
                }

                // Add MAIL FROM records
                let mail_from_domain = format!("mail.{}", domain);
                records.mail_from_verification.push(DnsRecord {
                    name: mail_from_domain.clone(),
                    record_type: "MX".to_string(),
                    value: "10 feedback-smtp.us-east-1.amazonses.com".to_string(),
                    ttl: Some(1800),
                    verified: false,
                    verification_attempted_at: None,
                    last_verified_at: None,
                });

                records.mail_from_verification.push(DnsRecord {
                    name: mail_from_domain,
                    record_type: "TXT".to_string(),
                    value: "v=spf1 include:amazonses.com ~all".to_string(),
                    ttl: Some(1800),
                    verified: false,
                    verification_attempted_at: None,
                    last_verified_at: None,
                });
            }
        }

        Ok(records)
    }

    /// Check SES verification status and update records using SES API
    pub async fn verify_email_records(
        &self,
        records: &mut EmailVerificationRecords,
        domain: &str,
    ) -> Result<(), AppError> {
        // Try to get email identity status
        let email_identity_result = self
            .client
            .get_email_identity()
            .email_identity(domain)
            .send()
            .await;

        match email_identity_result {
            Ok(identity) => {
                let domain_verified = identity.verified_for_sending_status();
                let now = chrono::Utc::now();

                // Update SES verification records
                for record in &mut records.ses_verification {
                    record.verified = domain_verified;
                    record.verification_attempted_at = Some(now);
                    if domain_verified {
                        record.last_verified_at = Some(now);
                    }
                }

                // Check DKIM status
                let dkim_verified = if let Some(dkim_attrs) = identity.dkim_attributes() {
                    dkim_attrs.status() == Some(&aws_sdk_sesv2::types::DkimStatus::Success)
                } else {
                    false
                };

                // Update DKIM records
                for record in &mut records.dkim_records {
                    record.verified = dkim_verified;
                    record.verification_attempted_at = Some(now);
                    if dkim_verified {
                        record.last_verified_at = Some(now);
                    }
                }

                // Check MAIL FROM status
                let mail_from_verified =
                    if let Some(mail_from_attrs) = identity.mail_from_attributes() {
                        mail_from_attrs.mail_from_domain_status()
                            == &aws_sdk_sesv2::types::MailFromDomainStatus::Success
                    } else {
                        false
                    };

                // Update MAIL FROM records
                for record in &mut records.mail_from_verification {
                    record.verified = mail_from_verified;
                    record.verification_attempted_at = Some(now);
                    if mail_from_verified {
                        record.last_verified_at = Some(now);
                    }
                }
            }
            Err(e) => {
                // Domain not found or error - mark all as unverified but attempted
                let now = chrono::Utc::now();

                for record in &mut records.ses_verification {
                    record.verified = false;
                    record.verification_attempted_at = Some(now);
                }

                for record in &mut records.dkim_records {
                    record.verified = false;
                    record.verification_attempted_at = Some(now);
                }

                for record in &mut records.mail_from_verification {
                    record.verified = false;
                    record.verification_attempted_at = Some(now);
                }

                tracing::warn!("Failed to verify SES identity for domain {}: {}", domain, e);
            }
        }

        Ok(())
    }
}
