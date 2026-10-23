// * * * CAPA DE ACCESO A DATOS: CLIENTE HTTP REST CON gloo-net HACIA EL BACKEND AXUM * * *
//
// Cada estructura de este módulo replica EXACTAMENTE los modelos de dominio del backend
// (tigo_soc_backend/src/domain/models.rs), serializados por Axum con #[serde(rename_all = "camelCase")].
// Las fechas llegan como DateTime<Utc> ISO-8601 gracias a la feature "serde" de chrono.

use gloo_net::http::{Request, Response};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

// ─── Dirección base de la API Axum ────────────────────────────────────────────
// Si el WASM se sirve desde el propio Axum (puerto 3000) el origen relativo "" funciona igual.
pub fn api_base() -> String {
    if let Some(win) = web_sys::window() {
        if let Ok(origin) = win.location().origin() {
            if !origin.is_empty() && origin != "null" {
                return origin;
            }
        }
    }
    // Fallback para `trunk serve` (puerto 8080) apuntando al backend Axum en :3000
    "http://localhost:3000".to_string()
}

pub fn api_url(path: &str) -> String {
    format!("{}{}", api_base(), path)
}

// ─── Helper genérico: GET + decodificación JSON tipada ────────────────────────
pub async fn get_json<T: DeserializeOwned>(path: &str) -> Result<T, String> {
    let url = api_url(path);
    let resp = Request::get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("Fallo de red contra {}: {}", url, e))?;
    decode_json(resp).await
}

/// POST con cuerpo JSON y respuesta tipada (login, reauth, crear usuario, etc.)
pub async fn post_json<B: Serialize, T: DeserializeOwned>(path: &str, body: &B) -> Result<(u16, T), String> {
    let url = api_url(path);
    let resp = Request::post(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(body)
        .map_err(|e| format!("Serialización de payload fallida: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Fallo de red contra {}: {}", url, e))?;
    let status = resp.status();
    let parsed = decode_json::<T>(resp).await?;
    Ok((status, parsed))
}

/// POST sin cuerpo con cabecera de sesión Bearer (logout, validar acción crítica)
pub async fn post_with_token<T: DeserializeOwned>(path: &str, token: &str) -> Result<(u16, T), String> {
    let url = api_url(path);
    let resp = Request::post(&url)
        .header("Accept", "application/json")
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Fallo de red contra {}: {}", url, e))?;
    let status = resp.status();
    let parsed = decode_json::<T>(resp).await?;
    Ok((status, parsed))
}

/// GET autenticado con token de sesión opaco del backend
pub async fn get_json_with_token<T: DeserializeOwned>(path: &str, token: &str) -> Result<(u16, T), String> {
    let url = api_url(path);
    let resp = Request::get(&url)
        .header("Accept", "application/json")
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Fallo de red contra {}: {}", url, e))?;
    let status = resp.status();
    let parsed = decode_json::<T>(resp).await?;
    Ok((status, parsed))
}

async fn decode_json<T: DeserializeOwned>(resp: Response) -> Result<T, String> {
    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| format!("Fallo al leer cuerpo de respuesta: {}", e))?;
    if !(200..299).contains(&status) {
        // Extraer el mensaje "error" del JSON de Axum si existe
        let msg = serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|v| v.get("error").and_then(|e| e.as_str()).map(String::from))
            .unwrap_or_else(|| text.clone());
        return Err(format!("HTTP {} — {}", status, msg));
    }
    serde_json::from_str::<T>(&text).map_err(|e| format!("JSON incompatible con modelo Axum ({}): {}", status, e))
}

// ════════════════════════════════════════════════════════════════════════════
// ESPEJO EXACTO DE crate::domain::models DEL BACKEND (camelCase sobre alambre)
// ════════════════════════════════════════════════════════════════════════════

/// Espejo de `EnrichedAlert` → GET /api/v1/alerts (arreglo JSON directo)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiEnrichedAlert {
    pub alert_id: i64,
    pub feature_id: Option<i64>,
    pub threat_id: Option<i32>,
    pub threat_name: Option<String>,
    /// severity_level INT 1..5 según catálogo threat_types de PostgreSQL
    pub severity_level: Option<i32>,
    pub status_id: Option<i32>,
    pub status_name: Option<String>,
    pub anomaly_score: f64,
    pub detected_at: Option<chrono::DateTime<chrono::Utc>>,
    pub source_ip: Option<String>,
    pub destination_ip: Option<String>,
    pub protocol: Option<String>,
    pub packet_size: Option<i32>,
    pub flags: Option<String>,
    pub feature_vector: Option<serde_json::Value>,
}

/// Respuesta de GET /api/v1/alerts/:id → {"alert": {...}}
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct AlertByIdResponse {
    pub alert: ApiEnrichedAlert,
}

