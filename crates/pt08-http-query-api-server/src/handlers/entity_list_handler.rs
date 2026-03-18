//! Entity list endpoint: returns all code entities in the codebase.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

use crate::shared_server_app_state::SharedServerAppState;

/// GET /code-entities-list-all
pub async fn handle_entity_list(
    State(state): State<Arc<SharedServerAppState>>,
) -> Json<Value> {
    let result = state
        .store
        .get_all_entity_records(state.codebase_id)
        .await;

    match result {
        Ok(entities) => {
            let items: Vec<Value> = entities
                .iter()
                .map(|e| {
                    json!({
                        "pk": e.pk.to_string(),
                        "name": e.name,
                        "entity_type": e.entity_type.to_string(),
                        "language": e.language,
                        "file_path": e.pk.file_path,
                        "start_line": e.pk.start_line,
                        "end_line": e.pk.end_line,
                        "signature": e.signature,
                        "wc": e.wc,
                        "is_test": e.is_test,
                    })
                })
                .collect();
            Json(json!({
                "count": items.len(),
                "entities": items,
            }))
        }
        Err(e) => Json(json!({
            "error": format!("failed to retrieve entities: {e}")
        })),
    }
}
