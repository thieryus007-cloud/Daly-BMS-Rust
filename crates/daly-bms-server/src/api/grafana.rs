//! Proxy Grafana — Contournement CORS
//!
//! Routes :
//! GET /api/v1/grafana/render/{dashboard_uid}
//! GET /api/v1/grafana/api/*
//! GET /api/v1/grafana/d/{uid}

use axum::{
    extract::{Path, RawQuery},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Response},
    body::Body,
};
use std::sync::Arc;

/// URL de base Grafana (localhost:3001 sur Pi5)
const GRAFANA_URL: &str = "http://localhost:3001";

/// Proxy GET vers Grafana (évite les problèmes CORS)
pub async fn proxy_grafana_api(
    Path(path): Path<String>,
    RawQuery(query): RawQuery,
    headers: HeaderMap,
) -> Response {
    let query_str = query.map(|q| format!("?{}", q)).unwrap_or_default();
    let url = format!("{}/api/{}{}", GRAFANA_URL, path, query_str);

    proxy_request(&url, &headers).await
}

/// Proxy pour les dashboards
pub async fn proxy_grafana_dashboard(
    Path(uid): Path<String>,
    _headers: HeaderMap,
) -> Response {
    let url = format!("{}/d/{}", GRAFANA_URL, uid);
    proxy_request(&url, &_headers).await
}

/// Proxy pour les rendus (images)
pub async fn proxy_grafana_render(
    Path(path): Path<String>,
    RawQuery(query): RawQuery,
    headers: HeaderMap,
) -> Response {
    let query_str = query.map(|q| format!("?{}", q)).unwrap_or_default();
    let url = format!("{}/render/{}{}", GRAFANA_URL, path, query_str);

    proxy_request(&url, &headers).await
}

async fn proxy_request(url: &str, _headers: &HeaderMap) -> Response {
    match reqwest::Client::new().get(url).send().await {
        Ok(resp) => {
            let status = resp.status();
            let body = match resp.bytes().await {
                Ok(bytes) => Body::from(bytes),
                Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            };

            (status, body).into_response()
        }
        Err(e) => {
            eprintln!("Grafana proxy error: {}", e);
            StatusCode::BAD_GATEWAY.into_response()
        }
    }
}