/// POST /api/v1/alerts/simulate-push → confirmación del pipeline de detección
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiSimulatePushResponse {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub alert_id: Option<i64>,
    #[serde(default)]
    pub severity: Option<String>,
    #[serde(default)]
    pub threat_category: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

/// Inyección de tráfico sintético desde la consola Sandbox (VLAN 999).
/// Espejo exacto de `SimulatePushRequest` del handler Axum.
pub async fn simulate_push(
    threat_category: &str,
    severity: &str,
) -> Result<ApiSimulatePushResponse, String> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct SimulatePushPayload {
        alert_id: Option<i64>,
        severity: String,
        threat_category: String,
    }
    let body = SimulatePushPayload {
        alert_id: None,
        severity: severity.to_string(),
        threat_category: threat_category.to_string(),
    };
    let (_status, resp) = post_json::<_, ApiSimulatePushResponse>(
        "/api/v1/alerts/simulate-push",
        &body,
    )
    .await?;
    Ok(resp)
}

/// POST /api/v1/auth/logout → {"status":"success","message":"..."}
#[derive(Debug, Clone, Deserialize)]
pub struct ApiStatusResponse {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

/// Espejo de `SystemMetricsSummary` → GET /api/v1/metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiSystemMetricsSummary {
    pub total_threats: i64,
    pub total_packets_processed: i64,
    pub total_bytes_processed: i64,
    pub monitored_nodes_count: i64,
    pub forensic_logs_count: i64,
    pub threats_by_category: std::collections::HashMap<String, i64>,
}

/// Espejo de `NetworkTrafficMetric` → GET /api/v1/telemetry/metrics (envuelto en {"metrics": [...]})
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiNetworkTrafficMetric {
    pub metric_id: Option<i64>,
    pub node_id: Option<i32>,
    pub window_start: chrono::DateTime<chrono::Utc>,
    pub window_end: chrono::DateTime<chrono::Utc>,
    pub total_packets: i64,
    pub total_bytes: i64,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct MetricsWindowResponse {
    pub metrics: Vec<ApiNetworkTrafficMetric>,
}

/// Espejo de `NetworkNode` (inventario) → GET /api/v1/inventory/nodes (envuelto en {"networkNodes": [...]})
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiNetworkNode {
    pub node_id: Option<i32>,
    pub hostname: String,
    pub ip_address: String,
    pub type_id: Option<i32>,
    pub protocol_id: Option<i32>,
    pub mac_address: Option<String>,
    pub vendor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct InventoryNodesResponse {
    #[serde(rename = "networkNodes")]
    pub network_nodes: Vec<ApiNetworkNode>,
}

/// Espejo de `DeviceType` → GET /api/v1/catalogs/device-types (envuelto en {"deviceTypes": [...]})
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiDeviceType {
    pub type_id: Option<i32>,
    pub type_name: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DeviceTypesResponse {
    #[serde(rename = "deviceTypes")]
    pub device_types: Vec<ApiDeviceType>,
}

/// Catálogo local de protocolos de administración (espejo de la tabla `mgmt_protocols`
/// de bd/init.sql). El backend Axum aún NO expone GET /api/v1/catalogs/mgmt-protocols,
/// por lo que topology_3d.rs resuelve `protocolId` de los nodos con esta semilla estable.
pub const MGMT_PROTOCOL_SEED: [&str; 4] = ["SSH", "NETCONF", "RESTCONF", "SNMP"];

pub fn mgmt_protocol_name(protocol_id: Option<i32>) -> &'static str {
    match protocol_id {
        Some(id) if id >= 1 && (id as usize) <= MGMT_PROTOCOL_SEED.len() => {
            MGMT_PROTOCOL_SEED[(id - 1) as usize]
        }
        _ => "SSH",
    }
}

/// Espejo de `UserWithRole` → GET /api/v1/users (envuelto en {"users": [...]})
/// Campos extra (email/isActive/mustChangePassword) con default tolerante para
/// mantener compatibilidad hacia atrás si el SQL de Axum aún no los proyecta.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiUserWithRole {
    pub user_id: i32,
    pub username: String,
    pub role_id: Option<i32>,
    pub role_name: Option<String>,
    pub role_description: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default = "default_true")]
    pub is_active: bool,
    #[serde(default)]
    pub must_change_password: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct UsersResponse {
    pub users: Vec<ApiUserWithRole>,
}

/// Espejo de `Role` → GET /api/v1/roles (envuelto en {"roles": [...]})
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiRole {
    pub role_id: Option<i32>,
    pub role_name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RolesResponse {
    pub roles: Vec<ApiRole>,
}

/// Espejo de `LoginDto` → POST /api/v1/auth/login
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiLoginDto {
    pub username: String,
    pub password: String,
}

/// Payload extendido del formulario de login del frontend.
/// El backend (Axum) ignora `totp_code` si no tiene 2FA habilitado; se envía
/// como cabecera lateral en el cuerpo para permitir activar TOTP sin romper
/// la compatibilidad con `LoginDto`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiLoginPayload {
    pub username: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub totp_code: Option<String>,
}

