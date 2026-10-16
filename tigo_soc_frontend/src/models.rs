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

/// Generador de datos simulados iniciales para demostración interactiva en caliente
pub fn get_initial_alerts() -> Vec<AlertItem> {
    vec![
        AlertItem {
            alert_id: 101,
            detected_at: "2026-10-12 04:31:15".to_string(),
            threat_name: "DNS_C2_EXFILTRATION".to_string(),
            layer: "L7".to_string(),
            source_ip: "192.168.1.50".to_string(),
            destination_ip: "192.168.1.1".to_string(),
            protocol: "UDP:53".to_string(),
            severity: Severity::CRITICAL,
            anomaly_score: 0.985,
            status: "PENDING_MANUAL".to_string(),
            mitigation_action: "DNS_SINKHOLE_L7_INTERCEPTION".to_string(),
            feature_details: "{\"domain\":\"evil-c2.com\",\"queries\":20,\"rate_pps\":182.4}".to_string(),
        },
        AlertItem {
            alert_id: 102,
            detected_at: "2026-10-12 04:18:22".to_string(),
            threat_name: "PORT_SCAN_BRUTEFORCE".to_string(),
            layer: "L3".to_string(),
            source_ip: "192.168.100.50".to_string(),
            destination_ip: "192.168.1.1".to_string(),
            protocol: "TCP:SYN".to_string(),
            severity: Severity::HIGH,
            anomaly_score: 0.952,
            status: "MITIGATED".to_string(),
            mitigation_action: "BLOCK_L3_NGFW_RESTCONF".to_string(),
            feature_details: "{\"scanned_ports\":500,\"syn_flags\":1,\"vendor\":\"NGFW\"}".to_string(),
        },
        AlertItem {
            alert_id: 103,
            detected_at: "2026-10-12 04:05:40".to_string(),
            threat_name: "L2_MAC_SPOOFING_ATTACK".to_string(),
            layer: "L2".to_string(),
            source_ip: "192.168.1.11".to_string(),
            destination_ip: "192.168.1.50".to_string(),
            protocol: "ARP/ETH".to_string(),
            severity: Severity::HIGH,
            anomaly_score: 0.941,
            status: "MITIGATED".to_string(),
            mitigation_action: "ISOLATE_L2_RADIUS_COA".to_string(),
            feature_details: "{\"spoofed_mac\":\"02:42:ac:11:ff:ee\",\"vlan\":99}".to_string(),
        },
        AlertItem {
            alert_id: 104,
            detected_at: "2026-10-12 03:52:10".to_string(),
            threat_name: "DDOS_SYN_FLOOD".to_string(),
            layer: "L4".to_string(),
            source_ip: "192.168.100.99".to_string(),
            destination_ip: "192.168.1.10".to_string(),
            protocol: "TCP:80".to_string(),
            severity: Severity::CRITICAL,
            anomaly_score: 0.991,
            status: "MITIGATED".to_string(),
            mitigation_action: "ACL_BLACKHOLE_DYNAMIC_INJECTION".to_string(),
            feature_details: "{\"null_route\":\"192.168.100.99/32\",\"drop_chain\":\"INPUT\"}".to_string(),
        },
        AlertItem {
            alert_id: 105,
            detected_at: "2026-10-12 03:30:05".to_string(),
            threat_name: "ICMP_FLOOD".to_string(),
            layer: "L3".to_string(),
            source_ip: "10.0.0.45".to_string(),
            destination_ip: "10.0.0.1".to_string(),
            protocol: "ICMP".to_string(),
            severity: Severity::MEDIUM,
            anomaly_score: 0.780,
            status: "ANALYZED".to_string(),
            mitigation_action: "RATE_LIMIT_ICMP".to_string(),
            feature_details: "{\"packet_rate\":450,\"avg_size\":64}".to_string(),
        },
        AlertItem {
            alert_id: 106,
            detected_at: "2026-10-12 03:15:44".to_string(),
            threat_name: "UNAUTHORIZED_ACCESS_PROBE".to_string(),
            layer: "L7".to_string(),
            source_ip: "172.16.0.88".to_string(),
            destination_ip: "172.16.0.2".to_string(),
            protocol: "HTTP:8080".to_string(),
            severity: Severity::LOW,
            anomaly_score: 0.420,
            status: "LOGGED".to_string(),
            mitigation_action: "NONE".to_string(),
            feature_details: "{\"uri\":\"/admin/login\",\"status_code\":401}".to_string(),
        },
    ]
}

