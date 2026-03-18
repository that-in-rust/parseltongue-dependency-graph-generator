//! Health check endpoint: returns server status and version.

use axum::Json;
use serde_json::{json, Value};

/// GET /server-health-check-status
pub async fn handle_health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": "3.0.0",
        "service": "parseltongue-http-query-api"
    }))
}