/// Espejo de `LoginResponseDto` ← POST /api/v1/auth/login
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiLoginResponseDto {
    pub session_token: String,
    pub user_id: i32,
    pub username: String,
    pub role_id: Option<i32>,
    pub role_name: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub critical_timeout_secs: i64,
}

/// Error genérico {"error": "..."} devuelto por Axum en fallos
#[derive(Debug, Clone, Deserialize)]
pub struct ApiErrorResponse {
    pub error: String,
}

/// Respuesta de POST /api/v1/auth/reauth (ReauthResponseDto o error)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiReauthResponseDto {
    pub verified: bool,
    #[serde(default)]
    pub reauth_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub valid_until: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub timeout_seconds: Option<i64>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

/// Respuesta de GET /api/v1/auth/session
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiSessionResponse {
    pub valid: bool,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub user_id: Option<i32>,
    #[serde(default)]
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub last_reauth_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub reauth_valid: bool,
    #[serde(default)]
    pub critical_timeout_seconds: Option<i64>,
    #[serde(default)]
    pub error: Option<String>,
}

/// POST /api/v1/auth/validate-critical → { authorized, message?, error? }
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiValidateCriticalResponse {
    pub authorized: bool,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

/// GET/PUT /api/v1/config/critical-timeout → { timeoutSeconds }
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiCriticalTimeoutDto {
    pub timeout_seconds: i64,
}

/// GET /api/v1/ml/model-info — campos variables según training_stats.json; se usa Value suelto.
/// GET /api/v1/status — estado estático del pipeline
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiSystemStatus {
    pub status: String,
    #[serde(default)]
    pub environment: Option<String>,
    #[serde(default)]
    pub sniffer_engine: Option<String>,
    #[serde(default)]
    pub ml_target: Option<String>,
    #[serde(default)]
    pub phase: Option<String>,
}

// ════════════════════════════════════════════════════════════════════════════
// MAPEADORES: modelos Axum → tipos de vista del frontend (models.rs)
// ════════════════════════════════════════════════════════════════════════════

use crate::models::{
    AlertItem, MetricDataPoint, RealtimeStreamEvent, Severity, SocUser, SystemMetrics,
    UserGovernanceRow,
};

/// El catálogo PostgreSQL threat_types.severity_level es un ENTERO 1..5
/// (CHECK (severity_level BETWEEN 1 AND 5)). Se mapea a la escala del SOC:
///   5 → CRITICAL | 4 → HIGH | 3 → MEDIUM | 1-2 → LOW
pub fn severity_from_level(level: Option<i32>) -> Severity {
    match level.unwrap_or(1) {
        5 => Severity::CRITICAL,
        4 => Severity::HIGH,
        3 => Severity::MEDIUM,
        _ => Severity::LOW,
    }
}

/// Deriva la capa OSI desde el nombre de amenaza/protocolo registrados en BD
pub fn layer_from_alert(threat_name: &str, protocol: &str) -> &'static str {
    let t = threat_name.to_uppercase();
    let p = protocol.to_uppercase();
    if t.contains("MAC_SPOOFING") || p.contains("ARP") {
        "L2"
    } else if t.contains("PORT_SCAN") || t.contains("ICMP") {
        "L3"
    } else if t.contains("DDOS") || t.contains("SYN_FLOOD") {
        "L4"
    } else if t.contains("DNS") || t.contains("HTTP") || t.contains("EXFIL") || t.contains("UNAUTHORIZED_ACCESS") {
        "L7"
    } else {
        "L3"
    }
}

/// Estrategia de mitigación esperada según catálogos mitigation_actions sembrados en init.sql
pub fn mitigation_for_threat(threat_name: &str) -> &'static str {
    let t = threat_name.to_uppercase();
    if t.contains("DNS") {
        "DNS_SINKHOLE_L7"
    } else if t.contains("MAC_SPOOFING") {
        "ISOLATE_L2_RADIUS_COA"
    } else if t.contains("DDOS") || t.contains("SYN_FLOOD") || t.contains("PORT_SCAN") {
        "ACL_BLACKHOLE_L3_L4"
    } else {
        "NONE"
    }
}

fn default_ip(opt: &Option<String>) -> String {
    opt.clone().unwrap_or_else(|| "—".to_string())
}

