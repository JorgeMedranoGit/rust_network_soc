// * * * MODELOS DE DOMINIO Y ESTRUCTURAS DE DATOS (ARQUITECTURA LIMPIA) * * *
#![allow(dead_code)]
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;

// * * * 1. CATÁLOGOS BASE * * *

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DeviceType {
    pub type_id: Option<i32>,
    pub type_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MgmtProtocol {
    pub protocol_id: Option<i32>,
    pub protocol_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ThreatType {
    pub threat_id: Option<i32>,
    pub threat_name: String,
    pub severity_level: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AlertStatus {
    pub status_id: Option<i32>,
    pub status_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TaskStatus {
    pub status_id: Option<i32>,
    pub status_name: String,
}

// * * * 2. RBAC Y SEGURIDAD DEL SISTEMA * * *

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Role {
    pub role_id: Option<i32>,
    pub role_name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Permission {
    pub permission_id: Option<i32>,
    pub permission_name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RolePermission {
    pub role_id: i32,
    pub permission_id: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SysUser {
    pub user_id: Option<i32>,
    pub username: String,
    pub password_hash: String,
    pub role_id: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SystemParameter {
    pub param_id: Option<i32>,
    pub param_key: String,
    pub param_value: String,
    pub description: Option<String>,
    pub last_updated_by: Option<i32>,
    pub updated_at: Option<DateTime<Utc>>,
}

// * * * 3. INVENTARIO Y TELEMETRÍA (DIVIDIDA: AGREGADA VS FORENSE) * * *

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct NetworkNode {
    pub node_id: Option<i32>,
    pub hostname: String,
    pub ip_address: String,
    pub type_id: Option<i32>,
    pub protocol_id: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct FeatureStore {
    pub feature_id: Option<i64>,
    pub log_id: i64,
    pub feature_vector: Value,
    pub processed_at: Option<DateTime<Utc>>,
}

// * * * 4. ORQUESTACIÓN Y MITIGACIÓN * * *

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MitigationAction {
    pub action_id: Option<i32>,
    pub action_name: String,
    pub layer_target: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SecurityAlert {
    pub alert_id: Option<i64>,
    pub feature_id: Option<i64>,
    pub threat_id: Option<i32>,
    pub status_id: Option<i32>,
    pub anomaly_score: f64,
    pub detected_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct RollbackSnapshot {
    pub snapshot_id: Option<i64>,
    pub task_id: i64,
    pub pre_state_json: Value,
    pub rollback_payload: Value,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MitigationAudit {
    pub audit_id: Option<i64>,
    pub task_id: Option<i64>,
    pub executed_by_user: Option<i32>,
    pub executed_at: Option<DateTime<Utc>>,
    pub result_status: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SystemAuditLog {
    pub sys_audit_id: Option<i64>,
    pub user_id: Option<i32>,
    pub action: String,
    pub target_table: String,
    pub target_id: Option<i64>,
    pub timestamp: Option<DateTime<Utc>>,
}

// * * * 5. EVENTOS DE DOMINIO DE RED (SNIFFER - CERO ASIGNACIONES EN HEAP) * * *

use std::net::IpAddr;

pub const TCP_FLAG_FIN: u8 = 0b0000_0001;
pub const TCP_FLAG_SYN: u8 = 0b0000_0010;
pub const TCP_FLAG_RST: u8 = 0b0000_0100;
pub const TCP_FLAG_PSH: u8 = 0b0000_1000;
pub const TCP_FLAG_ACK: u8 = 0b0001_0000;
pub const TCP_FLAG_URG: u8 = 0b0010_0000;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[repr(u8)]
pub enum L4Protocol {
    TCP = 6,
    UDP = 17,
    ICMP = 1,
    Other = 0,
}

impl L4Protocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            L4Protocol::TCP => "TCP",
            L4Protocol::UDP => "UDP",
            L4Protocol::ICMP => "ICMP",
            L4Protocol::Other => "OTHER",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct NetworkEvent {
    pub source_ip: IpAddr,
    pub destination_ip: IpAddr,
    pub source_port: u16,
    pub destination_port: u16,
    pub protocol: L4Protocol,
    pub packet_size: u16,
    pub header_length: u8,
    pub ttl: u8,
    pub flags: u8,
    pub anomaly_score: Option<f32>,
    pub timestamp: DateTime<Utc>,
}

impl NetworkEvent {
    // * * * CONVERSIÓN DE MÁSCARAS DE BITS TCP A REPRESENTACIÓN EN CADENA * * *
    pub fn flags_to_string(&self) -> Option<String> {
        if self.flags == 0 {
            return None;
        }
        let mut list = Vec::with_capacity(4);
        if (self.flags & TCP_FLAG_SYN) != 0 { list.push("SYN"); }
        if (self.flags & TCP_FLAG_ACK) != 0 { list.push("ACK"); }
        if (self.flags & TCP_FLAG_FIN) != 0 { list.push("FIN"); }
        if (self.flags & TCP_FLAG_RST) != 0 { list.push("RST"); }
        if (self.flags & TCP_FLAG_PSH) != 0 { list.push("PSH"); }
        if (self.flags & TCP_FLAG_URG) != 0 { list.push("URG"); }
        if list.is_empty() {
            None
        } else {
            Some(list.join("|"))
        }
    }
}

// * * * 6. ESTRUCTURAS DE MACHINE LEARNING (LIGHTGBM Y POLARS) * * *

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExtractedFeatures {
    pub rate: f32,
    pub iat: f32,
    pub variance: f32,
    pub header_length: f32,
    pub ttl: f32,
    pub ack_count: f32,
    pub syn_count: f32,
    pub fin_count: f32,
    pub rst_count: f32,
    pub http: f32,
    pub https: f32,
    pub dns: f32,
    pub ssh: f32,
    pub tcp: f32,
    pub udp: f32,
    pub icmp: f32,
    pub tot_sum: f32,
    pub min_size: f32,
    pub max_size: f32,
    pub avg_size: f32,
    pub std_size: f32,
    pub tot_size: f32,
    pub number: f32,
}

impl ExtractedFeatures {
    // * * * CONVERSIÓN DE CARACTERÍSTICAS A ARREGLO ESTÁTICO EN EL STACK (ZERO-ALLOCATION) * * *
    #[inline(always)]
    pub fn to_array(&self) -> [f32; 23] {
        [
            self.rate,
            self.iat,
            self.variance,
            self.header_length,
            self.ttl,
            self.ack_count,
            self.syn_count,
            self.fin_count,
            self.rst_count,
            self.http,
            self.https,
            self.dns,
            self.ssh,
            self.tcp,
            self.udp,
            self.icmp,
            self.tot_sum,
            self.min_size,
            self.max_size,
            self.avg_size,
            self.std_size,
            self.tot_size,
            self.number,
        ]
    }

    // * * * CONVERTIR CARACTERÍSTICAS AL VECTOR EXACTO DE ENTRADA LIGHTGBM * * *
    pub fn to_vector(&self) -> Vec<f32> {
        self.to_array().to_vec()
    }

    // * * * SERIALIZAR A FORMATO JSONB ESTRUCTURADO PARA FEATURE_STORE * * *
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "rate": self.rate,
            "iat": self.iat,
            "variance": self.variance,
            "headerLength": self.header_length,
            "timeToLive": self.ttl,
            "ackCount": self.ack_count,
            "synCount": self.syn_count,
            "finCount": self.fin_count,
            "rstCount": self.rst_count,
            "http": self.http,
            "https": self.https,
            "dns": self.dns,
            "ssh": self.ssh,
            "tcp": self.tcp,
            "udp": self.udp,
            "icmp": self.icmp,
            "totSum": self.tot_sum,
            "min": self.min_size,
            "max": self.max_size,
            "avg": self.avg_size,
            "std": self.std_size,
            "totSize": self.tot_size,
            "number": self.number
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ThreatEvaluation {
    pub is_attack: bool,
    pub probability: f32,
    pub threat_name: String,
    pub threat_id: i32,
    pub severity: String,
    pub impact: String,
    pub resolution: String,
    pub description: String,
    pub technical_details: String,
    pub features: Vec<f32>,
    pub feature_time_us: f64,
    pub inference_time_us: f64,
    pub total_time_us: f64,
}

// * * * 7. ESTRUCTURA ENRIQUECIDA PARA CONSULTA FORENSE DE ALERTAS * * *

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct EnrichedAlert {
    pub alert_id: i64,
    pub feature_id: Option<i64>,
    pub threat_id: Option<i32>,
    pub threat_name: Option<String>,
    pub severity_level: Option<i32>,
    pub status_id: Option<i32>,
    pub status_name: Option<String>,
    pub anomaly_score: f64,
    pub detected_at: Option<DateTime<Utc>>,
    pub source_ip: Option<String>,
    pub destination_ip: Option<String>,
    pub protocol: Option<String>,
    pub packet_size: Option<i32>,
    pub flags: Option<String>,
    pub feature_vector: Option<Value>,
}
