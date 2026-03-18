//! Symbol trie index for exact and prefix-based entity name lookups.

use std::collections::HashMap;

use crate::entity_type_definitions::EntityPrimaryKeyLocation;

/// A prefix-based index for entity names.
///
/// Stores every prefix of each entity name (and sub-words from camelCase/snake_case splitting)
/// in a `HashMap` for O(1) prefix lookup.
pub struct SymbolTrieExactIndex {
    /// Maps each prefix string to a list of (name, pk) pairs.
    prefix_map: HashMap<String, Vec<(String, EntityPrimaryKeyLocation)>>,
}

impl SymbolTrieExactIndex {
    /// Build the index from a list of `(name, pk)` pairs.
    ///
    /// For each name:
    /// - Inserts the lowercased name at every prefix from length 1..=name.len()
    /// - Splits camelCase and snake_case names into sub-words and indexes those too.
    pub fn build_from_entity_names(entities: &[(String, EntityPrimaryKeyLocation)]) -> Self {
        let mut prefix_map: HashMap<String, Vec<(String, EntityPrimaryKeyLocation)>> =
            HashMap::new();

        for (name, pk) in entities {
            let lower = name.to_lowercase();
            // Index all prefixes of the full lowercased name
            Self::index_prefixes(&mut prefix_map, &lower, name, pk);

            // Split into sub-words and index each sub-word's prefixes
            let sub_words = split_into_sub_words(&lower);
            for sub_word in &sub_words {
                if sub_word != &lower {
                    Self::index_prefixes(&mut prefix_map, sub_word, name, pk);
                }
            }
        }

        Self { prefix_map }
    }

    /// Insert all prefixes (1..=word.len()) of `word` into the prefix map,
    /// associating them with the original `name` and `pk`.
    fn index_prefixes(
        prefix_map: &mut HashMap<String, Vec<(String, EntityPrimaryKeyLocation)>>,
        word: &str,
        name: &str,
        pk: &EntityPrimaryKeyLocation,
    ) {
        for end in 1..=word.len() {
            // Ensure we split on a char boundary
            if !word.is_char_boundary(end) {
                continue;
            }
            let prefix = &word[..end];
            let entry = prefix_map.entry(prefix.to_string()).or_default();
            // Avoid duplicate (name, pk) pairs for the same prefix
            if !entry.iter().any(|(n, p)| n == name && p == pk) {
                entry.push((name.to_string(), pk.clone()));
            }
        }
    }

    /// Search the index for exact and prefix matches.
    ///
    /// - Exact matches (query == lowercased name) get score 1.0
    /// - Prefix matches get score 0.5
    ///
    /// Results are sorted by score descending, then by name, and limited to `limit`.
    pub fn search_exact_and_prefix(
        &self,
        query: &str,
        limit: usize,
    ) -> Vec<(String, EntityPrimaryKeyLocation, f64)> {
        let lower_query = query.to_lowercase();

        let entries = match self.prefix_map.get(&lower_query) {
            Some(entries) => entries,
            None => return Vec::new(),
        };

        let mut results: Vec<(String, EntityPrimaryKeyLocation, f64)> = entries
            .iter()
            .map(|(name, pk)| {
                let score = if name.to_lowercase() == lower_query {
                    1.0
                } else {
                    0.5
                };
                (name.clone(), pk.clone(), score)
            })
            .collect();

        // Sort by score descending, then by name ascending for determinism
        results.sort_by(|a, b| {
            b.2.partial_cmp(&a.2)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });

        results.truncate(limit);
        results
    }
}

/// Split a lowercased name into sub-words by underscores and camelCase boundaries.
fn split_into_sub_words(name: &str) -> Vec<String> {
    let mut words = Vec::new();

    // First split by underscores
    for segment in name.split('_') {
        if segment.is_empty() {
            continue;
        }
        // Then split camelCase within each segment
        let camel_words = split_camel_case(segment);
        words.extend(camel_words);
    }

    words
}

/// Split a string at camelCase boundaries.
///
/// E.g. "calculatorAdd" -> ["calculator", "add"]
fn split_camel_case(s: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();

    for ch in s.chars() {
        if ch.is_uppercase() && !current.is_empty() {
            words.push(current.clone());
            current.clear();
        }
        current.push(ch.to_lowercase().next().unwrap_or(ch));
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
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
    fn test_search_exact_match_add() {
        let index = SymbolTrieExactIndex::build_from_entity_names(&test_entities());
        let results = index.search_exact_and_prefix("add", 10);
        assert!(!results.is_empty());
        // "add" should be an exact match with score 1.0
        let exact = results.iter().find(|(name, _, _)| name == "add");
        assert!(exact.is_some(), "expected exact match for 'add'");
        assert_eq!(exact.unwrap().2, 1.0);
    }

    #[test]
    fn test_search_prefix_match_calc() {
        let index = SymbolTrieExactIndex::build_from_entity_names(&test_entities());
        let results = index.search_exact_and_prefix("calc", 10);
        assert!(!results.is_empty());
        // "calculator_add" should appear as a prefix match
        let prefix_match = results
            .iter()
            .find(|(name, _, _)| name == "calculator_add");
        assert!(
            prefix_match.is_some(),
            "expected prefix match for 'calculator_add'"
        );
        assert_eq!(prefix_match.unwrap().2, 0.5);
    }

    #[test]
    fn test_search_no_match_zzz() {
        let index = SymbolTrieExactIndex::build_from_entity_names(&test_entities());
        let results = index.search_exact_and_prefix("zzz", 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_sub_word_indexing() {
        let index = SymbolTrieExactIndex::build_from_entity_names(&test_entities());
        // "login" is a sub-word of "auth_login"
        let results = index.search_exact_and_prefix("login", 10);
        let found = results.iter().find(|(name, _, _)| name == "auth_login");
        assert!(found.is_some(), "expected sub-word match for 'login' in 'auth_login'");
    }

    #[test]
    fn test_split_camel_case() {
        assert_eq!(
            split_camel_case("calculatorAdd"),
            vec!["calculator", "add"]
        );
        assert_eq!(split_camel_case("simple"), vec!["simple"]);
        assert_eq!(
            split_camel_case("myBigFunction"),
            vec!["my", "big", "function"]
        );
    }

    #[test]
    fn test_split_into_sub_words() {
        assert_eq!(
            split_into_sub_words("auth_login"),
            vec!["auth", "login"]
        );
        assert_eq!(
            split_into_sub_words("calculator_add"),
            vec!["calculator", "add"]
        );
    }
}