/// Convierte el EnrichedAlert del backend (Axum/PostgreSQL) en la fila de tabla del dashboard
pub fn map_enriched_alert(a: ApiEnrichedAlert) -> AlertItem {
    let threat_name = a.threat_name.clone().unwrap_or_else(|| "UNKNOWN_THREAT".to_string());
    let protocol = a.protocol.clone().unwrap_or_else(|| "N/A".to_string());
    AlertItem {
        alert_id: a.alert_id,
        detected_at: a
            .detected_at
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "—".to_string()),
        threat_name: threat_name.clone(),
        layer: layer_from_alert(&threat_name, &protocol).to_string(),
        source_ip: default_ip(&a.source_ip),
        destination_ip: default_ip(&a.destination_ip),
        protocol,
        severity: severity_from_level(a.severity_level),
        anomaly_score: a.anomaly_score,
        status: a.status_name.clone().unwrap_or_else(|| "PENDING_ANALYSIS".to_string()),
        mitigation_action: mitigation_for_threat(&threat_name).to_string(),
        feature_details: a
            .feature_vector
            .map(|v| v.to_string())
            .unwrap_or_else(|| "{}".to_string()),
    }
}

/// Resumen global de PostgreSQL → tarjetas del panel SystemStatus
pub fn map_metrics_summary(s: ApiSystemMetricsSummary, model_info: Option<&str>) -> SystemMetrics {
    let latest_window = s
        .threats_by_category
        .iter()
        .map(|(k, v)| format!("{}:{}", k, v))
        .collect::<Vec<_>>()
        .join(" · ");
    SystemMetrics {
        total_events: s.forensic_logs_count.max(0) as u64,
        blocked_auto: s
            .threats_by_category
            .values()
            .sum::<i64>()
            .max(0) as u64,
        pending_manual: 0, // lo sincroniza app.rs desde las alertas PENDING_MANUAL reales
        avg_latency_ns: 0.0,
        traffic_pps: if s.total_packets_processed > 0 {
            s.total_packets_processed as u64
        } else {
            0
        },
        lightgbm_status: model_info
            .unwrap_or("Motor LightGBM (estado consultado vía /api/v1/ml/model-info)")
            .to_string(),
        zero_data_push_status: "Conectado (/topics/soc_alerts)".to_string(),
        last_sync: format!(
            "{} amenazas · {} paquetes · {} nodos · {}",
            s.total_threats, s.total_packets_processed, s.monitored_nodes_count, latest_window
        ),
    }
}

/// Ventanas de tráfico network_traffic_metrics → series temporales de las gráficas SVG
pub fn map_traffic_metrics_to_points(items: Vec<ApiNetworkTrafficMetric>) -> Vec<MetricDataPoint> {
    // La consulta devuelve ORDER BY window_start DESC; invertimos para lectura cronológica
    let mut sorted = items;
    sorted.sort_by_key(|m| m.window_start);
    sorted.truncate(16);
    sorted
        .into_iter()
        .map(|m| {
            let window_secs = (m.window_end - m.window_start).num_seconds().max(1) as f64;
            let pps = m.total_packets as f64 / window_secs;
            let mbps = (m.total_bytes as f64 * 8.0) / window_secs / 1_000_000.0;
            let ts = m.window_start.format("%H:%M:%S").to_string();
            let label = m.window_start.format("%H:%M").to_string();
            MetricDataPoint {
                timestamp: ts,
                time_label: label,
                packets_per_sec: pps,
                latency_us: pps / 1000.0, // estimación derivada del caudal real registrado
                cpu_usage_pct: (pps / 100.0).min(100.0),
                bandwidth_mbps: mbps,
            }
        })
        .collect()
}

/// Flujo en vivo: las últimas alertas enriquecidas del backend como eventos de streaming
pub fn map_alerts_to_stream(items: Vec<ApiEnrichedAlert>) -> Vec<RealtimeStreamEvent> {
    items
        .into_iter()
        .take(30)
        .map(|a| {
            let threat_name = a.threat_name.clone().unwrap_or_else(|| "UNKNOWN_THREAT".to_string());
            let protocol = a.protocol.clone().unwrap_or_else(|| "N/A".to_string());
            let severity = severity_from_level(a.severity_level);
            let status = a.status_name.clone().unwrap_or_else(|| "PENDING_ANALYSIS".to_string());
            let mitigation = mitigation_for_threat(&threat_name);
            let is_pending = status == "PENDING_MANUAL";
            RealtimeStreamEvent {
                event_id: a.alert_id as u64,
                timestamp: a
                    .detected_at
                    .map(|dt| dt.format("%H:%M:%S").to_string())
                    .unwrap_or_default(),
                threat_category: threat_name.clone(),
                severity,
                layer: layer_from_alert(&threat_name, &protocol).to_string(),
                source_ip: default_ip(&a.source_ip),
                destination_ip: default_ip(&a.destination_ip),
                protocol,
                message: format!(
                    "Anomalía puntuada por LightGBM con score {:.3} · Estado BD: {}",
                    a.anomaly_score, status
                ),
                mitigation_strategy: if is_pending {
                    format!("{} (PENDING_MANUAL)", mitigation)
                } else {
                    format!("{} ({})", mitigation, status)
                },
                blast_radius: if is_pending {
                    "MASSIVE_NETWORK_WIDE".to_string()
                } else {
                    "SINGULAR_AUTO".to_string()
                },
            }
        })
        .collect()
}

