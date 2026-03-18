//! Trigram-based fuzzy matching index for entity name search.

use std::collections::{HashMap, HashSet};

use crate::entity_type_definitions::EntityPrimaryKeyLocation;

/// A fuzzy search index based on character trigrams.
///
/// Each entity name is decomposed into overlapping 3-character windows (trigrams),
/// and trigrams map back to entity indices for fast retrieval.
pub struct TrigramFuzzyMatchIndex {
    /// Maps trigram -> list of entity indices that contain this trigram.
    trigram_map: HashMap<String, Vec<usize>>,
    /// The original entity list (name, pk).
    entities: Vec<(String, EntityPrimaryKeyLocation)>,
    /// Precomputed trigram sets per entity (for Jaccard computation).
    entity_trigram_sets: Vec<HashSet<String>>,
}

impl TrigramFuzzyMatchIndex {
    /// Build the trigram index from a list of `(name, pk)` pairs.
    pub fn build_from_entity_names(names: &[(String, EntityPrimaryKeyLocation)]) -> Self {
        let mut trigram_map: HashMap<String, Vec<usize>> = HashMap::new();
        let mut entity_trigram_sets = Vec::with_capacity(names.len());

        for (idx, (name, _pk)) in names.iter().enumerate() {
            let trigrams = compute_trigrams(&name.to_lowercase());
            for tri in &trigrams {
                trigram_map.entry(tri.clone()).or_default().push(idx);
            }
            entity_trigram_sets.push(trigrams);
        }

        Self {
            trigram_map,
            entities: names.to_vec(),
            entity_trigram_sets,
        }
    }

    /// Search for entities whose names fuzzily match `query` using trigram overlap.
    ///
    /// Ranks results by Jaccard similarity: |intersection| / |union| of trigram sets.
    /// Returns the top `limit` results sorted by score descending.
    pub fn search_fuzzy_trigram_match(
        &self,
        query: &str,
        limit: usize,
    ) -> Vec<(String, EntityPrimaryKeyLocation, f64)> {
        let query_trigrams = compute_trigrams(&query.to_lowercase());
        if query_trigrams.is_empty() {
            return Vec::new();
        }

        // Collect candidate entity indices that share at least one trigram
        let mut candidate_counts: HashMap<usize, usize> = HashMap::new();
        for tri in &query_trigrams {
            if let Some(indices) = self.trigram_map.get(tri) {
                for &idx in indices {
                    *candidate_counts.entry(idx).or_insert(0) += 1;
                }
            }
        }

        // Compute Jaccard similarity for each candidate
        let mut scored: Vec<(String, EntityPrimaryKeyLocation, f64)> = candidate_counts
            .into_iter()
            .map(|(idx, shared_count)| {
                let entity_trigrams = &self.entity_trigram_sets[idx];
                let intersection = shared_count;
                let union = query_trigrams.len() + entity_trigrams.len() - intersection;
                let jaccard = if union > 0 {
                    intersection as f64 / union as f64
                } else {
                    0.0
                };
                let (name, pk) = &self.entities[idx];
                (name.clone(), pk.clone(), jaccard)
            })
            .collect();

        // Sort by score descending, then name ascending for determinism
        scored.sort_by(|a, b| {
            b.2.partial_cmp(&a.2)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });

        scored.truncate(limit);
        scored
    }
}

/// Compute the set of character trigrams from a string.
///
/// A trigram is a sliding window of 3 consecutive characters.
/// Returns a `HashSet` of unique trigrams.
fn compute_trigrams(s: &str) -> HashSet<String> {
    let chars: Vec<char> = s.chars().collect();
    let mut trigrams = HashSet::new();
    if chars.len() < 3 {
        // For very short strings, use what we have as a single "trigram"
        if !chars.is_empty() {
            trigrams.insert(chars.iter().collect::<String>());
        }
        return trigrams;
    }
    for window in chars.windows(3) {
        trigrams.insert(window.iter().collect::<String>());
    }
    trigrams
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

    fn test_entities() -> Vec<(String, EntityPrimaryKeyLocation)> {
        vec![
            ("add".to_string(), make_pk("math.rs", 1, 5)),
            ("subtract".to_string(), make_pk("math.rs", 7, 12)),
            ("multiply".to_string(), make_pk("math.rs", 14, 20)),
            (
                "calculator_add".to_string(),
                make_pk("calc.rs", 1, 10),
            ),
            ("auth_login".to_string(), make_pk("auth.rs", 1, 15)),
        ]
    }

    #[test]
    fn test_trigram_search_authn() {
        let index = TrigramFuzzyMatchIndex::build_from_entity_names(&test_entities());
        let results = index.search_fuzzy_trigram_match("authn", 10);
        // "auth_login" shares trigrams with "authn" ("aut", "uth")
        let auth_match = results.iter().find(|(name, _, _)| name == "auth_login");
        assert!(
            auth_match.is_some(),
            "expected auth_login to match 'authn', got: {:?}",
            results
        );
        // Should have a reasonably high score
        assert!(
            auth_match.unwrap().2 > 0.0,
            "expected positive Jaccard score"
        );
    }

    #[test]
    fn test_trigram_search_xyz_empty() {
        let index = TrigramFuzzyMatchIndex::build_from_entity_names(&test_entities());
        let results = index.search_fuzzy_trigram_match("xyz", 10);
        // "xyz" shares no trigrams with any entity
        assert!(
            results.is_empty() || results.iter().all(|(_, _, score)| *score < 0.1),
            "expected empty or very low scores for 'xyz', got: {:?}",
            results
        );
    }

    #[test]
    fn test_trigram_empty_query() {
        let index = TrigramFuzzyMatchIndex::build_from_entity_names(&test_entities());
        let results = index.search_fuzzy_trigram_match("", 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_compute_trigrams() {
        let trigrams = compute_trigrams("hello");
        assert!(trigrams.contains("hel"));
        assert!(trigrams.contains("ell"));
        assert!(trigrams.contains("llo"));
        assert_eq!(trigrams.len(), 3);
    }

    #[test]
    fn test_compute_trigrams_short() {
        let trigrams = compute_trigrams("ab");
        assert_eq!(trigrams.len(), 1);
        assert!(trigrams.contains("ab"));
    }
}
