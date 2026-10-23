// * * * CONTROLADORES Y HANDLERS DE LA API REST (ARQUITECTURA LIMPIA) * * *

use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{Duration, Utc};
use serde::Deserialize;
use serde_json::json;

use crate::domain::models::{
    CreateUserDto, CriticalTimeoutConfigDto, LoginDto, LoginResponseDto, ReauthDto,
    ReauthResponseDto, UpdateUserRoleDto,
};
use crate::infrastructure::{
    repo_catalogs::CatalogsRepository,
    repo_inventory::InventoryRepository,
    repo_orchestration::OrchestrationRepository,
    repo_telemetry::TelemetryRepository,
    repo_users::UsersRepository,
    security,
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

// * * * 9. AUTENTICACIÓN, SESIONES SEGURAS EN SERVIDOR Y RE-AUTENTICACIÓN RBAC * * *

fn extract_session_token(headers: &HeaderMap) -> Option<String> {
    if let Some(auth_header) = headers.get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                return Some(auth_str[7..].trim().to_string());
            }
        }
    }
    if let Some(custom) = headers.get("X-Session-Token") {
        if let Ok(s) = custom.to_str() {
            return Some(s.trim().to_string());
        }
    }
    None
}

/// Inicio de sesión seguro: valida credenciales y genera token opaco en PostgreSQL (user_sessions)
pub async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginDto>,
) -> impl IntoResponse {
    let repo = UsersRepository::new(state.pool.clone());
    let user_opt = match repo.get_user_by_username(&payload.username).await {
        Ok(u) => u,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Error en base de datos: {}", e) })),
            );
        }
    };

    let user = match user_opt {
        Some(u) => u,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Credenciales inválidas" })),
            );
        }
    };

    if !security::verify_password(&payload.password, &user.password_hash) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Credenciales inválidas" })),
        );
    }

    let user_id = user.user_id.unwrap_or(1);
    let session_token = security::generate_opaque_session_token();
    let ttl_seconds = 86400; // 24 horas

    let user_details = repo.get_user_by_id(user_id).await.ok().flatten();
    let role_name = user_details
        .and_then(|u| u.role_name)
        .unwrap_or_else(|| "SOC_OPERATOR".to_string());

    let session = match repo
        .create_session(&session_token, user_id, ttl_seconds, None, None)
        .await
    {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Fallo al registrar sesión: {}", e) })),
            );
        }
    };

    let critical_timeout = repo.get_critical_timeout_secs().await.unwrap_or(300);

    let response = LoginResponseDto {
        session_token: session.session_id,
        user_id,
        username: user.username,
        role_id: user.role_id,
        role_name,
        expires_at: session.expires_at,
        critical_timeout_secs: critical_timeout,
    };

    (StatusCode::OK, Json(json!(response)))
}

/// Cierre de sesión y revocación inmediata del token de sesión en el servidor
pub async fn logout_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let token = match extract_session_token(&headers) {
        Some(t) => t,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Cabecera de sesión ausente" })),
            );
        }
    };

    let repo = UsersRepository::new(state.pool);
    match repo.revoke_session(&token).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "status": "success", "message": "Sesión revocada exitosamente" })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

/// Valida la vigencia de la sesión actual y actualiza el timestamp de última actividad
pub async fn get_session_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let token = match extract_session_token(&headers) {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Token de sesión no proporcionado" })),
            );
        }
    };

    let repo = UsersRepository::new(state.pool);
    match repo.get_valid_session(&token).await {
        Ok(Some(session)) => {
            let _ = repo.touch_session(&token).await;
            let user_info = repo.get_user_by_id(session.user_id).await.ok().flatten();
            let critical_timeout = repo.get_critical_timeout_secs().await.unwrap_or(300);
            let reauth_valid = repo.is_reauth_valid(&token).await.unwrap_or(false);

            (
                StatusCode::OK,
                Json(json!({
                    "valid": true,
                    "sessionId": session.session_id,
                    "userId": session.user_id,
                    "user": user_info,
                    "expiresAt": session.expires_at,
                    "lastActivity": session.last_activity,
                    "lastReauthAt": session.last_reauth_at,
                    "reauthValid": reauth_valid,
                    "criticalTimeoutSeconds": critical_timeout
                })),
            )
        }
        Ok(None) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "valid": false, "error": "Sesión inválida o expirada" })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

