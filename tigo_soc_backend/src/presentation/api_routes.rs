use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde_json::json;
use sqlx::PgPool;

use crate::infrastructure::{
    repo_catalogs::CatalogsRepository,
    repo_inventory::InventoryRepository,
    repo_orchestration::OrchestrationRepository,
    repo_telemetry::TelemetryRepository,
};
use crate::presentation::telemetry_stream::TelemetryStreamHandler;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    #[allow(dead_code)]
    pub stream_handler: TelemetryStreamHandler,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check_handler))
        .route("/api/v1/status", get(system_status_handler))
        .route("/api/v1/catalogs/device-types", get(get_device_types_handler))
        .route("/api/v1/inventory/nodes", get(get_nodes_handler))
        .route("/api/v1/telemetry/metrics", get(get_metrics_handler))
        .route("/api/v1/telemetry/logs", get(get_logs_handler))
        .route("/api/v1/alerts", get(get_alerts_handler))
        .with_state(state)
}

async fn health_check_handler(State(state): State<AppState>) -> impl IntoResponse {
    let db_ok = sqlx::query("SELECT 1")
        .execute(&state.pool)
        .await
        .is_ok();

    let status = if db_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status,
        Json(json!({
            "status": if db_ok { "operational" } else { "degraded" },
            "database_connected": db_ok,
            "architecture": "Layer 3 Clean Architecture",
            "service": "Tigo SOC Backend"
        })),
    )
}

async fn system_status_handler() -> impl IntoResponse {
    Json(json!({
        "status": "online",
        "environment": "GNS3 Virtual Lab",
        "sniffer_engine": "libpcap + etherparse (Producer-Consumer)",
        "ml_target": "LightGBM Anomaly Detection",
        "phase": "Traffic Capture & Telemetry Normalization"
    }))
}

async fn get_device_types_handler(State(state): State<AppState>) -> impl IntoResponse {
    let repo = CatalogsRepository::new(state.pool);
    match repo.get_device_types().await {
        Ok(items) => (StatusCode::OK, Json(json!({ "device_types": items }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn get_nodes_handler(State(state): State<AppState>) -> impl IntoResponse {
    let repo = InventoryRepository::new(state.pool);
    match repo.get_nodes().await {
        Ok(items) => (StatusCode::OK, Json(json!({ "network_nodes": items }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn get_metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let repo = TelemetryRepository::new(state.pool);
    match repo.get_recent_metrics(50).await {
        Ok(items) => (StatusCode::OK, Json(json!({ "metrics": items }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn get_logs_handler(State(state): State<AppState>) -> impl IntoResponse {
    let repo = TelemetryRepository::new(state.pool);
    match repo.get_recent_logs(50).await {
        Ok(items) => (StatusCode::OK, Json(json!({ "logs": items }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn get_alerts_handler(State(state): State<AppState>) -> impl IntoResponse {
    let repo = OrchestrationRepository::new(state.pool);
    match repo.get_recent_alerts(50).await {
        Ok(items) => (StatusCode::OK, Json(json!({ "alerts": items }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}
