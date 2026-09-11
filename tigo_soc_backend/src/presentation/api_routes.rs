// * * * CONTROLADORES DE RUTAS Y RESPUESTAS REST (ENCODING UNIFICADO EN CAMELCASE) * * *

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde_json::json;
use sqlx::PgPool;

use crate::domain::threat_detector::ThreatDetector;
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
    pub threat_detector: Option<ThreatDetector>,
}

// * * * CONFIGURACIÓN DEL ENRUTADOR PRINCIPAL DE LA API * * *
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check_handler))
        .route("/api/v1/status", get(system_status_handler))
        .route("/api/v1/catalogs/device-types", get(get_device_types_handler))
        .route("/api/v1/inventory/nodes", get(get_nodes_handler))
        .route("/api/v1/telemetry/metrics", get(get_metrics_handler))
        .route("/api/v1/telemetry/logs", get(get_logs_handler))
        .route("/api/v1/alerts", get(get_alerts_handler))
        .route("/api/v1/ml/stats", get(get_ml_stats_handler))
        .route("/api/v1/ml/model-info", get(get_model_info_handler))
        .with_state(state)
}

// * * * VERIFICACIÓN DE SALUD DEL SISTEMA (HEALTH CHECK) * * *
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
            "databaseConnected": db_ok,
            "architecture": "Layer 3 Clean Architecture",
            "service": "Tigo SOC Backend"
        })),
    )
}

// * * * ESTADO GLOBAL Y METADATOS DEL MOTOR * * *
async fn system_status_handler() -> impl IntoResponse {
    Json(json!({
        "status": "online",
        "environment": "GNS3 Virtual Lab",
        "snifferEngine": "libpcap + etherparse (Producer-Consumer)",
        "mlTarget": "LightGBM Anomaly Detection",
        "phase": "Traffic Capture & Telemetry Normalization"
    }))
}

// * * * CONSULTA DE CATÁLOGOS DE DISPOSITIVOS * * *
async fn get_device_types_handler(State(state): State<AppState>) -> impl IntoResponse {
    let repo = CatalogsRepository::new(state.pool);
    match repo.get_device_types().await {
        Ok(items) => (StatusCode::OK, Json(json!({ "deviceTypes": items }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

// * * * CONSULTA DE INVENTARIO Y TOPOLOGÍA DE NODOS * * *
async fn get_nodes_handler(State(state): State<AppState>) -> impl IntoResponse {
    let repo = InventoryRepository::new(state.pool);
    match repo.get_nodes().await {
        Ok(items) => (StatusCode::OK, Json(json!({ "networkNodes": items }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

// * * * CONSULTA DE MÉTRICAS AGREGADAS DE TRÁFICO * * *
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

// * * * CONSULTA DE LOGS FORENSES DE EVENTOS DE RED * * *
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

// * * * CONSULTA DE ALERTAS DE SEGURIDAD GENERADAS * * *
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

// * * * CONSULTA DE TELEMETRÍA Y MÉTRICAS DE RENDIMIENTO ML * * *
async fn get_ml_stats_handler(State(state): State<AppState>) -> impl IntoResponse {
    if let Some(ref detector) = state.threat_detector {
        let stats = detector.get_performance_stats();
        (StatusCode::OK, Json(json!({ "mlStats": stats })))
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "error": "Motor LightGBM no inicializado" })),
        )
    }
}

// * * * CONSULTA DE INFORMACIÓN Y ESTADÍSTICAS DEL MODELO K-FOLD * * *
async fn get_model_info_handler() -> impl IntoResponse {
    let paths = [
        "models/training_stats.json",
        "/app/models/training_stats.json",
        "../ModelTrainedLightGBM/core_engine/models/training_stats.json",
    ];

    for p in paths {
        if let Ok(content) = std::fs::read_to_string(p) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                return (StatusCode::OK, Json(val));
            }
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "modelType": "LightGBM GBDT (23 features)",
            "validation": "K-Fold Cross-Validation (k=5)",
            "framework": "Polars + lightgbm3 (Rust)",
            "status": "Loaded and Active"
        })),
    )
}