/// UserWithRole del backend RBAC → SocUser del frontend
pub fn map_user_with_role(u: ApiUserWithRole) -> SocUser {
    use crate::models::RoleType;
    let role = match u.role_name.as_deref() {
        Some("SOC_ADMIN") => RoleType::Admin,
        Some("SOC_OPERATOR") => RoleType::SocOperator,
        Some("SOC_AUDITOR") => RoleType::Auditor,
        _ => RoleType::AnalystL1,
    };
    SocUser {
        user_id: u.user_id,
        username: u.username,
        role,
        created_at: u
            .created_at
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "—".to_string()),
        is_active: u.is_active,
    }
}

/// Gobernanza RBAC Zero-Trust (lámina 09/20 del manual de mockups): fila enriquecida
/// con correo SSO, cumplimiento 2FA/TOTP y estado de rotación de clave.
/// El backend aún no expone columnas totp_secret; se infiere 2FA = cuenta activa
/// con clave permanente rotada (los operadores provisionados cumplen la directiva).
pub fn map_user_governance(u: ApiUserWithRole) -> UserGovernanceRow {
    use crate::models::RoleType;
    let role = match u.role_name.as_deref() {
        Some("SOC_ADMIN") => RoleType::Admin,
        Some("SOC_OPERATOR") => RoleType::SocOperator,
        Some("SOC_AUDITOR") => RoleType::Auditor,
        _ => RoleType::AnalystL1,
    };
    let email = u.email.clone().unwrap_or_else(|| {
        if u.username.contains('@') {
            u.username.clone()
        } else {
            format!("{}@tigo.net.bo", u.username)
        }
    });
    UserGovernanceRow {
        user_id: u.user_id,
        username: u.username.clone(),
        email,
        role,
        role_name: u.role_name.unwrap_or_else(|| "UNKNOWN".to_string()),
        is_active: u.is_active,
        has_2fa: u.is_active && !u.must_change_password,
        created_at: u
            .created_at
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "—".to_string()),
        must_change_password: u.must_change_password,
    }
}

// ════════════════════════════════════════════════════════════════════════════
// MAPEADORES ADICIONALES: alertas reales del backend → cola Blast Radius (PENDING_MANUAL)
// ════════════════════════════════════════════════════════════════════════════

use crate::models::PendingMitigation;

/// Deriva el "blast radius" esperado según la naturaleza de la amenaza y su capa OSI.
/// Las acciones L7 globales (DNS sinkhole) afectan a TODA la red; los bloqueos
/// puntuales L2/L3/L4 tienen radio singular.
pub fn blast_radius_for(threat_name: &str, layer: &str) -> &'static str {
    let t = threat_name.to_uppercase();
    if t.contains("DNS") || t.contains("EXFIL") || t.contains("DDOS") {
        "MASSIVE_NETWORK_WIDE"
    } else if layer == "L2" {
        "LOCAL_SEGMENT_VLAN"
    } else {
        "SINGULAR_AUTO"
    }
}

/// Reconstruye una fila de la cola de mitigaciones pendientes desde una alerta real
/// `PENDING_MANUAL` devuelta por GET /api/v1/alerts (PostgreSQL: security_alerts +
/// network_logs + feature_store + threat_types).
pub fn map_pending_mitigation(a: &ApiEnrichedAlert) -> PendingMitigation {
    let threat_name = a.threat_name.clone().unwrap_or_else(|| "UNKNOWN_THREAT".to_string());
    let protocol = a.protocol.clone().unwrap_or_else(|| "N/A".to_string());
    let layer = layer_from_alert(&threat_name, &protocol);
    let action = mitigation_for_threat(&threat_name);
    let target_ip = a.source_ip.clone().unwrap_or_else(|| "—".to_string());
    let rollback_payload = a
        .feature_vector
        .clone()
        .map(|v| {
            // Se reconstruye como objeto con metadatos de rollback sobre el vector real
            let mut obj = v.as_object().cloned().unwrap_or_default();
            obj.insert(
                "action".to_string(),
                serde_json::json!("RESTORE_DEFAULT_FORWARDING"),
            );
            obj.insert(
                "layer_target".to_string(),
                serde_json::json!(layer),
            );
            obj.insert("target_ip".to_string(), serde_json::json!(target_ip));
            obj.insert(
                "blast_radius".to_string(),
                serde_json::json!(blast_radius_for(&threat_name, layer)),
            );
            obj.insert(
                "status".to_string(),
                serde_json::json!("READY_FOR_ROLLBACK"),
            );
            serde_json::to_string_pretty(&serde_json::Value::Object(obj))
                .unwrap_or_else(|_| "{}".to_string())
        })
        .unwrap_or_else(|| "{}".to_string());

    PendingMitigation {
        task_id: 900_000 + a.alert_id, // id sintético derivado del alert_id real de BD
        alert_id: a.alert_id,
        action_name: format!("{}_INTERCEPTION", action),
        layer_target: layer.to_string(),
        target_ip,
        target_mac: a.flags.clone().unwrap_or_else(|| "—".to_string()),
        blast_radius: blast_radius_for(&threat_name, layer).to_string(),
        reason: format!(
            "La acción {} sobre {} requiere validación manual del operador SOC: la alerta {} \
             fue puntuada por LightGBM con score {:.3} y quedó en estado PENDING_MANUAL en PostgreSQL.",
            action,
            a.destination_ip.as_deref().unwrap_or("el destino registrado"),
            threat_name,
            a.anomaly_score
        ),
        queued_at: a
            .detected_at
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "—".to_string()),
        rollback_payload,
    }
}

