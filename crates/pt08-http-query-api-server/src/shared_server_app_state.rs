//! Shared application state for the HTTP server.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use parseltongue_core::entity_type_definitions::CodeEntityRecord;
use parseltongue_core::graph::adjacency_list_graph::AdjacencyListGraphRepresentation;
use parseltongue_core::search::symbol_trie_exact_index::SymbolTrieExactIndex;
use parseltongue_core::search::trigram_fuzzy_match_index::TrigramFuzzyMatchIndex;
use parseltongue_core::storage::turso_storage_client::TursoStorageClient;

/// Shared state passed to every Axum handler via `State<Arc<SharedServerAppState>>`.
pub struct SharedServerAppState {
    pub store: TursoStorageClient,
    pub graph: AdjacencyListGraphRepresentation,
    pub symbol_trie: SymbolTrieExactIndex,
    pub trigram_index: TrigramFuzzyMatchIndex,
    pub git_recency: HashMap<String, f64>,
    pub codebase_id: i64,
    pub entity_visibility: HashMap<String, bool>,
    /// Cache of entities keyed by PK string for fast lookups.
    pub entity_wc_map: HashMap<String, u32>,
}

/// Build the shared state from a database path.
///
/// This loads all entities and edges from the database, builds the graph,
/// search indexes, and entity visibility map.
pub async fn build_server_app_state(db_path: &str) -> Result<Arc<SharedServerAppState>> {
    let store = TursoStorageClient::open_database_connection_path(db_path).await?;

    // Use the first existing codebase if one exists, otherwise create a "default" entry.
    let codebase_id = match store.get_first_codebase_id().await? {
        Some(id) => id,
        None => store.get_or_create_codebase_entry("default").await?,
    };

    let entities = store.get_all_entity_records(codebase_id).await?;
    let edges = store.get_all_edge_records(codebase_id).await?;

    // Build graph from edges
    let graph = AdjacencyListGraphRepresentation::build_from_edge_records(&edges);

    // Build search indexes
    let entity_names: Vec<_> = entities
        .iter()
        .filter(|e| e.entity_type.is_searchable())
        .map(|e| (e.name.clone(), e.pk.clone()))
        .collect();
    let symbol_trie = SymbolTrieExactIndex::build_from_entity_names(&entity_names);
    let trigram_index = TrigramFuzzyMatchIndex::build_from_entity_names(&entity_names);

    // Entity visibility (all public by default, mark private if name starts with _)
    let entity_visibility: HashMap<String, bool> = entities
        .iter()
        .map(|e| (e.pk.to_string(), !e.name.starts_with('_')))
        .collect();

    // Word-count map for token budgets
    let entity_wc_map: HashMap<String, u32> = entities
        .iter()
        .map(|e| (e.pk.to_string(), e.wc))
        .collect();

    Ok(Arc::new(SharedServerAppState {
        store,
        graph,
        symbol_trie,
        trigram_index,
        git_recency: HashMap::new(),
        codebase_id,
        entity_visibility,
        entity_wc_map,
    }))
}

/// Build a minimal in-memory state for testing (no database load).
pub async fn build_test_server_app_state(
    entities: &[CodeEntityRecord],
    edges: &[parseltongue_core::entity_type_definitions::EdgeRecord],
    store: TursoStorageClient,
    codebase_id: i64,
) -> Arc<SharedServerAppState> {
    let graph = AdjacencyListGraphRepresentation::build_from_edge_records(edges);

    let entity_names: Vec<_> = entities
        .iter()
        .filter(|e| e.entity_type.is_searchable())
        .map(|e| (e.name.clone(), e.pk.clone()))
        .collect();
    let symbol_trie = SymbolTrieExactIndex::build_from_entity_names(&entity_names);
    let trigram_index = TrigramFuzzyMatchIndex::build_from_entity_names(&entity_names);

    let entity_visibility: HashMap<String, bool> = entities
        .iter()
        .map(|e| (e.pk.to_string(), !e.name.starts_with('_')))
        .collect();

    let entity_wc_map: HashMap<String, u32> = entities
        .iter()
        .map(|e| (e.pk.to_string(), e.wc))
        .collect();

    Arc::new(SharedServerAppState {
        store,
        graph,
        symbol_trie,
        trigram_index,
        git_recency: HashMap::new(),
        codebase_id,
        entity_visibility,
        entity_wc_map,
    })
}
