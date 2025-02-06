use serde_json::json;

use crate::application::ApiResult;

pub async fn check() -> ApiResult<serde_json::Value> {
    Ok(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now()
    })
    .into())
}
