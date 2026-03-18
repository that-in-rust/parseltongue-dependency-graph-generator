//! API documentation endpoint: returns a listing of all available endpoints.

use axum::Json;
use serde_json::{json, Value};

/// GET /api-reference-documentation-help
pub async fn handle_api_docs() -> Json<Value> {
    Json(json!({
        "version": "3.0.0",
        "endpoints": [
            { "path": "/server-health-check-status", "method": "GET", "description": "Health check – returns status and version" },
            { "path": "/codebase-statistics-overview-summary", "method": "GET", "description": "Entity and edge count summary" },
            { "path": "/api-reference-documentation-help", "method": "GET", "description": "This endpoint – API reference listing" },
            { "path": "/code-entities-list-all", "method": "GET", "description": "List all code entities" },
            { "path": "/code-entities-search-fuzzy", "method": "GET", "description": "Fuzzy RRF search across entities (?q=query)" },
            { "path": "/code-entity-detail-view/{pk}", "method": "GET", "description": "Detail view for a single entity by PK" },
            { "path": "/dependency-edges-list-all", "method": "GET", "description": "List all dependency edges" },
            { "path": "/strongly-connected-components-analysis", "method": "GET", "description": "Tarjan SCC analysis" },
            { "path": "/leiden-community-detection-clusters", "method": "GET", "description": "Leiden community clustering" },
            { "path": "/centrality-measures-entity-ranking", "method": "GET", "description": "PageRank or betweenness centrality (?method=pagerank|betweenness)" },
            { "path": "/kcore-decomposition-layering-analysis", "method": "GET", "description": "K-core decomposition (?k=N optional filter)" },
            { "path": "/entropy-complexity-measurement-scores", "method": "GET", "description": "Shannon entropy per entity" },
            { "path": "/coupling-cohesion-metrics-suite", "method": "GET", "description": "CK metrics (CBO, LCOM, RFC, WMC)" },
            { "path": "/technical-debt-sqale-scoring", "method": "GET", "description": "SQALE debt scores and ratings" },
            { "path": "/ingestion-coverage-folder-report", "method": "GET", "description": "Folder coverage report (?depth=N)" },
            { "path": "/smart-context-token-budget", "method": "GET", "description": "Smart context within token budget (?tokens=N)" },
            { "path": "/complexity-hotspots-ranking-view", "method": "GET", "description": "Top complexity hotspots (?top=N)" },
            { "path": "/circular-dependency-detection-scan", "method": "GET", "description": "Circular dependency scan (SCCs with size > 1)" },
            { "path": "/semantic-cluster-grouping-list", "method": "GET", "description": "Semantic cluster groupings via Leiden" },
            { "path": "/query", "method": "GET", "description": "7-event journey endpoint (?q=query&cluster=N)" },
        ]
    }))
}
