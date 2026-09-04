#![allow(dead_code)]
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;

// =========================================================================
// 1. CATÁLOGOS BASE
// =========================================================================

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct DeviceType {
    pub type_id: Option<i32>,
    pub type_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct MgmtProtocol {
    pub protocol_id: Option<i32>,
    pub protocol_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct ThreatType {
    pub threat_id: Option<i32>,
    pub threat_name: String,
    pub severity_level: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct AlertStatus {
    pub status_id: Option<i32>,
    pub status_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct TaskStatus {
    pub status_id: Option<i32>,
    pub status_name: String,
}

// =========================================================================
// 2. RBAC Y SEGURIDAD DEL SISTEMA
// =========================================================================

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct Role {
    pub role_id: Option<i32>,
    pub role_name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct Permission {
    pub permission_id: Option<i32>,
    pub permission_name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct RolePermission {
    pub role_id: i32,
    pub permission_id: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct SysUser {
    pub user_id: Option<i32>,
    pub username: String,
    pub password_hash: String,
    pub role_id: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct SystemParameter {
    pub param_id: Option<i32>,
    pub param_key: String,
    pub param_value: String,
    pub description: Option<String>,
    pub last_updated_by: Option<i32>,
    pub updated_at: Option<DateTime<Utc>>,
}

// =========================================================================
// 3. INVENTARIO Y TELEMETRÍA (DIVIDIDA: AGREGADA VS FORENSE)
// =========================================================================

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct NetworkNode {
    pub node_id: Option<i32>,
    pub hostname: String,
    pub ip_address: String,
    pub type_id: Option<i32>,
    pub protocol_id: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct NetworkTrafficMetric {
    pub metric_id: Option<i64>,
    pub node_id: Option<i32>,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub total_packets: i64,
    pub total_bytes: i64,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct NetworkLog {
    pub log_id: Option<i64>,
    pub node_id: Option<i32>,
    pub source_ip: String,
    pub destination_ip: String,
    pub protocol: String,
    pub packet_size: i32,
    pub flags: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct FeatureStore {
    pub feature_id: Option<i64>,
    pub log_id: i64,
    pub feature_vector: Value,
    pub processed_at: Option<DateTime<Utc>>,
}

// =========================================================================
// 4. ORQUESTACIÓN Y MITIGACIÓN
// =========================================================================

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct MitigationAction {
    pub action_id: Option<i32>,
    pub action_name: String,
    pub layer_target: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct SecurityAlert {
    pub alert_id: Option<i64>,
    pub feature_id: Option<i64>,
    pub threat_id: Option<i32>,
    pub status_id: Option<i32>,
    pub anomaly_score: f64,
    pub detected_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct ExecutionQueue {
    pub task_id: Option<i64>,
    pub alert_id: Option<i64>,
    pub action_id: Option<i32>,
    pub node_id: Option<i32>,
    pub status_id: Option<i32>,
    pub queued_at: Option<DateTime<Utc>>,
    pub executed_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct RollbackSnapshot {
    pub snapshot_id: Option<i64>,
    pub task_id: i64,
    pub pre_state_json: Value,
    pub rollback_payload: Value,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct MitigationAudit {
    pub audit_id: Option<i64>,
    pub task_id: Option<i64>,
    pub executed_by_user: Option<i32>,
    pub executed_at: Option<DateTime<Utc>>,
    pub result_status: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct SystemAuditLog {
    pub sys_audit_id: Option<i64>,
    pub user_id: Option<i32>,
    pub action: String,
    pub target_table: String,
    pub target_id: Option<i64>,
    pub timestamp: Option<DateTime<Utc>>,
}

// =========================================================================
// 5. DOMAIN EVENTS (Para Sniffer y Threat Detection)
// =========================================================================

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkEvent {
    pub source_ip: String,
    pub destination_ip: String,
    pub protocol: String,
    pub packet_size: i32,
    pub flags: Option<String>,
    pub anomaly_score: Option<f32>,
    pub timestamp: DateTime<Utc>,
}
