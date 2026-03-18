//! Reciprocal Rank Fusion (RRF) combiner for merging 4 search signals.

use std::collections::HashMap;

use crate::entity_type_definitions::EntityPrimaryKeyLocation;

/// RRF constant k (smoothing factor).
const RRF_K: f64 = 60.0;

/// Signal weights.
const W_FTS: f64 = 0.35;
const W_TRIE: f64 = 0.30;
const W_TRIGRAM: f64 = 0.20;
const W_GIT: f64 = 0.15;

/// A search result after RRF fusion of multiple signals.
#[derive(Clone, Debug)]
pub struct RrfSearchResult {
    /// The entity primary key location.
    pub entity_pk: EntityPrimaryKeyLocation,
    /// The entity name.
    pub name: String,
    /// The combined RRF score.
    pub score: f64,
    /// Which signals contributed to this result.
    pub signals: Vec<String>,
}

/// Combine four ranked signal lists using Reciprocal Rank Fusion.
///
/// RRF formula per entity:
/// ```text
/// score = w1/(k + rank_fts) + w2/(k + rank_trie) + w3/(k + rank_trigram) + w4/(k + rank_git)
/// ```
///
/// Where `rank` is 1-based position in each signal's ranked list.
/// Entities not present in a signal's list receive no contribution from that signal.
///
/// # Arguments
/// - `fts_results` — Full-text search ranked results (name, pk, _score)
/// - `trie_results` — Symbol trie exact/prefix ranked results
/// - `trigram_results` — Trigram fuzzy match ranked results
/// - `git_scores` — Git recency scores (file_path -> score); converted to a ranked list internally
/// - `limit` — Maximum number of results to return
pub fn combine_rrf_search_signals(
    fts_results: &[(String, EntityPrimaryKeyLocation, f64)],
    trie_results: &[(String, EntityPrimaryKeyLocation, f64)],
    trigram_results: &[(String, EntityPrimaryKeyLocation, f64)],
    git_scores: &HashMap<String, f64>,
    limit: usize,
) -> Vec<RrfSearchResult> {
    // Accumulator: entity PK string -> (name, EntityPrimaryKeyLocation, accumulated score, signals)
    let mut acc: HashMap<String, (String, EntityPrimaryKeyLocation, f64, Vec<String>)> =
        HashMap::new();

    // Helper: process a ranked list with a given weight and signal name
    let mut add_signal = |results: &[(String, EntityPrimaryKeyLocation, f64)],
                          weight: f64,
                          signal_name: &str| {
        for (rank_0, (name, pk, _score)) in results.iter().enumerate() {
            let rank = (rank_0 + 1) as f64; // 1-based
            let rrf_contribution = weight / (RRF_K + rank);
            let pk_key = pk.to_string();
            let entry = acc
                .entry(pk_key)
                .or_insert_with(|| (name.clone(), pk.clone(), 0.0, Vec::new()));
            entry.2 += rrf_contribution;
            entry.3.push(signal_name.to_string());
        }
    };

    // Add FTS signal
    add_signal(fts_results, W_FTS, "fts");

    // Add trie signal
    add_signal(trie_results, W_TRIE, "trie");

    // Add trigram signal
    add_signal(trigram_results, W_TRIGRAM, "trigram");

    // Add git recency signal: convert to a ranked list sorted by score descending
    if !git_scores.is_empty() {
        let mut git_ranked: Vec<(&String, &f64)> = git_scores.iter().collect();
        git_ranked.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));

        for (rank_0, (file_path, _score)) in git_ranked.iter().enumerate() {
            let rank = (rank_0 + 1) as f64;
            let rrf_contribution = W_GIT / (RRF_K + rank);

            // Find any existing entries whose PK file_path matches this git file
            // We need to boost entities in files that are recently modified
            let matching_pks: Vec<String> = acc
                .iter()
                .filter(|(_k, (_, pk, _, _))| pk.file_path == **file_path)
                .map(|(k, _)| k.clone())
                .collect();

            if matching_pks.is_empty() {
                // Create an entry for the file itself (file sentinel)
                let pk = EntityPrimaryKeyLocation::file_sentinel(file_path.as_str());
                let pk_key = pk.to_string();
                let entry = acc
                    .entry(pk_key)
                    .or_insert_with(|| (file_path.to_string(), pk, 0.0, Vec::new()));
                entry.2 += rrf_contribution;
                entry.3.push("git".to_string());
            } else {
                for pk_key in matching_pks {
                    if let Some(entry) = acc.get_mut(&pk_key) {
                        entry.2 += rrf_contribution;
                        entry.3.push("git".to_string());
                    }
                }
            }
        }
    }

    // Collect, sort by score descending, take top `limit`
    let mut results: Vec<RrfSearchResult> = acc
        .into_values()
        .map(|(name, entity_pk, score, signals)| RrfSearchResult {
            entity_pk,
            name,
            score,
            signals,
        })
        .collect();

    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.name.cmp(&b.name))
    });

    results.truncate(limit);
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pk(path: &str, start: i32, end: i32) -> EntityPrimaryKeyLocation {
        EntityPrimaryKeyLocation {
            file_path: path.to_string(),
            start_line: start,
            end_line: end,
        }
    }

    #[test]
    fn test_rrf_entity_a_highest() {
        // Entity "A" is #1 in all 4 signals
        let pk_a = make_pk("a.rs", 1, 10);
        let pk_b = make_pk("b.rs", 1, 10);
        let pk_c = make_pk("c.rs", 1, 10);

        let fts = vec![
            ("A".to_string(), pk_a.clone(), 1.0),
            ("B".to_string(), pk_b.clone(), 0.5),
        ];
        let trie = vec![
            ("A".to_string(), pk_a.clone(), 1.0),
            ("C".to_string(), pk_c.clone(), 0.5),
        ];
        let trigram = vec![
            ("A".to_string(), pk_a.clone(), 1.0),
            ("B".to_string(), pk_b.clone(), 0.3),
        ];

        let mut git_scores = HashMap::new();
        git_scores.insert("a.rs".to_string(), 1.0);
        git_scores.insert("b.rs".to_string(), 0.5);

        let results = combine_rrf_search_signals(&fts, &trie, &trigram, &git_scores, 10);

        assert!(!results.is_empty(), "expected non-empty results");

        // "A" should be first
        assert_eq!(results[0].name, "A", "expected entity A to be ranked #1");

        // Hand-calculate expected score for A (rank 1 in all 4 signals):
        // 0.35/(60+1) + 0.30/(60+1) + 0.20/(60+1) + 0.15/(60+1) = 1.0/61
        let expected = 1.0 / 61.0;
        let tolerance = 1e-10;
        assert!(
            (results[0].score - expected).abs() < tolerance,
            "expected score ~{:.6}, got {:.6}",
            expected,
            results[0].score
        );

        println!("RRF results:");
        for r in &results {
            println!(
                "  name={}, score={:.6}, signals={:?}",
                r.name, r.score, r.signals
            );
        }
    }

    #[test]
    fn test_rrf_empty_signals() {
        let results = combine_rrf_search_signals(&[], &[], &[], &HashMap::new(), 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_rrf_single_signal() {
        let pk_a = make_pk("a.rs", 1, 10);
        let fts = vec![("A".to_string(), pk_a.clone(), 1.0)];

        let results = combine_rrf_search_signals(&fts, &[], &[], &HashMap::new(), 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "A");

        // Score should be W_FTS / (k + 1) = 0.35 / 61
        let expected = W_FTS / (RRF_K + 1.0);
        assert!(
            (results[0].score - expected).abs() < 1e-10,
            "expected {expected}, got {}",
            results[0].score
        );
    }

    #[test]
    fn test_rrf_limit() {
        let entries: Vec<(String, EntityPrimaryKeyLocation, f64)> = (0..20)
            .map(|i| {
                (
                    format!("entity_{i}"),
                    make_pk(&format!("file_{i}.rs"), 1, 10),
                    1.0 - (i as f64 * 0.01),
                )
            })
            .collect();

        let results = combine_rrf_search_signals(&entries, &[], &[], &HashMap::new(), 5);
        assert_eq!(results.len(), 5);
    }
}
