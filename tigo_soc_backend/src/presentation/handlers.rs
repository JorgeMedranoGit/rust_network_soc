// * * * CONTROLADORES Y HANDLERS DE LA API REST (ARQUITECTURA LIMPIA) * * *

use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;

use crate::infrastructure::{
    repo_catalogs::CatalogsRepository,
    repo_inventory::InventoryRepository,
    repo_orchestration::OrchestrationRepository,
    repo_telemetry::TelemetryRepository,
};
use crate::presentation::api_routes::AppState;

// * * * 1. ENDPOINT LIGERO DE SALUD (HEALTH CHECK) * * *
// Devuelve HTTP 200 y JSON {"status": "online", "service": "Tigo SOC API"}
pub async fn health_check_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "online",
            "service": "Tigo SOC API"
        })),
    )
}

// * * * 2. ENDPOINT DE ALERTAS RECIENTES (ÚLTIMAS 50) * * *
// Consulta a PostgreSQL devolviendo un arreglo JSON con las últimas 50 alertas detectadas
pub async fn get_alerts_handler(State(state): State<AppState>) -> impl IntoResponse {
    let repo = OrchestrationRepository::new(state.pool);
    match repo.get_recent_enriched_alerts(50).await {
        Ok(items) => (StatusCode::OK, Json(json!(items))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

// * * * 3. ENDPOINT DE RESUMEN ESTADÍSTICO DE MÉTRICAS GLOBALES * * *
// Consulta a PostgreSQL devolviendo resumen de amenazas, tráfico procesado y nodos
pub async fn get_metrics_summary_handler(State(state): State<AppState>) -> impl IntoResponse {
    let repo = TelemetryRepository::new(state.pool);
    match repo.get_metrics_summary().await {
        Ok(summary) => (StatusCode::OK, Json(json!(summary))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

// * * * 4. ENDPOINT DE CONSULTA DE ALERTA POR ID * * *
pub async fn get_alert_by_id_handler(
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

// * * * 5. ENDPOINTS DE TELEMETRÍA Y LOGS COMPLEMENTARIOS * * *

pub async fn get_telemetry_metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let repo = TelemetryRepository::new(state.pool);
    match repo.get_recent_metrics(50).await {
        Ok(items) => (StatusCode::OK, Json(json!({ "metrics": items }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn get_logs_handler(State(state): State<AppState>) -> impl IntoResponse {
    let repo = TelemetryRepository::new(state.pool);
    match repo.get_recent_logs(50).await {
        Ok(items) => (StatusCode::OK, Json(json!({ "logs": items }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

// * * * 6. CATÁLOGOS E INVENTARIO TOPOLÓGICO * * *

pub async fn get_device_types_handler(State(state): State<AppState>) -> impl IntoResponse {
    let repo = CatalogsRepository::new(state.pool);
    match repo.get_device_types().await {
        Ok(items) => (StatusCode::OK, Json(json!({ "deviceTypes": items }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn get_nodes_handler(State(state): State<AppState>) -> impl IntoResponse {
    let repo = InventoryRepository::new(state.pool);
    match repo.get_nodes().await {
        Ok(items) => (StatusCode::OK, Json(json!({ "networkNodes": items }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn system_status_handler() -> impl IntoResponse {
    Json(json!({
        "status": "online",
        "environment": "GNS3 Virtual Lab",
        "snifferEngine": "libpcap + etherparse (Producer-Consumer)",
        "mlTarget": "LightGBM Anomaly Detection",
        "phase": "Traffic Capture & Telemetry Normalization"
    }))
}

// * * * 7. NOTIFICACIONES PUSH Y SUSCRIPCIONES FCM * * *

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SimulatePushRequest {
    pub alert_id: Option<i64>,
    pub severity: Option<String>,
    pub threat_category: Option<String>,
}

pub async fn simulate_push_handler(
    State(state): State<AppState>,
    Json(payload): Json<SimulatePushRequest>,
) -> impl IntoResponse {
    let severity = payload.severity.unwrap_or_else(|| "CRITICAL".to_string());
    let category = payload.threat_category.unwrap_or_else(|| "DATA_EXFILTRATION".to_string());

    let repo_telemetry = TelemetryRepository::new(state.pool.clone());
    let repo_orch = OrchestrationRepository::new(state.pool.clone());

    let synthetic_features = json!({
        "number": 15.0,
        "rate": 182.4,
        "http": 1.0,
        "tcp": 1.0,
        "udp": 0.0,
        "min_size": 64.0,
        "max_size": 1400.0,
        "avg_size": 890.5,
        "std_size": 220.1,
        "syn_count": 15.0,
        "ack_count": 0.0,
        "simulation": true
    });

    let alert_id = if let Ok(log_id) = repo_telemetry
        .insert_log(
            Some(6),
            "192.168.1.50",
            "192.168.1.10",
            "TCP",
            1400,
            Some("SYN"),
        )
        .await
    {
        if let Ok(feature_id) = repo_telemetry.insert_features(log_id, synthetic_features).await {
            repo_orch
                .create_alert(Some(feature_id), Some(1), Some(1), 0.982)
                .await
                .unwrap_or_else(|_| payload.alert_id.unwrap_or(999))
        } else {
            payload.alert_id.unwrap_or(999)
        }
    } else {
        payload.alert_id.unwrap_or(999)
    };

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

#[derive(Deserialize, Debug)]
pub struct SubscribeTopicRequest {
    pub token: String,
}

pub async fn subscribe_topic_handler(
    State(state): State<AppState>,
    Json(payload): Json<SubscribeTopicRequest>,
) -> impl IntoResponse {
    let fcm = Arc::clone(&state.fcm_notifier);
    match fcm.subscribe_device_to_topic(&payload.token).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "message": "Dispositivo suscrito exitosamente al tema /topics/soc_alerts",
                "topic": "/topics/soc_alerts"
            })),
        ),
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({
                "status": "error",
                "message": format!("Fallo al suscribir dispositivo a FCM: {}", e)
            })),
        ),
    }
}

// * * * 8. TELEMETRÍA Y ESTADÍSTICAS DEL MOTOR ML * * *

pub async fn get_ml_stats_handler(State(state): State<AppState>) -> impl IntoResponse {
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

pub async fn get_model_info_handler() -> impl IntoResponse {
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
