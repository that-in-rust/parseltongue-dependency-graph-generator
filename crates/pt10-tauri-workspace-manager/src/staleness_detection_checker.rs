//! Staleness detection: compare on-disk file counts with stored entity counts.

use std::path::Path;

use anyhow::Result;
use walkdir::WalkDir;

use crate::workspace_lifecycle_commands::{WorkspaceInfo, WorkspaceStatus};

/// Check every workspace and return `(workspace_id, status)` pairs.
///
/// For each workspace:
/// - If `root_path` no longer exists → `Stale`
/// - If the DB file doesn't exist → `Unindexed`
/// - Otherwise count files on disk (max depth 3 for speed) and compare to
///   the stored `entity_count`. If the ratio differs by more than 20 %, mark
///   `Stale`; otherwise `Fresh`.
pub fn check_staleness_status_all(
    workspaces: &[WorkspaceInfo],
) -> Result<Vec<(String, WorkspaceStatus)>> {
    let mut results = Vec::with_capacity(workspaces.len());

    for ws in workspaces {
        let root = Path::new(&ws.root_path);

        if !root.exists() {
            results.push((ws.id.clone(), WorkspaceStatus::Stale));
            continue;
        }

        let db = Path::new(&ws.db_path);
        if !db.exists() {
            results.push((ws.id.clone(), WorkspaceStatus::Unindexed));
            continue;
        }

        // Quick file count with limited depth for speed
        let file_count = WalkDir::new(root)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .count() as u64;

        let status = if ws.entity_count == 0 && file_count == 0 {
            WorkspaceStatus::Fresh
        } else if ws.entity_count == 0 {
            // DB has no entities but files exist on disk
            WorkspaceStatus::Stale
        } else {
            let ratio = file_count as f64 / ws.entity_count as f64;
            if ratio < 0.8 || ratio > 1.2 {
                WorkspaceStatus::Stale
            } else {
                WorkspaceStatus::Fresh
            }
        };

        results.push((ws.id.clone(), status));
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_staleness_fresh() {
        let ws_dir = TempDir::new().unwrap();
        // Create some files so we have a non-zero count
        for i in 0..5 {
            std::fs::write(ws_dir.path().join(format!("file{i}.rs")), "fn main() {}").unwrap();
        }

        // Create a fake DB file
        let pt_dir = ws_dir.path().join(".parseltongue");
        std::fs::create_dir_all(&pt_dir).unwrap();
        let db_path = pt_dir.join("parseltongue.db");
        std::fs::write(&db_path, "fake-db").unwrap();

        let ws = WorkspaceInfo {
            id: "test-id".to_string(),
            root_path: ws_dir.path().to_str().unwrap().to_string(),
            name: "test".to_string(),
            db_path: db_path.to_str().unwrap().to_string(),
            status: WorkspaceStatus::Fresh,
            // 7 files on disk: 5 .rs files + .parseltongue dir has parseltongue.db = 6 files
            // but we need entity_count to be close to actual file count
            // Let's count: 5 .rs files + 1 db file = 6 files total
            entity_count: 6,
            edge_count: 0,
            last_indexed_at: None,
        };

        let results = check_staleness_status_all(&[ws]).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1, WorkspaceStatus::Fresh);
    }
}
