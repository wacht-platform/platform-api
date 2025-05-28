use crate::application::AppError;
use crate::core::models::{DnsRecord, DomainVerificationRecords, EmailVerificationRecords};
use chrono::Utc;
use serde::Deserialize;

#[derive(Clone)]
pub struct DnsVerificationService {}

#[derive(Debug, Deserialize)]
struct DnsResponse {
    #[serde(rename = "Answer")]
    answer: Option<Vec<DnsAnswer>>,
}

#[derive(Debug, Deserialize)]
struct DnsAnswer {
    name: String,
    #[serde(rename = "type")]
    record_type: u16,
    #[serde(rename = "TTL")]
    ttl: u32,
    data: String,
}

impl DnsVerificationService {
    pub fn new() -> Self {
        Self {}
    }

    /// Verify a single DNS record using Google's DNS-over-HTTPS API
    pub fn verify_dns_record(&self, record: &DnsRecord) -> Result<bool, AppError> {
        let record_type_num = match record.record_type.as_str() {
            "A" => 1,
            "CNAME" => 5,
            "TXT" => 16,
            "MX" => 15,
            _ => {
                return Err(AppError::BadRequest(format!(
                    "Unsupported DNS record type: {}",
                    record.record_type
                )));
            }
        };

        let url = format!(
            "https://dns.google/resolve?name={}&type={}",
            record.name, record_type_num
        );

        let mut response = ureq::get(&url)
            .header("Accept", "application/dns-json")
            .call()
            .map_err(|e| AppError::External(format!("DNS query failed: {}", e)))?;

        let dns_response: DnsResponse = response
            .body_mut()
            .read_json()
            .map_err(|e| AppError::External(format!("Failed to parse DNS response: {}", e)))?;

        if let Some(answers) = dns_response.answer {
            for answer in answers {
                if self.matches_expected_value(&answer.data, &record.value, &record.record_type) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Check if the DNS answer matches the expected value (ignoring TTL)
    fn matches_expected_value(&self, actual: &str, expected: &str, record_type: &str) -> bool {
        match record_type {
            "CNAME" => {
                // CNAME records often have trailing dots, normalize both
                let actual_normalized = actual.trim_end_matches('.');
                let expected_normalized = expected.trim_end_matches('.');
                actual_normalized.eq_ignore_ascii_case(expected_normalized)
            }
            "TXT" => {
                // TXT records might be quoted
                let actual_unquoted = actual.trim_matches('"');
                actual_unquoted == expected
            }
            "A" => {
                // A records should be exact IP matches
                actual == expected
            }
            "MX" => {
                // MX records have priority and hostname, check if hostname matches
                if let Some(hostname) = actual.split_whitespace().nth(1) {
                    let hostname_normalized = hostname.trim_end_matches('.');
                    let expected_normalized = expected.trim_end_matches('.');
                    hostname_normalized.eq_ignore_ascii_case(expected_normalized)
                } else {
                    false
                }
            }
            _ => actual == expected,
        }
    }

    /// Verify all DNS records in domain verification records
    pub fn verify_domain_records(
        &self,
        records: &mut DomainVerificationRecords,
    ) -> Result<(), AppError> {
        // Verify Cloudflare verification records
        for record in &mut records.cloudflare_verification {
            record.verification_attempted_at = Some(Utc::now());
            match self.verify_dns_record(record) {
                Ok(verified) => {
                    record.verified = verified;
                    if verified {
                        record.last_verified_at = Some(Utc::now());
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to verify DNS record {}: {}", record.name, e);
                    record.verified = false;
                }
            }
        }

        // Verify custom hostname verification records
        for record in &mut records.custom_hostname_verification {
            record.verification_attempted_at = Some(Utc::now());
            match self.verify_dns_record(record) {
                Ok(verified) => {
                    record.verified = verified;
                    if verified {
                        record.last_verified_at = Some(Utc::now());
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to verify DNS record {}: {}", record.name, e);
                    record.verified = false;
                }
            }
        }

        Ok(())
    }

    /// Note: Email verification is now handled by SES service directly
    /// This method is kept for compatibility but delegates to SES
    pub fn verify_email_records(
        &self,
        _records: &mut EmailVerificationRecords,
    ) -> Result<(), AppError> {
        // Email verification is now handled by SES service's verify_email_records method
        // This method is kept for interface compatibility
        Ok(())
    }

    /// Check if all domain verification records are verified
    pub fn are_domain_records_verified(&self, records: &DomainVerificationRecords) -> bool {
        let cloudflare_verified = records.cloudflare_verification.iter().all(|r| r.verified);
        let hostname_verified = records
            .custom_hostname_verification
            .iter()
            .all(|r| r.verified);

        cloudflare_verified && hostname_verified
    }

    /// Check if all email verification records are verified
    pub fn are_email_records_verified(&self, records: &EmailVerificationRecords) -> bool {
        let ses_verified = records.ses_verification.iter().all(|r| r.verified);
        let mail_from_verified = records.mail_from_verification.iter().all(|r| r.verified);
        let dkim_verified = records.dkim_records.iter().all(|r| r.verified);

        ses_verified && mail_from_verified && dkim_verified
    }
}
