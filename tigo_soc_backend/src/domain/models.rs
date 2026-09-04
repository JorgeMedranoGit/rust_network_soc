use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceType {
    pub type_id: Option<i32>,
    pub type_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MgmtProtocol {
    pub protocol_id: Option<i32>,
    pub protocol_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ThreatType {
    pub threat_id: Option<i32>,
    pub threat_name: String,
    pub severity_level: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AlertStatus {
    pub status_id: Option<i32>,
    pub status_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskStatus {
    pub status_id: Option<i32>,
    pub status_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Role {
    pub role_id: Option<i32>,
    pub role_name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SysUser {
    pub user_id: Option<i32>,
    pub username: String,
    pub password_hash: String,
    pub role_id: i32,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkNode {
    pub node_id: Option<i32>,
    pub hostname: String,
    pub ip_address: String,
    pub type_id: i32,
    pub protocol_id: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkLog {
    pub log_id: Option<i64>,
    pub node_id: i32,
    pub source_ip: String,
    pub destination_ip: String,
    pub protocol: String,
    pub packet_size: i32,
    pub flags: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FeatureStore {
    pub feature_id: Option<i64>,
    pub log_id: i64,
    pub feature_vector: Value,
    pub processed_at: Option<DateTime<Utc>>,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SecurityAlert {
    pub alert_id: Option<i64>,
    pub feature_id: i64,
    pub threat_id: i32,
    pub status_id: i32,
    pub anomaly_score: f64,
    pub detected_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExecutionQueue {
    pub task_id: Option<i64>,
    pub alert_id: i64,
    pub action_id: i32,
    pub node_id: i32,
    pub status_id: i32,
    pub queued_at: Option<DateTime<Utc>>,
    pub executed_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RollbackSnapshot {
    pub snapshot_id: Option<i64>,
    pub task_id: i64,
    pub pre_state_json: Value,
    pub rollback_payload: Value,
    pub created_at: Option<DateTime<Utc>>,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SystemAuditLog {
    pub sys_audit_id: Option<i64>,
    pub user_id: Option<i32>,
    pub action: String,
    pub target_table: String,
    pub target_id: Option<i64>,
    pub timestamp: Option<DateTime<Utc>>,
}