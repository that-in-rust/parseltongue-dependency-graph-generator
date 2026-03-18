//! Git history recency signal: scores files based on how recently they were modified.

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Build a mapping of file paths to recency scores based on git log history.
///
/// Runs `git log --pretty=format:"%at" --name-only -n 500` in the given repo root.
/// For each file, computes an exponential decay score: `e^(-age_days / 30.0)`.
///
/// Returns an empty `HashMap` if:
/// - git is not available
/// - the path is not a git repository
/// - any other error occurs (no panic)
pub fn build_git_recency_scores(repo_root: &Path) -> HashMap<String, f64> {
    let output = match Command::new("git")
        .args(["log", "--pretty=format:%at", "--name-only", "-n", "500"])
        .current_dir(repo_root)
        .output()
    {
        Ok(o) => o,
        Err(_) => return HashMap::new(),
    };

    if !output.status.success() {
        return HashMap::new();
    }

    let stdout = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(_) => return HashMap::new(),
    };

    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as f64;

    let mut scores: HashMap<String, f64> = HashMap::new();
    let mut current_timestamp: Option<f64> = None;

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Try to parse as a timestamp (unix epoch seconds)
        if let Ok(ts) = trimmed.parse::<u64>() {
            current_timestamp = Some(ts as f64);
            continue;
        }

        // Otherwise it's a file path
        if let Some(ts) = current_timestamp {
            let age_secs = (now_secs - ts).max(0.0);
            let age_days = age_secs / 86400.0;
            let score = (-age_days / 30.0_f64).exp();

            // Keep the highest score (most recent commit) for each file
            let entry = scores.entry(trimmed.to_string()).or_insert(0.0);
            if score > *entry {
                *entry = score;
            }
        }
    }

    scores
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_git_recency_on_repo() {
        // Run on the actual repo root — should return a non-empty map
        let repo_root = PathBuf::from("/code/that-in-rust/parseltongue-rust-LLM-companion");
        let scores = build_git_recency_scores(&repo_root);
        println!(
            "git recency scores: {} files, sample: {:?}",
            scores.len(),
            scores.iter().take(5).collect::<Vec<_>>()
        );
        assert!(
            !scores.is_empty(),
            "expected non-empty git recency scores on repo root"
        );
        // All scores should be in (0, 1]
        for (path, score) in &scores {
            assert!(
                *score > 0.0 && *score <= 1.0,
                "score for {path} out of range: {score}"
            );
        }
    }

    #[test]
    fn test_git_recency_on_tmp_no_panic() {
        // /tmp is not a git repo — should return empty, no panic
        let scores = build_git_recency_scores(Path::new("/tmp"));
        assert!(
            scores.is_empty(),
            "expected empty map for non-git directory"
        );
    }
}
