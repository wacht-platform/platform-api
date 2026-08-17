use chrono::{DateTime, Utc};
use common::error::AppError;
use common::ResultExt;
use std::time::Duration;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use super::types::{CachedContentResponse, ExplicitCachePlan, PreparedGenerateRequest};
use super::{ExplicitCacheRequest, GeminiClient};

const GEMINI_API_ROOT_URL: &str = "https://generativelanguage.googleapis.com/v1beta";
const REQUEST_TIMEOUT_SECS: u64 = 240;
const EXPLICIT_CACHE_MIN_TOKENS_FLASH: usize = 4_096;
const EXPLICIT_CACHE_MIN_TOKENS_PRO: usize = 4_096;
const EXPLICIT_CACHE_ESTIMATED_CHARS_PER_TOKEN: usize = 4;
/// PATCH TTL when less than this remains, so a 1-day cache never dies mid-run.
const EXPLICIT_CACHE_RENEW_LEAD_SECS: i64 = 2 * 3_600;

impl GeminiClient {
    pub(crate) async fn prepare_generate_request_body(
        &self,
        request_body: String,
        cache_request: Option<&ExplicitCacheRequest>,
    ) -> PreparedGenerateRequest {
        let Some(cache_request) = cache_request else {
            return Self::uncached_prepare(request_body);
        };

        let Some(mut plan) = self.build_explicit_cache_plan(&request_body, cache_request) else {
            return Self::uncached_prepare(request_body);
        };

        if cache_request.reuse_only && !self.plan_can_reuse_prior(cache_request, &plan) {
            return Self::uncached_prepare(request_body);
        }

        let cache_state = if cache_request.reuse_only {
            cache_request.prior_state.clone()
        } else {
            match self.ensure_explicit_cache(cache_request, &plan).await {
                Ok(state) => state,
                Err(error) => {
                    tracing::warn!(
                        model = %self.model,
                        cache_key = %cache_request.cache_key,
                        error = %error,
                        "Gemini explicit cache create/renew failed; sending uncached request"
                    );
                    return Self::uncached_prepare(request_body);
                }
            }
        };

        if let Some(state) = cache_state.as_ref() {
            if let Some(object) = plan.send_request_payload.as_object_mut() {
                object.insert("cachedContent".to_string(), json!(state.cache_name));
            }
        }

        let request_body =
            serde_json::to_string(&plan.send_request_payload).unwrap_or(request_body);
        PreparedGenerateRequest {
            request_body,
            cache_state,
        }
    }

    fn uncached_prepare(request_body: String) -> PreparedGenerateRequest {
        PreparedGenerateRequest {
            request_body,
            cache_state: None,
        }
    }

