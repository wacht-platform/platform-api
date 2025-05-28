use crate::application::AppError;
use crate::core::models::{DnsRecord, DomainVerificationRecords};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct CreateCustomHostnameRequest {
    pub hostname: String,
    pub custom_origin_server: String,
}

#[derive(Debug, Deserialize)]
pub struct CloudflareResponse<T> {
    pub success: bool,
    pub errors: Vec<CloudflareError>,
    pub messages: Vec<String>,
    pub result: Option<T>,
}

#[derive(Debug, Deserialize)]
pub struct CloudflareError {
    pub code: u32,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct CustomHostname {
    pub id: String,
    pub hostname: String,
    pub custom_origin_server: String,
    pub status: String,
    pub verification_errors: Option<Vec<String>>,
}

#[derive(Clone)]
pub struct CloudflareService {
    api_key: String,
    zone_id: String,
}

impl CloudflareService {
    pub fn new(api_key: String, zone_id: String) -> Self {
        Self { api_key, zone_id }
    }

    pub fn create_custom_hostname(
        &self,
        hostname: &str,
        origin_server: &str,
    ) -> Result<CustomHostname, AppError> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/custom_hostnames",
            self.zone_id
        );

        let request_body = CreateCustomHostnameRequest {
            hostname: hostname.to_string(),
            custom_origin_server: origin_server.to_string(),
        };

        let mut response = ureq::post(&url)
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .send_json(&request_body)
            .map_err(|e| AppError::External(format!("Cloudflare API request failed: {}", e)))?;

        let cloudflare_response: CloudflareResponse<CustomHostname> =
            response.body_mut().read_json().map_err(|e| {
                AppError::External(format!("Failed to parse Cloudflare response: {}", e))
            })?;

        if !cloudflare_response.success {
            let error_messages: Vec<String> = cloudflare_response
                .errors
                .iter()
                .map(|e| format!("{}: {}", e.code, e.message))
                .collect();
            return Err(AppError::External(format!(
                "Cloudflare API errors: {}",
                error_messages.join(", ")
            )));
        }

        cloudflare_response.result.ok_or_else(|| {
            AppError::External("Cloudflare API returned success but no result".to_string())
        })
    }

    pub fn delete_custom_hostname(&self, hostname_id: &str) -> Result<(), AppError> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/custom_hostnames/{}",
            self.zone_id, hostname_id
        );

        let response = ureq::delete(&url)
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .call()
            .map_err(|e| AppError::External(format!("Cloudflare API request failed: {}", e)))?;

        if response.status() != 200 {
            return Err(AppError::External(format!(
                "Cloudflare API error ({})",
                response.status()
            )));
        }

        Ok(())
    }

    /// Generate DNS records required for custom hostname verification
    pub fn generate_domain_verification_records(
        &self,
        frontend_hostname: &str,
        backend_hostname: &str,
    ) -> DomainVerificationRecords {
        let mut records = DomainVerificationRecords::default();

        // Add CNAME records for custom hostnames
        records.custom_hostname_verification.push(DnsRecord {
            name: frontend_hostname.to_string(),
            record_type: "CNAME".to_string(),
            value: "accounts.wacht.services".to_string(),
            ttl: Some(300),
            verified: false,
            verification_attempted_at: None,
            last_verified_at: None,
        });

        records.custom_hostname_verification.push(DnsRecord {
            name: backend_hostname.to_string(),
            record_type: "CNAME".to_string(),
            value: "fapi.wacht.services".to_string(),
            ttl: Some(300),
            verified: false,
            verification_attempted_at: None,
            last_verified_at: None,
        });

        records
    }
}