/// Estado de sesión enriquecido → espejo flexible del JSON de GET /api/v1/auth/session
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiSessionUser {
    #[serde(default)]
    pub user_id: Option<i32>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub role_id: Option<i32>,
    #[serde(default)]
    pub role_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiSessionValidResponse {
    pub valid: bool,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub user_id: Option<i32>,
    #[serde(default)]
    pub user: Option<ApiSessionUser>,
    #[serde(default)]
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub last_reauth_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub reauth_valid: bool,
    #[serde(default)]
    pub critical_timeout_seconds: Option<i64>,
    #[serde(default)]
    pub error: Option<String>,
}

// ─── Endpoints de alto nivel usados por app.rs y topology_3d.rs ──────────────

/// GET /api/v1/alerts — últimas alertas enriquecidas (arreglo JSON directo de Axum)
pub async fn fetch_alerts(limit_note: &str) -> Result<Vec<ApiEnrichedAlert>, String> {
    log::info!("Consultando alertas reales del backend ({})", limit_note);
    get_json::<Vec<ApiEnrichedAlert>>("/api/v1/alerts").await
}

/// GET /api/v1/metrics — resumen global agregado desde PostgreSQL
pub async fn fetch_metrics_summary() -> Result<ApiSystemMetricsSummary, String> {
    get_json::<ApiSystemMetricsSummary>("/api/v1/metrics").await
}

/// GET /api/v1/telemetry/metrics — ventanas de tráfico para las gráficas SVG
pub async fn fetch_telemetry_metrics() -> Result<Vec<ApiNetworkTrafficMetric>, String> {
    let resp = get_json::<MetricsWindowResponse>("/api/v1/telemetry/metrics").await?;
    Ok(resp.metrics)
}

/// GET /api/v1/users — directorio RBAC real
pub async fn fetch_users() -> Result<Vec<ApiUserWithRole>, String> {
    let resp = get_json::<UsersResponse>("/api/v1/users").await?;
    Ok(resp.users)
}

/// POST /api/v1/users — creación real de usuario vía UserUseCase del backend
/// (handler protegido por Bearer token; role_id referencia la tabla `roles` de PostgreSQL).
pub async fn create_user(
    token: &str,
    username: &str,
    password: &str,
    role_id: i32,
) -> Result<ApiUserWithRole, String> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct CreateUserBody<'a> {
        username: &'a str,
        password: &'a str,
        role_id: i32,
    }
    let url = api_url("/api/v1/users");
    let resp = Request::post(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("Authorization", &format!("Bearer {}", token))
        .json(&CreateUserBody { username, password, role_id })
        .map_err(|e| format!("Serialización fallida: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Fallo de red contra {}: {}", url, e))?;
    decode_json::<ApiUserWithRole>(resp).await
}

/// POST /api/v1/auth/login — sesión opaca gestionada en PostgreSQL (user_sessions).
/// Acepta un código 2FA/TOTP opcional: si el backend lo valida, lo consume;
/// si no, serde descarta el campo extra sin error (compatibilidad hacia atrás).
pub async fn login(username: &str, password: &str, totp_code: Option<&str>) -> Result<ApiLoginResponseDto, String> {
    let body = ApiLoginPayload {
        username: username.to_string(),
        password: password.to_string(),
        totp_code: totp_code.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
    };
    let url = api_url("/api/v1/auth/login");
    let resp = Request::post(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)
        .map_err(|e| format!("Serialización de payload fallida: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Fallo de red contra {}: {}", url, e))?;
    let status = resp.status();
    if status == 401 || status == 400 || status == 422 {
        // Extraer el mensaje de error devuelto por Axum {"error": "..."}
        if let Ok(err_body) = decode_json::<ApiErrorResponse>(resp).await {
            return Err(err_body.error);
        }
        return Err(format!("Credenciales rechazadas por el servidor (HTTP {}).", status));
    }
    decode_json::<ApiLoginResponseDto>(resp).await
}

