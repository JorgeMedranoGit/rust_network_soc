use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

use crate::domain::threat_detector::ThreatDetector;
use crate::infrastructure::{
    fcm_client::FcmNotifier,
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
    pub fcm_notifier: Arc<FcmNotifier>,
}

// * * * CONFIGURACIÓN DEL ENRUTADOR PRINCIPAL DE LA API * * *
pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    let mut router = Router::new()
        .route("/health", get(health_check_handler))
        .route("/api/v1/status", get(system_status_handler))
        .route("/api/v1/catalogs/device-types", get(get_device_types_handler))
        .route("/api/v1/inventory/nodes", get(get_nodes_handler))
        .route("/api/v1/telemetry/metrics", get(get_metrics_handler))
        .route("/api/v1/telemetry/logs", get(get_logs_handler))
        .route("/api/v1/alerts", get(get_alerts_handler))
        .route("/api/v1/alerts/:id", get(get_alert_by_id_handler))
        .route("/api/v1/alerts/simulate-push", post(simulate_push_handler))
        .route("/api/v1/ml/stats", get(get_ml_stats_handler))
        .route("/api/v1/ml/model-info", get(get_model_info_handler));

    // * * * SERVICIO DE ARCHIVOS ESTÁTICOS PWA PARA TÉCNICOS * * *
    let pwa_paths = ["../frontend", "frontend", "/app/frontend"];
    for p in pwa_paths {
        if std::path::Path::new(p).exists() {
            println!("|- INSTRUCCION -| [PWA] Montando interfaz PWA en ruta estática: /pwa desde [{}]", p);
            router = router.nest_service("/pwa", ServeDir::new(p));
            break;
        }
    }

    router.layer(cors).with_state(state)
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
    match repo.get_recent_enriched_alerts(50).await {
        Ok(items) => (StatusCode::OK, Json(json!({ "alerts": items }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

// * * * CONSULTA DE ALERTA INDIVIDUAL CON EVIDENCIA FORENSE ENRIQUECIDA * * *
async fn get_alert_by_id_handler(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let repo = OrchestrationRepository::new(state.pool);
    match repo.get_enriched_alert_by_id(id).await {
        Ok(Some(alert)) => (StatusCode::OK, Json(json!({ "alert": alert }))),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": format!("Alerta con ID #{} no encontrada", id) })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SimulatePushRequest {
    alert_id: Option<i64>,
    severity: Option<String>,
    threat_category: Option<String>,
}

// * * * SIMULADOR DE NOTIFICACIÓN PUSH ZERO-DATA HACIA /topics/soc_alerts * * *
async fn simulate_push_handler(
    State(state): State<AppState>,
    Json(payload): Json<SimulatePushRequest>,
) -> impl IntoResponse {
    let alert_id = payload.alert_id.unwrap_or(999);
    let severity = payload.severity.unwrap_or_else(|| "CRITICAL".to_string());
    let category = payload.threat_category.unwrap_or_else(|| "DATA_EXFILTRATION".to_string());

    let fcm = Arc::clone(&state.fcm_notifier);
    match fcm.send_opaque_alert(alert_id, &severity, &category).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "message": "Zero-Data Push webhook despachado exitosamente",
                "alertId": alert_id,
                "severity": severity,
                "threatCategory": category,
                "topic": "/topics/soc_alerts"
            })),
        ),
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({
                "status": "error",
                "message": format!("Fallo al enviar notificación Push FCM: {}", e)
            })),
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
