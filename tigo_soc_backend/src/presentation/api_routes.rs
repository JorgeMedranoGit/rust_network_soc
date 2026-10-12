// * * * CONFIGURACIÓN DE ENRUTAMIENTO, ESTADO COMPARTIDO Y MIDDLEWARE CORS (AXUM) * * *

use std::sync::Arc;
use axum::{
    extract::FromRef,
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::domain::threat_detector::ThreatDetector;
use crate::infrastructure::fcm_client::FcmNotifier;
use crate::presentation::{
    handlers::{
        get_alert_by_id_handler, get_alerts_handler, get_device_types_handler,
        get_logs_handler, get_metrics_summary_handler, get_ml_stats_handler,
        get_model_info_handler, get_nodes_handler, get_telemetry_metrics_handler,
        health_check_handler, simulate_push_handler, subscribe_topic_handler,
        system_status_handler,
    },
    telemetry_stream::TelemetryStreamHandler,
};

// * * * ESTADO GLOBAL COMPARTIDO INYECTABLE EN CONTROLADORES MEDIANTE STATE * * *
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    #[allow(dead_code)]
    pub stream_handler: TelemetryStreamHandler,
    pub threat_detector: Option<ThreatDetector>,
    pub fcm_notifier: Arc<FcmNotifier>,
}

// * * * Soporte para extraer PgPool directamente mediante State(pool): State<PgPool> * * *
impl FromRef<AppState> for PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

// * * * CONFIGURACIÓN DEL ENRUTADOR PRINCIPAL DE LA API SOC * * *
pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let mut router = Router::new()
        .route("/health", get(health_check_handler))
        .route("/api/v1/alerts", get(get_alerts_handler))
        .route("/api/v1/metrics", get(get_metrics_summary_handler))
        // * * * Endpoints Forenses y de Telemetría Complementarios * * *
        .route("/api/v1/alerts/:id", get(get_alert_by_id_handler))
        .route("/api/v1/alerts/simulate-push", post(simulate_push_handler))
        .route("/api/v1/alerts/subscribe", post(subscribe_topic_handler))
        .route("/api/v1/telemetry/metrics", get(get_telemetry_metrics_handler))
        .route("/api/v1/telemetry/logs", get(get_logs_handler))
        // * * * Catálogos, Topología y Estado del Sistema * * *
        .route("/api/v1/status", get(system_status_handler))
        .route("/api/v1/catalogs/device-types", get(get_device_types_handler))
        .route("/api/v1/inventory/nodes", get(get_nodes_handler))
        .route("/api/v1/ml/stats", get(get_ml_stats_handler))
        .route("/api/v1/ml/model-info", get(get_model_info_handler));

    let pwa_paths = ["../frontend", "frontend", "/app/frontend"];
    for p in pwa_paths {
        if std::path::Path::new(p).exists() {
            println!("|- INSTRUCCION -| [PWA] Montando interfaz PWA en ruta estática: /pwa desde [{}]", p);
            router = router.nest_service("/pwa", ServeDir::new(p));
            break;
        }
    }

    router
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