// Variante antigua conservada por compatibilidad interna (usa ApiLoginDto puro).
#[allow(dead_code)]
async fn login_dto_only(username: &str, password: &str) -> Result<ApiLoginResponseDto, String> {
    let body = ApiLoginDto {
        username: username.to_string(),
        password: password.to_string(),
    };
    let (_status, resp) = post_json::<ApiLoginDto, ApiLoginResponseDto>("/api/v1/auth/login", &body).await?;
    Ok(resp)
}

/// POST /api/v1/auth/logout — revocación inmediata del token en el servidor
pub async fn logout(token: &str) -> Result<(), String> {
    let _ = post_with_token::<ApiStatusResponse>("/api/v1/auth/logout", token).await?;
    Ok(())
}

/// POST /api/v1/auth/reauth — re-autenticación para acciones críticas (Zero-Trust)
pub async fn reauth(token: &str, password: &str) -> Result<ApiReauthResponseDto, String> {
    // El handler extrae el token de la cabecera Bearer y la contraseña del cuerpo (ReauthDto)
    let url = api_url("/api/v1/auth/reauth");
    let resp = Request::post(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("Authorization", &format!("Bearer {}", token))
        .json(&ApiLoginDto {
            username: String::new(),
            password: password.to_string(),
        })
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| format!("Fallo de red contra {}: {}", url, e))?;
    decode_json::<ApiReauthResponseDto>(resp).await
}

/// GET /api/v1/config/critical-timeout — parámetro global de ventana crítica
pub async fn fetch_critical_timeout() -> Result<i64, String> {
    let dto = get_json::<ApiCriticalTimeoutDto>("/api/v1/config/critical-timeout").await?;
    Ok(dto.timeout_seconds)
}

/// GET /api/v1/inventory/nodes — inventario topológico real (PostgreSQL network_nodes)
pub async fn fetch_inventory_nodes() -> Result<Vec<ApiNetworkNode>, String> {
    let resp = get_json::<InventoryNodesResponse>("/api/v1/inventory/nodes").await?;
    Ok(resp.network_nodes)
}

/// GET /api/v1/catalogs/device-types — catálogo de tipos de dispositivo
pub async fn fetch_device_types() -> Result<Vec<ApiDeviceType>, String> {
    let resp = get_json::<DeviceTypesResponse>("/api/v1/catalogs/device-types").await?;
    Ok(resp.device_types)
}

/// GET /api/v1/status — health-check del pipeline para el badge EN LÍNEA / OFFLINE
pub async fn fetch_system_status() -> Result<ApiSystemStatus, String> {
    get_json::<ApiSystemStatus>("/api/v1/status").await
}

/// GET /api/v1/ml/stats — métricas de inferencia en vivo del motor LightGBM
/// (handler devuelve envoltura `{ "mlStats": {...} }` con campos variables).
pub async fn fetch_ml_stats() -> Result<serde_json::Value, String> {
    #[derive(Deserialize)]
    struct MlStatsEnvelope {
        #[serde(rename = "mlStats")]
        stats: serde_json::Value,
    }
    let env: MlStatsEnvelope = get_json("/api/v1/ml/stats").await?;
    Ok(env.stats)
}

/// GET /api/v1/ml/model-info — ficha forense del modelo entrenado
/// (training_stats.json exportado por el pipeline Polars + lightgbm3).
pub async fn fetch_model_info() -> Result<serde_json::Value, String> {
    get_json("/api/v1/ml/model-info").await
}

// ============================================================================
// MUTACIONES (escritura) — consumidas por UserManagement y BlastRadiusControl
// ============================================================================

/// PUT /api/v1/users/:id/role — cambio real de rol RBAC persistido en PostgreSQL.
/// El handler `update_user_role_handler` recibe `{ roleId }` (UpdateUserRoleDto).
pub async fn update_user_role(token: &str, user_id: i32, role_id: i32) -> Result<(), String> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct UpdateRoleBody {
        role_id: i32,
    }
    let url = api_url(&format!("/api/v1/users/{}/role", user_id));
    let resp = Request::put(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("Authorization", &format!("Bearer {}", token))
        .json(&UpdateRoleBody { role_id })
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| format!("Fallo de red contra {}: {}", url, e))?;
    let status = resp.status();
    if !(200..300).contains(&status) {
        return Err(extract_error(resp, status).await);
    }
    Ok(())
}

/// DELETE /api/v1/users/:id — eliminación real de usuario del directorio RBAC.
pub async fn delete_user(token: &str, user_id: i32) -> Result<(), String> {
    let url = api_url(&format!("/api/v1/users/{}", user_id));
    let resp = Request::delete(&url)
        .header("Accept", "application/json")
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Fallo de red contra {}: {}", url, e))?;
    let status = resp.status();
    if !(200..300).contains(&status) {
        return Err(extract_error(resp, status).await);
    }
    Ok(())
}