pub fn get_initial_metrics() -> SystemMetrics {
    SystemMetrics {
        total_events: 184520,
        blocked_auto: 142,
        pending_manual: 1,
        avg_latency_ns: 2840.0,
        traffic_pps: 3250,
        lightgbm_status: "Activo (5 Folds / 23 Features)".to_string(),
        zero_data_push_status: "Conectado (/topics/soc_alerts)".to_string(),
        last_sync: "Hace 2 segundos".to_string(),
    }
}

pub fn get_initial_pending_mitigations() -> Vec<PendingMitigation> {
    vec![
        PendingMitigation {
            task_id: 501,
            alert_id: 101,
            action_name: "DNS_SINKHOLE_L7_INTERCEPTION".to_string(),
            layer_target: "L7".to_string(),
            target_ip: "192.168.1.50".to_string(),
            target_mac: "00:50:56:c0:00:08".to_string(),
            blast_radius: "MASSIVE_NETWORK_WIDE".to_string(),
            reason: "La alteración de la resolución DNS para evil-c2.com afecta globalmente a todos los clientes de la red corporativa.".to_string(),
            queued_at: "2026-10-12 04:31:16".to_string(),
            rollback_payload: r#"{
  "action": "RESTORE_DNS_ZONE_RESOLUTION",
  "layer_target": "L7",
  "vendor": "DnsSinkhole",
  "target_ip": "192.168.1.50",
  "domain": "evil-c2.com",
  "sinkhole_ip": "192.168.1.254",
  "instruction": {
    "protocol": "DNS_UDP_53",
    "method": "UNSINKHOLE_ZONE",
    "bind9_command": "rndc flushname evil-c2.com",
    "unbound_command": "unbound-control local_data_remove evil-c2.com"
  },
  "blast_radius": "MASSIVE_NETWORK_WIDE",
  "status": "READY_FOR_ROLLBACK"
}"#.to_string(),
        }
    ]
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

pub fn get_initial_chart_data() -> Vec<MetricDataPoint> {
    vec![
        MetricDataPoint { timestamp: "05:45:00".to_string(), time_label: "05:45".to_string(), packets_per_sec: 1420.0, latency_us: 2.4, cpu_usage_pct: 18.5, bandwidth_mbps: 11.2 },
        MetricDataPoint { timestamp: "05:46:00".to_string(), time_label: "05:46".to_string(), packets_per_sec: 1650.0, latency_us: 2.6, cpu_usage_pct: 21.0, bandwidth_mbps: 13.5 },
        MetricDataPoint { timestamp: "05:47:00".to_string(), time_label: "05:47".to_string(), packets_per_sec: 1530.0, latency_us: 2.3, cpu_usage_pct: 19.8, bandwidth_mbps: 12.0 },
        MetricDataPoint { timestamp: "05:48:00".to_string(), time_label: "05:48".to_string(), packets_per_sec: 2100.0, latency_us: 3.1, cpu_usage_pct: 28.4, bandwidth_mbps: 17.8 },
        MetricDataPoint { timestamp: "05:49:00".to_string(), time_label: "05:49".to_string(), packets_per_sec: 3850.0, latency_us: 4.8, cpu_usage_pct: 49.2, bandwidth_mbps: 34.6 },
        MetricDataPoint { timestamp: "05:50:00".to_string(), time_label: "05:50".to_string(), packets_per_sec: 5200.0, latency_us: 6.9, cpu_usage_pct: 68.5, bandwidth_mbps: 48.0 },
        MetricDataPoint { timestamp: "05:51:00".to_string(), time_label: "05:51".to_string(), packets_per_sec: 4900.0, latency_us: 5.8, cpu_usage_pct: 61.3, bandwidth_mbps: 42.1 },
        MetricDataPoint { timestamp: "05:52:00".to_string(), time_label: "05:52".to_string(), packets_per_sec: 3200.0, latency_us: 3.9, cpu_usage_pct: 39.0, bandwidth_mbps: 26.5 },
        MetricDataPoint { timestamp: "05:53:00".to_string(), time_label: "05:53".to_string(), packets_per_sec: 2400.0, latency_us: 3.0, cpu_usage_pct: 31.2, bandwidth_mbps: 19.4 },
        MetricDataPoint { timestamp: "05:54:00".to_string(), time_label: "05:54".to_string(), packets_per_sec: 1980.0, latency_us: 2.7, cpu_usage_pct: 25.0, bandwidth_mbps: 15.8 },
        MetricDataPoint { timestamp: "05:55:00".to_string(), time_label: "05:55".to_string(), packets_per_sec: 2150.0, latency_us: 2.8, cpu_usage_pct: 26.4, bandwidth_mbps: 16.9 },
        MetricDataPoint { timestamp: "05:56:00".to_string(), time_label: "05:56".to_string(), packets_per_sec: 2300.0, latency_us: 2.9, cpu_usage_pct: 28.0, bandwidth_mbps: 18.2 },
    ]
}

