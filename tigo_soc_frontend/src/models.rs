// * * * MODELOS DE DATOS Y ESTRUCTURAS REACTIVAS DEL SOC FRONTEND * * *

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    CRITICAL,
    HIGH,
    MEDIUM,
    LOW,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::CRITICAL => "CRITICAL",
            Severity::HIGH => "HIGH",
            Severity::MEDIUM => "MEDIUM",
            Severity::LOW => "LOW",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlertItem {
    pub alert_id: i64,
    pub detected_at: String,
    pub threat_name: String,
    pub layer: String,
    pub source_ip: String,
    pub destination_ip: String,
    pub protocol: String,
    pub severity: Severity,
    pub anomaly_score: f64,
    pub status: String,
    pub mitigation_action: String,
    pub feature_details: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub total_events: u64,
    pub blocked_auto: u64,
    pub pending_manual: u64,
    pub avg_latency_ns: f64,
    pub traffic_pps: u64,
    pub lightgbm_status: String,
    pub zero_data_push_status: String,
    pub last_sync: String,
}

impl Default for SystemMetrics {
    /// Estado vacío inicial — se llena con datos REALES vía GET /api/v1/metrics (Axum + PostgreSQL)
    fn default() -> Self {
        SystemMetrics {
            total_events: 0,
            blocked_auto: 0,
            pending_manual: 0,
            avg_latency_ns: 0.0,
            traffic_pps: 0,
            lightgbm_status: "Consultando backend Axum…".to_string(),
            zero_data_push_status: "Conectado (/topics/soc_alerts)".to_string(),
            last_sync: "—".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PendingMitigation {
    pub task_id: i64,
    pub alert_id: i64,
    pub action_name: String,
    pub layer_target: String,
    pub target_ip: String,
    pub target_mac: String,
    pub blast_radius: String,
    pub reason: String,
    pub queued_at: String,
    pub rollback_payload: String,
}




#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricDataPoint {
    pub timestamp: String,
    pub time_label: String,
    pub packets_per_sec: f64,
    pub latency_us: f64,
    pub cpu_usage_pct: f64,
    pub bandwidth_mbps: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealtimeStreamEvent {
    pub event_id: u64,
    pub timestamp: String,
    pub threat_category: String,
    pub severity: Severity,
    pub layer: String,
    pub source_ip: String,
    pub destination_ip: String,
    pub protocol: String,
    pub message: String,
    pub mitigation_strategy: String,
    pub blast_radius: String,
}



// * * * 4. CONTROL DE ACCESO BASADO EN ROLES (RBAC), USUARIOS Y SESIONES * * *

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoleType {
    Admin,       // Administrador general del SOC
    SocOperator, // Operador SOC - Ejecución de mitigaciones y rollback
    Auditor,     // Auditor - Solo lectura forense y telemetría
    AnalystL1,   // Analista L1 - Triaje inicial
}

#[allow(dead_code)]
impl RoleType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RoleType::Admin => "ADMINISTRADOR",
            RoleType::SocOperator => "OPERADOR_SOC",
            RoleType::Auditor => "AUDITOR",
            RoleType::AnalystL1 => "ANALISTA_L1",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            RoleType::Admin => "Administrador General",
            RoleType::SocOperator => "Operador SOC",
            RoleType::Auditor => "Auditor de Seguridad",
            RoleType::AnalystL1 => "Analista L1 (Triaje)",
        }
    }

    pub fn can_manage_users(&self) -> bool {
        matches!(self, RoleType::Admin)
    }

    pub fn can_execute_mitigations(&self) -> bool {
        matches!(self, RoleType::Admin | RoleType::SocOperator)
    }

    pub fn is_read_only(&self) -> bool {
        matches!(self, RoleType::Auditor)
    }

    /// Determina si el rol puede ejecutar mitigaciones o rollback destructivos.
    /// AnalystL1 solo hace triaje — NO puede ejecutar comandos de red.
    pub fn can_execute_destructive_actions(&self) -> bool {
        matches!(self, RoleType::Admin | RoleType::SocOperator)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SocUser {
    pub user_id: i32,
    pub username: String,
    pub role: RoleType,
    pub created_at: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserSessionState {
    pub current_user: SocUser,
    pub session_token: String,
    pub expires_at: String,
    pub last_reauth_timestamp: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalSecurityConfig {
    pub critical_action_timeout_secs: u64,
}

// ════════════════════════════════════════════════════════════════════════════
// VISTA DE GOBERNANZA RBAC — espejo de ApiUserWithRole (GET /api/v1/users)
// Añade los campos exigidos por la lámina 09/20 del manual de mockups:
// correo SSO corporativo, estado 2FA/TOTP y cumplimiento ISO 27001.
// ════════════════════════════════════════════════════════════════════════════
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserGovernanceRow {
    pub user_id: i32,
    pub username: String,
    pub email: String,
    pub role: RoleType,
    pub role_name: String,
    pub is_active: bool,
    /// true si el operador tiene segundo factor TOTP registrado (RFC 6238)
    pub has_2fa: bool,
    pub created_at: String,
    /// Cumplimiento Zero-Trust: clave temporal sin rotar = incumplimiento
    pub must_change_password: bool,
}

impl Default for UserGovernanceRow {
    fn default() -> Self {
        UserGovernanceRow {
            user_id: 0,
            username: String::new(),
            email: String::new(),
            role: RoleType::Auditor,
            role_name: String::new(),
            is_active: false,
            has_2fa: false,
            created_at: String::new(),
            must_change_password: false,
        }
    }
}




