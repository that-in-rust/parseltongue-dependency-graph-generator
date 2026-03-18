//! Server process management: start/stop HTTP server instances per workspace.

use std::collections::HashMap;

use anyhow::{Context, Result};
use tokio::task::JoinHandle;

use pt08_http_query_api_server::http_server_startup_runner::start_http_server_listener;
use pt08_http_query_api_server::shared_server_app_state::build_server_app_state;

/// A running server instance for a single workspace.
pub struct ServerInstance {
    pub workspace_id: String,
    pub port: u16,
    pub handle: JoinHandle<Result<()>>,
}

/// Manages zero or more HTTP server instances, keyed by workspace id.
pub struct ServerProcessManager {
    pub instances: HashMap<String, ServerInstance>,
}

impl ServerProcessManager {
    /// Create a new empty manager.
    pub fn new() -> Self {
        Self {
            instances: HashMap::new(),
        }
    }

    /// Start an HTTP server for the given workspace.
    ///
    /// Binds to port 0 to find a free port, then spawns a background task
    /// running the pt08 server. Returns the URL (e.g. `http://127.0.0.1:<port>`).
    pub async fn start_http_server_instance(
        &mut self,
        workspace_id: &str,
        db_path: &str,
    ) -> Result<String> {
        // Find a free port by binding to 0
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .context("Failed to bind to ephemeral port")?;
        let port = listener.local_addr()?.port();
        // Drop the listener so the port is available for the actual server
        drop(listener);

        let state = build_server_app_state(db_path).await?;
        let wid = workspace_id.to_string();

        let handle: JoinHandle<Result<()>> = tokio::spawn(async move {
            start_http_server_listener(state, port).await
        });

        let url = format!("http://127.0.0.1:{port}");

        self.instances.insert(
            wid.clone(),
            ServerInstance {
                workspace_id: wid,
                port,
                handle,
            },
        );

        Ok(url)
    }

    /// Stop the server for the given workspace, aborting its task.
    pub async fn stop_http_server_instance(&mut self, workspace_id: &str) -> Result<()> {
        let instance = self
            .instances
            .remove(workspace_id)
            .context(format!("No server running for workspace {workspace_id}"))?;
        instance.handle.abort();
        Ok(())
    }

    /// Get the URL of a running server, if any.
    pub fn get_server_url(&self, workspace_id: &str) -> Option<String> {
        self.instances
            .get(workspace_id)
            .map(|inst| format!("http://127.0.0.1:{}", inst.port))
    }
}

impl Default for ServerProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_manager_basic() {
        let mgr = ServerProcessManager::new();
        assert!(mgr.instances.is_empty());
        assert!(mgr.get_server_url("nonexistent").is_none());
    }
}