/// Re-autenticación estricta por contraseña requerida para mitigación o rollback
pub async fn reauth_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ReauthDto>,
) -> impl IntoResponse {
    let token = match extract_session_token(&headers) {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Token de sesión requerido para re-autenticación" })),
            );
        }
    };

    let repo = UsersRepository::new(state.pool);
    let session = match repo.get_valid_session(&token).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Sesión expirada. Inicie sesión nuevamente." })),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            );
        }
    };

    let user = match repo.get_user_by_id(session.user_id).await {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Usuario no encontrado" })),
            );
        }
    };

    let db_user = match repo.get_user_by_username(&user.username).await {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Error al recuperar credenciales de usuario" })),
            );
        }
    };

    if !security::verify_password(&payload.password, &db_user.password_hash) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "verified": false,
                "error": "Contraseña de re-autenticación incorrecta. Acción crítica bloqueada."
            })),
        );
    }

    if let Err(e) = repo.update_reauth_timestamp(&token).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Fallo al actualizar timestamp de re-autenticación: {}", e) })),
        );
    }

    let timeout_secs = repo.get_critical_timeout_secs().await.unwrap_or(300);
    let now = Utc::now();
    let valid_until = now + Duration::seconds(timeout_secs);

    let resp = ReauthResponseDto {
        verified: true,
        reauth_timestamp: now,
        valid_until,
        timeout_seconds: timeout_secs,
        message: "Re-autenticación exitosa. Acciones críticas autorizadas.".to_string(),
    };

    (StatusCode::OK, Json(json!(resp)))
}

// * * * 10. CRUD DE USUARIOS Y ASIGNACIÓN DE ROLES (RBAC) * * *