pub fn get_initial_realtime_events() -> Vec<RealtimeStreamEvent> {
    vec![
        RealtimeStreamEvent {
            event_id: 1001,
            timestamp: "06:01:42".to_string(),
            threat_category: "DNS_C2_EXFILTRATION".to_string(),
            severity: Severity::CRITICAL,
            layer: "L7".to_string(),
            source_ip: "192.168.1.50".to_string(),
            destination_ip: "192.168.1.1".to_string(),
            protocol: "UDP:53".to_string(),
            message: "Peticiones recursivas anómalas a evil-c2.com detectadas por LightGBM (Score: 0.985)".to_string(),
            mitigation_strategy: "DNS_SINKHOLE_L7 (PENDING_MANUAL)".to_string(),
            blast_radius: "MASSIVE_NETWORK_WIDE".to_string(),
        },
        RealtimeStreamEvent {
            event_id: 1002,
            timestamp: "06:01:28".to_string(),
            threat_category: "PORT_SCAN_BRUTEFORCE".to_string(),
            severity: Severity::HIGH,
            layer: "L3".to_string(),
            source_ip: "192.168.100.50".to_string(),
            destination_ip: "192.168.1.1".to_string(),
            protocol: "TCP:SYN".to_string(),
            message: "Escaneo perimetral agresivo de puertos bloqueado vía RESTCONF/YANG en NGFW".to_string(),
            mitigation_strategy: "BLOCK_L3_NGFW_RESTCONF (AUTO)".to_string(),
            blast_radius: "SINGULAR_AUTO".to_string(),
        },
        RealtimeStreamEvent {
            event_id: 1003,
            timestamp: "06:00:55".to_string(),
            threat_category: "DDOS_SYN_FLOOD".to_string(),
            severity: Severity::CRITICAL,
            layer: "L4".to_string(),
            source_ip: "192.168.100.99".to_string(),
            destination_ip: "192.168.1.10".to_string(),
            protocol: "TCP:80".to_string(),
            message: "Inundación SYN masiva mitigada mediante Null Route L3 y filtro dinámico iptables".to_string(),
            mitigation_strategy: "ACL_BLACKHOLE_L3_L4 (AUTO)".to_string(),
            blast_radius: "SINGULAR_AUTO".to_string(),
        },
        RealtimeStreamEvent {
            event_id: 1004,
            timestamp: "06:00:12".to_string(),
            threat_category: "L2_MAC_SPOOFING_ATTACK".to_string(),
            severity: Severity::HIGH,
            layer: "L2".to_string(),
            source_ip: "192.168.1.11".to_string(),
            destination_ip: "192.168.1.50".to_string(),
            protocol: "ARP".to_string(),
            message: "Dispositivo no autenticado aislado en VLAN de cuarentena mediante RADIUS CoA UDP 3799".to_string(),
            mitigation_strategy: "ISOLATE_L2_RADIUS_COA (AUTO)".to_string(),
            blast_radius: "SINGULAR_AUTO".to_string(),
        },
    ]
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

pub fn get_initial_users() -> Vec<SocUser> {
    vec![
        SocUser {
            user_id: 1,
            username: "admin_soc".to_string(),
            role: RoleType::Admin,
            created_at: "2026-10-12 00:00:00".to_string(),
            is_active: true,
        },
        SocUser {
            user_id: 2,
            username: "operador_red".to_string(),
            role: RoleType::SocOperator,
            created_at: "2026-10-12 01:15:30".to_string(),
            is_active: true,
        },
        SocUser {
            user_id: 3,
            username: "auditor_seguridad".to_string(),
            role: RoleType::Auditor,
            created_at: "2026-10-12 02:30:10".to_string(),
            is_active: true,
        },
        SocUser {
            user_id: 4,
            username: "analista_triaje".to_string(),
            role: RoleType::AnalystL1,
            created_at: "2026-10-12 03:45:00".to_string(),
            is_active: true,
        },
    ]
}

pub fn get_initial_security_config() -> GlobalSecurityConfig {
    GlobalSecurityConfig {
        critical_action_timeout_secs: 300,
    }
}

pub fn get_initial_session() -> UserSessionState {
    let users = get_initial_users();
    UserSessionState {
        current_user: users[0].clone(),
        session_token: "d9e4a7b2c1f08e54a938c71b02d64f891a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d".to_string(),
        expires_at: "24h (Server-Backed Opaque Session)".to_string(),
        last_reauth_timestamp: 0.0,
    }
}

