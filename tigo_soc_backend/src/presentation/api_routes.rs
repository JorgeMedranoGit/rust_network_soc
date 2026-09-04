/// * * * MODULO PARA CARGAR LAS RUTAS * * *
use axum::{routing::get, Router};

pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(health_check_handler))
        .route("/api/v1/status", get(system_status_handler))
}

async fn health_check_handler() -> &'static str {
    "Tigo SOC Backend - Status: Operational (Layer 3 Architecture Active)"
}

async fn system_status_handler() -> &'static str {
    r#"{"status": "online", "environment": "GNS3 Virtual Lab", "phase": "Week 3 Complete"}"#
}