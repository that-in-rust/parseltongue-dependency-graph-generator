//! Query preprocessing: tokenize, filter stopwords, deduplicate, and limit terms.

use std::collections::HashSet;

/// Stopwords to filter out during query preprocessing.
const STOPWORDS: &[&str] = &[
    "the", "and", "for", "that", "this", "with", "from", "have", "are", "was",
    "were", "been", "being", "has", "had", "does", "did", "will", "would", "could",
    "should", "might", "shall", "can", "may", "must", "need", "please", "help",
    "find", "show", "get", "want", "look", "search", "where", "what", "which",
    "when", "how", "who", "why", "about", "into", "through", "during", "before",
    "after", "above", "below", "between", "under", "over", "just", "also", "very",
    "really", "quite", "rather", "some", "any", "all", "each", "every", "both",
    "few", "more", "most", "other", "such", "only", "same", "than", "too", "not",
    "but", "its", "our", "your", "their",
];

/// Maximum number of terms returned from preprocessing.
const MAX_TERMS: usize = 12;

/// Minimum token length (inclusive).
const MIN_TOKEN_LEN: usize = 3;

/// Maximum token length (inclusive).
const MAX_TOKEN_LEN: usize = 30;

/// Preprocess a raw query string into a list of search tokens.
///
/// Steps:
/// 1. Lowercase the input
/// 2. Split on non-alphanumeric characters (keeping hyphens/underscores within words)
/// 3. Filter tokens to 3–30 characters
/// 4. Remove stopwords
/// 5. Deduplicate (preserving first-seen order)
/// 6. Limit to 12 terms
pub fn preprocess_query_input_text(raw_query: &str) -> Vec<String> {
    let stopword_set: HashSet<&str> = STOPWORDS.iter().copied().collect();
    let lowered = raw_query.to_lowercase();

    // Split on characters that are NOT alphanumeric, hyphen, or underscore
    let tokens: Vec<&str> = lowered
        .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
        .filter(|s| !s.is_empty())
        .collect();

    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for token in tokens {
        let len = token.len();
        if len < MIN_TOKEN_LEN || len > MAX_TOKEN_LEN {
            continue;
        }
        if stopword_set.contains(token) {
            continue;
        }
        if seen.insert(token.to_string()) {
            result.push(token.to_string());
            if result.len() >= MAX_TERMS {
                break;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess_authentication_flow() {
        let result = preprocess_query_input_text("please help me find the authentication flow");
        assert_eq!(result, vec!["authentication", "flow"]);
    }

    #[test]
    fn test_preprocess_empty() {
        let result = preprocess_query_input_text("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_preprocess_all_stopwords() {
        let result = preprocess_query_input_text("the and for that this with from");
        assert!(result.is_empty());
    }

    #[test]
    fn test_preprocess_deduplication() {
        let result = preprocess_query_input_text("foo bar foo baz bar");
        assert_eq!(result, vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn test_preprocess_max_terms() {
        let input = "aaa bbb ccc ddd eee fff ggg hhh iii jjj kkk lll mmm nnn ooo";
        let result = preprocess_query_input_text(input);
        assert_eq!(result.len(), MAX_TERMS);
    }

    #[test]
    fn test_preprocess_hyphen_underscore_preserved() {
        let result = preprocess_query_input_text("my-func some_var");
        assert!(result.contains(&"my-func".to_string()));
        assert!(result.contains(&"some_var".to_string()));
    }

    #[test]
    fn test_preprocess_short_tokens_filtered() {
        // "me" is 2 chars -> filtered, "ab" is 2 chars -> filtered
        let result = preprocess_query_input_text("me ab hello");
        assert_eq!(result, vec!["hello"]);
    }
}