    fn build_explicit_cache_plan(
        &self,
        request_body: &str,
        cache_request: &ExplicitCacheRequest,
    ) -> Option<ExplicitCachePlan> {
        let mut send_request_payload = serde_json::from_str::<Value>(request_body).ok()?;
        let send_request_object = send_request_payload.as_object_mut()?;

        if send_request_object.get("cachedContent").is_some()
            || send_request_object.get("cached_content").is_some()
        {
            return None;
        }

        let system_instruction = send_request_object
            .get("system_instruction")
            .cloned()
            .or_else(|| send_request_object.get("systemInstruction").cloned());

        let tools = send_request_object.get("tools").cloned();
        // toolConfig stays on the generate request: forced-tool nudges must not
        // bust the shared agent cache.

        let contents = send_request_object
            .get("contents")
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default();

        if system_instruction.is_none() && tools.is_none() && contents.is_empty() {
            return None;
        }

        let cacheable_content_count = contents
            .len()
            .saturating_sub(cache_request.live_tail_count.min(contents.len()));
        let cacheable_contents = contents[..cacheable_content_count].to_vec();
        let live_tail_contents = contents[cacheable_content_count..].to_vec();

        let stable_payload = json!({
            "systemInstruction": system_instruction.clone(),
            "tools": tools.clone(),
            "contents": cacheable_contents.clone(),
        });
        let prefix_signature =
            self.short_hash(serde_json::to_string(&stable_payload).ok()?.as_bytes(), 32);
        let cached_contents_signature = self.short_hash(
            serde_json::to_string(&cacheable_contents).ok()?.as_bytes(),
            32,
        );

        let mut full_cache_payload = serde_json::Map::new();
        full_cache_payload.insert("model".to_string(), json!(format!("models/{}", self.model)));
        full_cache_payload.insert(
            "displayName".to_string(),
            json!(format!(
                "agent-engine-{}",
                self.short_hash(cache_request.cache_key.as_bytes(), 12)
            )),
        );
        full_cache_payload.insert(
            "ttl".to_string(),
            json!(format!("{}s", cache_request.ttl_secs.max(1))),
        );

        if let Some(system_instruction) = system_instruction {
            full_cache_payload.insert("systemInstruction".to_string(), system_instruction);
        }

        if !cacheable_contents.is_empty() {
            full_cache_payload.insert(
                "contents".to_string(),
                Value::Array(cacheable_contents.clone()),
            );
        }

        if let Some(tools) = tools {
            full_cache_payload.insert("tools".to_string(), tools);
        }

        let full_cache_payload = Value::Object(full_cache_payload);
        let estimated_prefix_tokens = Self::estimate_tokens(&full_cache_payload);
        if estimated_prefix_tokens < Self::explicit_cache_min_tokens_for_model(&self.model) {
            return None;
        }

        let mut should_refresh = true;
        let mut should_renew_ttl = false;
        if let Some(prior_state) = cache_request.prior_state.as_ref() {
            let probe = ExplicitCachePlan {
                full_cache_payload: full_cache_payload.clone(),
                send_request_payload: Value::Null,
                prefix_signature: prefix_signature.clone(),
                cached_contents_signature: cached_contents_signature.clone(),
                cached_content_count: cacheable_contents.len(),
                should_refresh: true,
                should_renew_ttl: false,
            };
            if self.can_use_cached_prefix(cache_request, prior_state, &probe) {
                should_refresh = false;
                should_renew_ttl = prior_state.expire_at
                    <= Utc::now() + chrono::Duration::seconds(EXPLICIT_CACHE_RENEW_LEAD_SECS);

                send_request_object.remove("system_instruction");
                send_request_object.remove("systemInstruction");
                send_request_object.remove("tools");
                send_request_object.insert(
                    "cachedContent".to_string(),
                    json!(prior_state.cache_name),
                );
                send_request_object.insert("contents".to_string(), Value::Array(live_tail_contents));
            }
        }

        Some(ExplicitCachePlan {
            full_cache_payload,
            send_request_payload,
            prefix_signature,
            cached_contents_signature,
            cached_content_count: cacheable_contents.len(),
            should_refresh,
            should_renew_ttl,
        })
    }

    fn plan_can_reuse_prior(
        &self,
        cache_request: &ExplicitCacheRequest,
        plan: &ExplicitCachePlan,
    ) -> bool {
        cache_request
            .prior_state
            .as_ref()
            .is_some_and(|prior| self.can_use_cached_prefix(cache_request, prior, plan))
    }

    fn can_use_cached_prefix(
        &self,
        cache_request: &ExplicitCacheRequest,
        prior_state: &models::PromptCacheState,
        plan: &ExplicitCachePlan,
    ) -> bool {
        prior_state.cache_key == cache_request.cache_key
            && prior_state.model_name == self.model
            && prior_state.expire_at > Utc::now() + chrono::Duration::seconds(5)
            && prior_state.prefix_signature == plan.prefix_signature
            && prior_state.cached_contents_signature == plan.cached_contents_signature
            && prior_state.cached_content_count == plan.cached_content_count
    }

    async fn ensure_explicit_cache(
        &self,
        cache_request: &ExplicitCacheRequest,
        plan: &ExplicitCachePlan,
    ) -> Result<Option<models::PromptCacheState>, AppError> {
        if plan.should_refresh {
            return self.create_explicit_cache(cache_request, plan).await;
        }

        let Some(prior_state) = cache_request.prior_state.as_ref() else {
            return Ok(None);
        };

        if plan.should_renew_ttl {
            match self
                .renew_explicit_cache_ttl(&prior_state.cache_name, cache_request.ttl_secs)
                .await
            {
                Ok(expire_at) => {
                    let mut renewed = prior_state.clone();
                    renewed.expire_at = expire_at;
                    renewed.reuse_turns = renewed.reuse_turns.saturating_add(1);
                    return Ok(Some(renewed));
                }
                Err(error) => {
                    tracing::warn!(
                        cache_name = %prior_state.cache_name,
                        error = %error,
                        "Gemini cache TTL renew failed; reusing remaining TTL"
                    );
                }
            }
        }

        let mut aged = prior_state.clone();
        aged.reuse_turns = aged.reuse_turns.saturating_add(1);
        Ok(Some(aged))
    }

