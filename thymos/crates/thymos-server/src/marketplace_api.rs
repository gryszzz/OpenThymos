//! HTTP endpoints for the tool marketplace.
//!
//! Routes:
//!   GET    /marketplace/packages         — list all packages (latest versions)
//!   GET    /marketplace/packages/:name   — get a specific package
//!   POST   /marketplace/packages         — publish a package
//!   DELETE /marketplace/packages/:name/:version — unpublish a version
//!   GET    /marketplace/search?q=...&tag=...&kind=... — search packages

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;
use std::sync::Arc;

use thymos_marketplace::{MarketplaceService, Package, SearchQuery};

/// Shared marketplace state.
pub type MarketplaceState = Arc<MarketplaceService>;

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: Option<String>,
    pub tag: Option<String>,
    pub kind: Option<String>,
    pub author: Option<String>,
}

/// GET /marketplace/packages — list all packages.
pub async fn list_packages(State(mp): State<MarketplaceState>) -> impl IntoResponse {
    let packages = mp.list();
    let json: Vec<serde_json::Value> = packages
        .iter()
        .map(|p| serde_json::to_value(p).unwrap())
        .collect();
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "packages": json,
            "total": mp.total_packages(),
        })),
    )
}

/// GET /marketplace/packages/:name — get a specific package (latest version).
pub async fn get_package(
    State(mp): State<MarketplaceState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match mp.get(&name, None) {
        Ok(pkg) => (StatusCode::OK, Json(serde_json::to_value(pkg).unwrap())).into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// POST /marketplace/packages — publish a package.
pub async fn publish_package(
    State(mp): State<MarketplaceState>,
    Json(pkg): Json<Package>,
) -> impl IntoResponse {
    match mp.publish(pkg) {
        Ok(()) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "status": "published" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// DELETE /marketplace/packages/:name/:version — unpublish a version.
pub async fn unpublish_package(
    State(mp): State<MarketplaceState>,
    Path((name, version)): Path<(String, String)>,
) -> impl IntoResponse {
    match mp.unpublish(&name, &version) {
        Ok(pkg) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "unpublished",
                "name": pkg.name,
                "version": pkg.version,
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// GET /marketplace/search?q=...&tag=...&kind=...&author=...
pub async fn search_packages(
    State(mp): State<MarketplaceState>,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let query = SearchQuery {
        text: params.q,
        tags: params.tag.map(|t| vec![t]).unwrap_or_default(),
        author: params.author,
        kind: params.kind,
    };
    let results: Vec<serde_json::Value> = mp
        .search(&query)
        .iter()
        .map(|p| serde_json::to_value(p).unwrap())
        .collect();
    let count = results.len();
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "results": results,
            "count": count,
        })),
    )
}

/// Build the marketplace sub-router.
pub fn marketplace_router(state: MarketplaceState) -> axum::Router<()> {
    use axum::routing::{delete, get};
    axum::Router::new()
        .route(
            "/marketplace/packages",
            get(list_packages).post(publish_package),
        )
        .route("/marketplace/packages/{name}", get(get_package))
        .route(
            "/marketplace/packages/{name}/{version}",
            delete(unpublish_package),
        )
        .route("/marketplace/search", get(search_packages))
        .with_state(state)
}
