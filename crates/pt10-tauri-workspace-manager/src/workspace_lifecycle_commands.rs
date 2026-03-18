//! Workspace lifecycle management: add, list, remove workspaces and persist config.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Status of a workspace's index.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum WorkspaceStatus {
    Fresh,
    Stale,
    Unindexed,
}

/// Metadata about a registered workspace.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub id: String,
    pub root_path: String,
    pub name: String,
    pub db_path: String,
    pub status: WorkspaceStatus,
    pub entity_count: u64,
    pub edge_count: u64,
    pub last_indexed_at: Option<i64>,
}

/// Manages the set of registered workspaces and their on-disk config.
pub struct WorkspaceManager {
    pub config_path: PathBuf,
    pub workspaces: Vec<WorkspaceInfo>,
}

impl WorkspaceManager {
    /// Create a new manager, loading existing config from `config_dir/workspaces.json`
    /// or starting with an empty list.
    pub fn new(config_dir: &Path) -> Result<Self> {
        let config_path = config_dir.join("workspaces.json");
        let workspaces = if config_path.exists() {
            Self::load_config(&config_path)
                .context("Failed to load workspace config")?
        } else {
            Vec::new()
        };
        Ok(Self {
            config_path,
            workspaces,
        })
    }

    /// Register a new workspace folder path.
    ///
    /// Validates the path exists, creates a `.parseltongue` directory inside it,
    /// initialises a database path, and persists the updated config.
    pub fn add_workspace_folder_path(&mut self, path: &str) -> Result<WorkspaceInfo> {
        let root = Path::new(path);
        anyhow::ensure!(root.exists(), "Path does not exist: {path}");

        let pt_dir = root.join(".parseltongue");
        std::fs::create_dir_all(&pt_dir)
            .context("Failed to create .parseltongue directory")?;

        let db_path = pt_dir.join("parseltongue.db");

        let name = root
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string());

        let info = WorkspaceInfo {
            id: Uuid::new_v4().to_string(),
            root_path: path.to_string(),
            name,
            db_path: db_path.to_string_lossy().to_string(),
            status: WorkspaceStatus::Unindexed,
            entity_count: 0,
            edge_count: 0,
            last_indexed_at: None,
        };

        self.workspaces.push(info.clone());
        self.save_config()?;
        Ok(info)
    }

    /// Return all registered workspaces.
    pub fn list_all_workspace_entries(&self) -> Vec<WorkspaceInfo> {
        self.workspaces.clone()
    }

    /// Remove a workspace by its id.
    pub fn remove_workspace_entry_path(&mut self, workspace_id: &str) -> Result<()> {
        let before = self.workspaces.len();
        self.workspaces.retain(|w| w.id != workspace_id);
        anyhow::ensure!(
            self.workspaces.len() < before,
            "Workspace not found: {workspace_id}"
        );
        self.save_config()?;
        Ok(())
    }

    /// Persist the current workspace list to JSON.
    pub fn save_config(&self) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&self.workspaces)?;
        std::fs::write(&self.config_path, json)?;
        Ok(())
    }

    /// Load a workspace list from a JSON file.
    pub fn load_config(path: &Path) -> Result<Vec<WorkspaceInfo>> {
        let data = std::fs::read_to_string(path)?;
        let workspaces: Vec<WorkspaceInfo> = serde_json::from_str(&data)?;
        Ok(workspaces)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_workspace_add() {
        let config_dir = TempDir::new().unwrap();
        let workspace_dir = TempDir::new().unwrap();

        let mut mgr = WorkspaceManager::new(config_dir.path()).unwrap();
        let info = mgr
            .add_workspace_folder_path(workspace_dir.path().to_str().unwrap())
            .unwrap();

        assert_eq!(info.root_path, workspace_dir.path().to_str().unwrap());
        assert_eq!(info.status, WorkspaceStatus::Unindexed);
        assert!(!info.id.is_empty());
    }

    #[test]
    fn test_workspace_list() {
        let config_dir = TempDir::new().unwrap();
        let ws1 = TempDir::new().unwrap();
        let ws2 = TempDir::new().unwrap();

        let mut mgr = WorkspaceManager::new(config_dir.path()).unwrap();
        mgr.add_workspace_folder_path(ws1.path().to_str().unwrap())
            .unwrap();
        mgr.add_workspace_folder_path(ws2.path().to_str().unwrap())
            .unwrap();

        let list = mgr.list_all_workspace_entries();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_workspace_remove() {
        let config_dir = TempDir::new().unwrap();
        let ws = TempDir::new().unwrap();

        let mut mgr = WorkspaceManager::new(config_dir.path()).unwrap();
        let info = mgr
            .add_workspace_folder_path(ws.path().to_str().unwrap())
            .unwrap();

        mgr.remove_workspace_entry_path(&info.id).unwrap();
        assert!(mgr.list_all_workspace_entries().is_empty());
    }
}
