//! Route definition builder: constructs the Axum router with all 20 endpoints.

use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use tower_http::cors::{Any, CorsLayer};

use crate::handlers::api_docs_handler::handle_api_docs;
use crate::handlers::centrality_handler::handle_centrality;
use crate::handlers::circular_deps_handler::handle_circular_deps;
use crate::handlers::ck_metrics_handler::handle_ck_metrics;
use crate::handlers::cluster_handler::handle_clusters;
use crate::handlers::coverage_handler::handle_coverage;
use crate::handlers::edge_list_handler::handle_edge_list;
use crate::handlers::entity_detail_handler::handle_entity_detail;
use crate::handlers::entity_list_handler::handle_entity_list;
use crate::handlers::entropy_handler::handle_entropy;
use crate::handlers::health_check_handler::handle_health_check;
use crate::handlers::hotspots_handler::handle_hotspots;
use crate::handlers::journey_handler::handle_journey;
use crate::handlers::kcore_handler::handle_kcore;
use crate::handlers::leiden_handler::handle_leiden;
use crate::handlers::scc_handler::handle_scc;
use crate::handlers::search_handler::handle_fuzzy_search;
use crate::handlers::smart_context_handler::handle_smart_context;
use crate::handlers::sqale_handler::handle_sqale;
use crate::handlers::statistics_handler::handle_statistics;
use crate::shared_server_app_state::SharedServerAppState;

/// Build the complete Axum router with all 20 API routes and CORS middleware.
pub fn build_route_definitions(state: Arc<SharedServerAppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/server-health-check-status", get(handle_health_check))
        .route(
            "/codebase-statistics-overview-summary",
            get(handle_statistics),
        )
        .route(
            "/api-reference-documentation-help",
            get(handle_api_docs),
        )
        .route("/code-entities-list-all", get(handle_entity_list))
        .route("/code-entities-search-fuzzy", get(handle_fuzzy_search))
        .route("/code-entity-detail-view/{pk}", get(handle_entity_detail))
        .route("/dependency-edges-list-all", get(handle_edge_list))
        .route(
            "/strongly-connected-components-analysis",
            get(handle_scc),
        )
        .route(
            "/leiden-community-detection-clusters",
            get(handle_leiden),
        )
        .route(
            "/centrality-measures-entity-ranking",
            get(handle_centrality),
        )
        .route(
            "/kcore-decomposition-layering-analysis",
            get(handle_kcore),
        )
        .route(
            "/entropy-complexity-measurement-scores",
            get(handle_entropy),
        )
        .route("/coupling-cohesion-metrics-suite", get(handle_ck_metrics))
        .route("/technical-debt-sqale-scoring", get(handle_sqale))
        .route(
            "/ingestion-coverage-folder-report",
            get(handle_coverage),
        )
        .route("/smart-context-token-budget", get(handle_smart_context))
        .route(
            "/complexity-hotspots-ranking-view",
            get(handle_hotspots),
        )
        .route(
            "/circular-dependency-detection-scan",
            get(handle_circular_deps),
        )
        .route("/semantic-cluster-grouping-list", get(handle_clusters))
        .route("/query", get(handle_journey))
        .layer(cors)
        .with_state(state)
}