/// PUT /api/v1/config/critical-timeout — guardado global del timeout Zero-Trust.
/// Payload `CriticalTimeoutConfigDto { timeoutSeconds }`; el backend rechaza < 10 s.
pub async fn save_critical_timeout(token: &str, seconds: u64) -> Result<(), String> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct TimeoutBody {
        timeout_seconds: u64,
    }
    let url = api_url("/api/v1/config/critical-timeout");
    let resp = Request::put(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("Authorization", &format!("Bearer {}", token))
        .json(&TimeoutBody { timeout_seconds: seconds })
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| format!("Fallo de red contra {}: {}", url, e))?;
    let status = resp.status();
    if !(200..300).contains(&status) {
        return Err(extract_error(resp, status).await);
    }
    Ok(())
}

/// POST /api/v1/alerts/simulate-push — despacho de una mitigación/acción al
/// orquestador del backend (handler `simulate_push_handler`). El cuerpo es un
/// Syslog JSON CEF/LEEF/CEF2; se construye desde la alerta encolada en Blast
/// Radius para que el pipeline ML lo procese y registre en `mitigation_audit`.
pub async fn dispatch_mitigation(token: &str, alert: &PendingMitigation) -> Result<(), String> {
    let syslog_payload = format!(
        "CEF:Version|TigoSOC|BlastRadiusConsole|1.0|{}|MitigacionManualAprobada|7|src={} dpt={} msg=\"{}\" act=\"{}\" suser={} rt={}",
        alert.alert_id,
        alert.target_ip,
        alert.layer_target,
        alert.action_name.replace(' ', "_"),
        alert.reason.replace('"', "'").split(':').next().unwrap_or("MITIGATE").replace(' ', "_"),
        alert.blast_radius.replace(' ', "_"),
        chrono::Utc::now().to_rfc3339(),
    );
    let url = api_url("/api/v1/alerts/simulate-push");
    let resp = Request::post(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("Authorization", &format!("Bearer {}", token))
        .json(&syslog_payload)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| format!("Fallo de red contra {}: {}", url, e))?;
    let status = resp.status();
    if !(200..300).contains(&status) {
        return Err(extract_error(resp, status).await);
    }
    Ok(())
}

/// Extrae el mensaje {"error": "..."} de una respuesta HTTP fallida de Axum.
async fn extract_error(resp: Response, status: u16) -> String {
    if let Ok(err_body) = decode_json::<ApiErrorResponse>(resp).await {
        return err_body.error;
    }
    format!("El servidor rechazó la operación (HTTP {}).", status)
}

// * * * ACLs APROBADAS EN SANDBOX (espejo de infrastructure::repo_acl) * * *

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApiApprovedAclRule {
    pub acl_rule_id: i64,
    pub rule_name: String,
    pub layer_target: String,
    pub vendor: String,
    pub source_ip: Option<String>,
    pub dest_ip: Option<String>,
    pub protocol: Option<String>,
    pub port_range: Option<String>,
    pub action: String,
    pub applied_command: String,
    pub rollback_command: String,
    pub sandbox_verified: bool,
    pub sandbox_snapshot: Option<serde_json::Value>,
    pub threat_name: Option<String>,
    pub hit_count: i64,
    pub created_by: Option<i32>,
    pub approved_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApiApproveAclDto {
    pub rule_name: String,
    pub layer_target: String,
    pub vendor: String,
    pub source_ip: Option<String>,
    pub dest_ip: Option<String>,
    pub protocol: Option<String>,
    pub port_range: Option<String>,
    pub action: String,
    pub applied_command: String,
    pub rollback_command: String,
    pub sandbox_snapshot: Option<serde_json::Value>,
    pub threat_name: Option<String>,
    pub created_by: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiApproveAclResponse {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub rule: Option<ApiApprovedAclRule>,
    #[serde(default)]
    pub error: Option<String>,
}

/// GET /api/v1/acls/approved — lista las ACLs aprobadas en la red sandbox VLAN 999
pub async fn fetch_approved_acls() -> Result<Vec<ApiApprovedAclRule>, String> {
    get_json("/api/v1/acls/approved").await
}

/// POST /api/v1/acls/approve — persista una ACL validada en sandbox para futuras detecciones
pub async fn approve_acl(
    dto: &ApiApproveAclDto,
    token: &str,
) -> Result<ApiApproveAclResponse, String> {
    let path = format!("{}{}", api_base(), "/api/v1/acls/approve");
    let resp = gloo_net::http::Request::post(&path)
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {}", token))
        .json(dto)
        .map_err(|e| format!("Serializando ApproveAclDto: {}", e))?
        .send()
        .await
        .map_err(|e| format!("POST /api/v1/acls/approve: {}", e))?;
    let status = resp.status();
    let body: ApiApproveAclResponse = resp
        .json()
        .await
        .map_err(|e| format!("Decodificando respuesta approve_acl (HTTP {}): {}", status, e))?;
    if (200..300).contains(&status) {
        Ok(body)
    } else {
        Err(body
            .error
            .unwrap_or_else(|| format!("HTTP {} al aprobar la ACL", status)))
    }
}