    async fn create_explicit_cache(
        &self,
        cache_request: &ExplicitCacheRequest,
        plan: &ExplicitCachePlan,
    ) -> Result<Option<models::PromptCacheState>, AppError> {
        let cache_url = format!("{}/cachedContents", GEMINI_API_ROOT_URL);
        let serialized_payload = serde_json::to_string(&plan.full_cache_payload).map_err(|e| {
            AppError::Internal(format!("Failed to serialize Gemini cache payload: {e}"))
        })?;
        let cache_response = self
            .client
            .post(&cache_url)
            .header("x-goog-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .body(serialized_payload)
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .send()
            .await
            .map_err_internal("Gemini cache create request failed")?;

        if !cache_response.status().is_success() {
            let status = cache_response.status();
            let body = cache_response.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "Gemini cache create request failed with status {}: {}",
                status,
                body.chars().take(2000).collect::<String>()
            )));
        }

        let response_body = cache_response.text().await.map_err(|e| {
            AppError::Internal(format!("Failed to read Gemini cache create response: {e}"))
        })?;
        let parsed: CachedContentResponse = serde_json::from_str(&response_body).map_err(|e| {
            AppError::Internal(format!(
                "Failed to parse Gemini cache create response JSON: {}. Raw body (first 2000 chars): {}",
                e,
                response_body.chars().take(2000).collect::<String>()
            ))
        })?;

        let expire_at = parsed
            .expire_time
            .as_deref()
            .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
            .map(|value: DateTime<chrono::FixedOffset>| value.with_timezone(&Utc))
            .unwrap_or_else(|| Utc::now() + chrono::Duration::seconds(cache_request.ttl_secs));

        let handle = models::PromptCacheState {
            cache_key: cache_request.cache_key.clone(),
            model_name: self.model.clone(),
            cache_name: parsed.name.clone(),
            prefix_signature: plan.prefix_signature.clone(),
            cached_contents_signature: plan.cached_contents_signature.clone(),
            cached_content_count: plan.cached_content_count,
            expire_at,
            reuse_turns: 0,
        };

        if let Some(prior) = cache_request.prior_state.as_ref() {
            if prior.cache_name != handle.cache_name {
                self.delete_explicit_cache(&prior.cache_name).await;
            }
        }

        Ok(Some(handle))
    }

    async fn renew_explicit_cache_ttl(
        &self,
        cache_name: &str,
        ttl_secs: i64,
    ) -> Result<DateTime<Utc>, AppError> {
        let url = format!("{GEMINI_API_ROOT_URL}/{cache_name}");
        let body = json!({ "ttl": format!("{}s", ttl_secs.max(1)) });
        let response = self
            .client
            .patch(&url)
            .header("x-goog-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .send()
            .await
            .map_err_internal("Gemini cache TTL renew request failed")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "Gemini cache TTL renew failed with status {}: {}",
                status,
                body.chars().take(500).collect::<String>()
            )));
        }

        let response_body = response.text().await.unwrap_or_default();
        let parsed: CachedContentResponse = serde_json::from_str(&response_body).unwrap_or(
            CachedContentResponse {
                name: cache_name.to_string(),
                expire_time: None,
                usage_metadata: None,
            },
        );
        let expire_at = parsed
            .expire_time
            .as_deref()
            .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
            .map(|value: DateTime<chrono::FixedOffset>| value.with_timezone(&Utc))
            .unwrap_or_else(|| Utc::now() + chrono::Duration::seconds(ttl_secs.max(1)));
        Ok(expire_at)
    }

    pub(crate) async fn delete_explicit_cache(&self, cache_name: &str) {
        if cache_name.is_empty() {
            return;
        }
        let url = format!("{GEMINI_API_ROOT_URL}/{cache_name}");
        match self
            .client
            .delete(&url)
            .header("x-goog-api-key", &self.api_key)
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() || resp.status().as_u16() == 404 => {}
            Ok(resp) => {
                tracing::warn!(cache_name, status = %resp.status(), "Gemini cache delete failed")
            }
            Err(error) => {
                tracing::warn!(cache_name, %error, "Gemini cache delete request errored")
            }
        }
    }

    fn explicit_cache_min_tokens_for_model(model: &str) -> usize {
        let model = model.to_ascii_lowercase();
        if model.contains("flash") {
            EXPLICIT_CACHE_MIN_TOKENS_FLASH
        } else if model.contains("pro") {
            EXPLICIT_CACHE_MIN_TOKENS_PRO
        } else {
            EXPLICIT_CACHE_MIN_TOKENS_PRO
        }
    }

    fn estimate_tokens(value: &Value) -> usize {
        let approximate_chars = serde_json::to_string(value)
            .map(|serialized| serialized.chars().count())
            .unwrap_or_default();
        approximate_chars.div_ceil(EXPLICIT_CACHE_ESTIMATED_CHARS_PER_TOKEN)
    }

    fn short_hash(&self, bytes: &[u8], length: usize) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let digest = hasher.finalize();
        let mut encoded = String::with_capacity(digest.len() * 2);
        for byte in digest {
            encoded.push_str(&format!("{byte:02x}"));
        }
        encoded.chars().take(length).collect()
    }
}
