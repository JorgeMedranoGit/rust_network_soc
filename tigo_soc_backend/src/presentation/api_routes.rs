// * * * CONFIGURACIÓN DE ENRUTAMIENTO, ESTADO COMPARTIDO Y MIDDLEWARE CORS (AXUM) * * *

use std::sync::Arc;
use axum::{
    extract::FromRef,
    routing::{delete, get, post, put},
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
        create_user_handler, delete_user_handler, get_alert_by_id_handler, get_alerts_handler,
        get_critical_timeout_handler, get_device_types_handler, get_logs_handler,
        get_metrics_summary_handler, get_ml_stats_handler, get_model_info_handler,
        get_nodes_handler, get_roles_handler, get_session_handler, get_telemetry_metrics_handler,
        get_users_handler, health_check_handler, login_handler, logout_handler, reauth_handler,
        approve_acl_handler, get_approved_acls_handler, simulate_push_handler,
        subscribe_topic_handler, system_status_handler,
        update_critical_timeout_handler, update_user_role_handler,
        validate_critical_action_handler,
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
        // * * * ACLs aprobadas en la red GNS3 SANDBOX (VLAN 999) -> futuras detecciones * * *
        .route("/api/v1/acls/approved", get(get_approved_acls_handler))
        .route("/api/v1/acls/approve", post(approve_acl_handler))
        .route("/api/v1/ml/stats", get(get_ml_stats_handler))
        .route("/api/v1/ml/model-info", get(get_model_info_handler))
        // * * * Autenticación y Sesiones Seguras en Servidor * * *
        .route("/api/v1/auth/login", post(login_handler))
        .route("/api/v1/auth/logout", post(logout_handler))
        .route("/api/v1/auth/session", get(get_session_handler))
        .route("/api/v1/auth/reauth", post(reauth_handler))
        .route("/api/v1/auth/validate-critical", post(validate_critical_action_handler))
        // * * * CRUD de Usuarios y Roles (RBAC) * * *
        .route("/api/v1/users", get(get_users_handler).post(create_user_handler))
        .route("/api/v1/users/:id/role", put(update_user_role_handler))
        .route("/api/v1/users/:id", delete(delete_user_handler))
        .route("/api/v1/roles", get(get_roles_handler))
        // * * * Configuración Global de Timeout para Acciones Críticas * * *
        .route("/api/v1/config/critical-timeout", get(get_critical_timeout_handler).put(update_critical_timeout_handler));


    let pwa_paths = ["../frontend", "frontend", "/app/frontend"];
    for p in pwa_paths {
        if std::path::Path::new(p).exists() {
            println!("|- INSTRUCCION -| [FRONTEND UNIFICADO] Montando consola SPA Leptos WASM en puerto 3000 desde [{}]", p);
            router = router
                .nest_service("/pwa", ServeDir::new(p))
                .fallback_service(ServeDir::new(p));
            break;
        }
    }

    router
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
