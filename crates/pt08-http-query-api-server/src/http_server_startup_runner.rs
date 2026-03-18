//! HTTP server startup: binds a TCP listener and serves the Axum router.

use std::sync::Arc;

use anyhow::Result;
use tokio::net::TcpListener;

use crate::route_definition_builder::build_route_definitions;
use crate::shared_server_app_state::SharedServerAppState;

/// Start the HTTP server on the given port.
///
/// Builds the router from shared state, binds a `TcpListener`, and serves
/// requests until the process is terminated.
pub async fn start_http_server_listener(
    state: Arc<SharedServerAppState>,
    port: u16,
) -> Result<()> {
    let router = build_route_definitions(state);

    let addr = format!("0.0.0.0:{port}");
    tracing::info!("Starting HTTP server on {addr}");

    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
