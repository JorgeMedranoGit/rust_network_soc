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
