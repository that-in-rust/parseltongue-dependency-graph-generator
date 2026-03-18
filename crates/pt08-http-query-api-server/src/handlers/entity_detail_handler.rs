//! Entity detail endpoint: returns full information for a single entity by PK.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};

use parseltongue_core::entity_type_definitions::EntityPrimaryKeyLocation;

use crate::shared_server_app_state::SharedServerAppState;

/// GET /code-entity-detail-view/{pk}
///
/// The `pk` path parameter is formatted as `file_path:start_line:end_line`.
pub async fn handle_entity_detail(
    State(state): State<Arc<SharedServerAppState>>,
    Path(pk_str): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let pk: EntityPrimaryKeyLocation = pk_str.parse().map_err(|e: String| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": format!("invalid PK format: {e}") })),
        )
    })?;

    let result = state
        .store
        .get_entity_by_pk(
            &pk.file_path,
            pk.start_line,
            pk.end_line,
            state.codebase_id,
        )
        .await;

    match result {
        Ok(Some(entity)) => {
            let neighbors_fwd = state.graph.get_forward_neighbor_nodes(&pk_str);
            let neighbors_rev = state.graph.get_reverse_neighbor_nodes(&pk_str);

            Ok(Json(json!({
                "pk": entity.pk.to_string(),
                "name": entity.name,
                "entity_type": entity.entity_type.to_string(),
                "language": entity.language,
                "file_path": entity.pk.file_path,
                "start_line": entity.pk.start_line,
                "end_line": entity.pk.end_line,
                "signature": entity.signature,
                "snippet": entity.snippet,
                "doc_comment": entity.doc_comment,
                "wc": entity.wc,
                "is_test": entity.is_test,
                "visibility": entity.visibility,
                "forward_neighbors": neighbors_fwd.iter().map(|(t, et)| json!({"target": t, "edge_type": et})).collect::<Vec<_>>(),
                "reverse_neighbors": neighbors_rev.iter().map(|(s, et)| json!({"source": s, "edge_type": et})).collect::<Vec<_>>(),
            })))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": format!("entity not found: {pk_str}") })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("database error: {e}") })),
        )),
    }
}