pub async fn get_users_handler(State(state): State<AppState>) -> impl IntoResponse {
    let repo = UsersRepository::new(state.pool);
    match repo.get_users_with_roles().await {
        Ok(users) => (StatusCode::OK, Json(json!({ "users": users }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn create_user_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserDto>,
) -> impl IntoResponse {
    if payload.username.trim().is_empty() || payload.password.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "El nombre de usuario y la contraseña no pueden estar vacíos" })),
        );
    }

    let repo = UsersRepository::new(state.pool);
    if let Ok(Some(_)) = repo.get_user_by_username(&payload.username).await {
        return (
            StatusCode::CONFLICT,
            Json(json!({ "error": format!("El usuario '{}' ya existe en el sistema", payload.username) })),
        );
    }

    let password_hash = security::hash_password(&payload.password);
    match repo
        .create_user(&payload.username, &password_hash, payload.role_id)
        .await
    {
        Ok(created) => (
            StatusCode::CREATED,
            Json(json!({
                "status": "success",
                "message": "Usuario registrado exitosamente",
                "user": created
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn update_user_role_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateUserRoleDto>,
) -> impl IntoResponse {
    let repo = UsersRepository::new(state.pool);
    match repo.update_user_role(id, payload.role_id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "message": format!("Rol de usuario #{} actualizado a rol #{}", id, payload.role_id)
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn delete_user_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let repo = UsersRepository::new(state.pool);
    match repo.delete_user(id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "message": format!("Usuario #{} eliminado", id)
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn get_roles_handler(State(state): State<AppState>) -> impl IntoResponse {
    let repo = CatalogsRepository::new(state.pool);
    match repo.get_roles().await {
        Ok(roles) => (StatusCode::OK, Json(json!({ "roles": roles }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

// * * * 11. CONFIGURACIÓN DEL TIMEOUT GLOBAL DE ACCIONES CRÍTICAS * * *

pub async fn get_critical_timeout_handler(State(state): State<AppState>) -> impl IntoResponse {
    let repo = UsersRepository::new(state.pool);
    match repo.get_critical_timeout_secs().await {
        Ok(seconds) => (
            StatusCode::OK,
            Json(json!({ "timeoutSeconds": seconds })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn update_critical_timeout_handler(
    State(state): State<AppState>,
    Json(payload): Json<CriticalTimeoutConfigDto>,
) -> impl IntoResponse {
    if payload.timeout_seconds < 10 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "El timeout mínimo permitido es de 10 segundos" })),
        );
    }

    let repo = UsersRepository::new(state.pool);
    match repo.set_critical_timeout_secs(payload.timeout_seconds, None).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "message": format!("Timeout global de acciones críticas configurado en {} segundos", payload.timeout_seconds),
                "timeoutSeconds": payload.timeout_seconds
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

/// Endpoint de validación previa antes de despachar comandos de mitigación o rollback
pub async fn validate_critical_action_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let token = match extract_session_token(&headers) {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "authorized": false,
                    "error": "UNAUTHENTICATED",
                    "message": "Token de sesión en servidor requerido"
                })),
            );
        }
    };

    let repo = UsersRepository::new(state.pool);
    match repo.is_reauth_valid(&token).await {
        Ok(true) => (
            StatusCode::OK,
            Json(json!({
                "authorized": true,
                "message": "Re-autenticación válida. Comando de mitigación/rollback autorizado para ejecución."
            })),
        ),
        Ok(false) => (
            StatusCode::FORBIDDEN,
            Json(json!({
                "authorized": false,
                "error": "REAUTH_REQUIRED",
                "message": "Timeout de inactividad de acciones críticas expirado. Re-autenticación por contraseña requerida."
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "authorized": false,
                "error": e.to_string()
            })),
        ),
    }
}


// * * * 11. ACLs APROBADAS EN SANDBOX (RED GNS3 SECUNDARIA VLAN 999) * * *
// Flujo de dos redes: la sandbox prueba la ACL; al aprobarse se persiste en
// approved_acl_rules y alimenta futuras detecciones automaticas.

/// GET /api/v1/acls/approved — lista las ACLs aprobadas para la lamina Sandbox
pub async fn get_approved_acls_handler(State(state): State<AppState>) -> impl IntoResponse {
    let repo = crate::infrastructure::repo_acl::AclRepository::new(state.pool);
    match repo.list_approved().await {
        Ok(items) => (StatusCode::OK, Json(json!(items))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

/// POST /api/v1/acls/approve — aprueba una ACL validada en la sandbox y la guarda
pub async fn approve_acl_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(dto): Json<crate::infrastructure::repo_acl::ApproveAclDto>,
) -> impl IntoResponse {
    if dto.rule_name.trim().is_empty() || dto.applied_command.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({ "error": "rule_name y applied_command son obligatorios" })),
        );
    }
    let users_repo = UsersRepository::new(state.pool.clone());
    let created_by = match extract_session_token(&headers) {
        Some(token) => users_repo
            .get_valid_session(&token)
            .await
            .ok()
            .flatten()
            .map(|s| s.user_id),
        None => None,
    };
    // Si el frontend no adjunto el snapshot, se genera uno con los comandos exactos
    let snapshot = dto.sandbox_snapshot.clone().unwrap_or_else(|| json!({
        "source": "lamina-sandbox-acl",
        "vlan": 999,
        "subnet": "10.200.0.0/24",
        "verifiedAt": Utc::now().to_rfc3339(),
        "appliedCommand": dto.applied_command,
        "rollbackCommand": dto.rollback_command,
    }));
    let full = crate::infrastructure::repo_acl::ApproveAclDto {
        created_by: dto.created_by.or(created_by),
        sandbox_snapshot: Some(snapshot),
        ..dto
    };
    let repo = crate::infrastructure::repo_acl::AclRepository::new(state.pool);
    match repo.approve_rule(&full).await {
        Ok(rule) => (
            StatusCode::CREATED,
            Json(json!({
                "status": "success",
                "message": "ACL aprobada en sandbox y persistida para futuras detecciones",
                "rule": rule
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}
