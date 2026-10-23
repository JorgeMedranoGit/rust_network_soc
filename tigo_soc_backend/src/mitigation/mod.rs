// * * * MÓDULO DE MITIGACIÓN MULTI-VENDOR Y ARQUITECTURA AGNÓSTICA DE HARDWARE * * *
#![allow(async_fn_in_trait, dead_code)]

pub mod acl_blackhole;
pub mod cisco;
pub mod dns_sinkhole;
pub mod gns3;
pub mod mikrotik;
pub mod ngfw_restconf;
pub mod openwrt;
pub mod radius_coa;

use std::str::FromStr;
use serde_json::Value;
use sqlx::{PgPool, Row};

pub use acl_blackhole::AclBlackhole;
pub use cisco::CiscoIse;
pub use dns_sinkhole::DnsSinkhole;
pub use gns3::Gns3Alpine;
pub use mikrotik::MikroTikRouter;
pub use ngfw_restconf::NgfwClient;
pub use openwrt::OpenWrtRouter;
pub use radius_coa::RadiusClient;

// * * * 1. TRAIT CENTRAL PARA DISPOSITIVOS DE MITIGACIÓN AGNÓSTICOS * * *
pub trait MitigationDevice: Send + Sync {
    async fn quarantine_ip(&self, ip: &str, mac: &str) -> Result<(), String>;

    async fn quarantine_with_rollback(&self, ip: &str, mac: &str) -> Result<Option<Value>, String> {
        self.quarantine_ip(ip, mac).await.map(|_| None)
    }
}

// * * * 2. ENUM DE MARCAS Y FABRICANTES SOPORTADOS * * *
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vendor {
    SwitchRadius,
    NGFW,
    DnsSinkhole,
    AclBlackhole,
    Cisco,
    GNS3,
    OpenWrt,
    MikroTik,
    Unknown,
}

impl Vendor {
    pub fn as_str(&self) -> &'static str {
        match self {
            Vendor::SwitchRadius => "Switch RADIUS CoA (Capa 2)",
            Vendor::NGFW => "NGFW RESTCONF (Perimetral)",
            Vendor::DnsSinkhole => "DNS Sinkholing (Capa 7)",
            Vendor::AclBlackhole => "ACL Dinámica & Blackholing (Capas 3 y 4)",
            Vendor::Cisco => "Cisco ISE",
            Vendor::GNS3 => "GNS3 Alpine",
            Vendor::OpenWrt => "OpenWrt",
            Vendor::MikroTik => "MikroTik RouterOS",
            Vendor::Unknown => "Desconocido",
        }
    }
}

impl FromStr for Vendor {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_lowercase();
        if lower.contains("radius") || lower.contains("coa") {
            Ok(Vendor::SwitchRadius)
        } else if lower.contains("sinkhole") || lower.contains("dns") {
            Ok(Vendor::DnsSinkhole)
        } else if lower.contains("blackhole") || lower.contains("null") || lower.contains("acl") {
            Ok(Vendor::AclBlackhole)
        } else if lower.contains("ngfw") || lower.contains("restconf") || lower.contains("fortigate") || lower.contains("paloalto") {
            Ok(Vendor::NGFW)
        } else if lower.contains("cisco") {
            Ok(Vendor::Cisco)
        } else if lower.contains("mikrotik") || lower.contains("routerboard") {
            Ok(Vendor::MikroTik)
        } else if lower.contains("openwrt") {
            Ok(Vendor::OpenWrt)
        } else if lower.contains("switch") {
            Ok(Vendor::SwitchRadius)
        } else if lower.contains("gns3") || lower.contains("alpine") || lower.contains("docker") || lower.contains("linux") {
            Ok(Vendor::GNS3)
        } else {
            Ok(Vendor::Unknown)
        }
    }
}

// * * * EVALUACIÓN DE RADIO DE IMPACTO (BLAST RADIUS) * * *
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlastRadius {
    /// Afecta a un solo host (ej. L2 RADIUS aislando MAC/puerto o NGFW bloqueando IP externa singular).
    /// Ejecución completamente automática.
    SingularAuto,
    /// Afecta a múltiples clientes o a toda la red corporativa (ej. L7 DNS Sinkholing afectando resolución de toda la red).
    /// La acción NO se ejecuta automáticamente y pasa a revisión manual con estado PENDING_MANUAL.
    MassiveManual,
}

impl BlastRadius {
    pub fn as_str(&self) -> &'static str {
        match self {
            BlastRadius::SingularAuto => "SINGULAR_AUTO",
            BlastRadius::MassiveManual => "MASSIVE_MANUAL",
        }
    }
}

/// Evalúa el Blast Radius de la acción según el fabricante y la capa de red impactada
pub fn evaluate_blast_radius(vendor: Vendor) -> BlastRadius {
    match vendor {
        Vendor::DnsSinkhole => BlastRadius::MassiveManual,
        _ => BlastRadius::SingularAuto,
    }
}

// * * * DETECCIÓN DE TOPOLOGÍA LOCAL PARA PRIORIZAR AISLAMIENTO DE CAPA 2 (ENLACE) * * *
pub fn is_local_topology(ip: &str) -> bool {
    ip.starts_with("192.168.1.") || ip.starts_with("10.0.") || ip.starts_with("172.16.")
}

// * * * RESOLUCIÓN DE DIRECCIÓN MAC DE CAPA 2 DESDE EL INVENTARIO DE BD * * *
pub async fn get_mac_by_ip(pool: &PgPool, ip: &str) -> Option<String> {
    let row = sqlx::query("SELECT mac_address FROM network_nodes WHERE ip_address = $1")
        .bind(ip)
        .fetch_optional(pool)
        .await
        .ok()??;
    let mac: Option<String> = row.try_get("mac_address").unwrap_or(None);
    mac.filter(|m| !m.trim().is_empty())
}

// * * * 3. DETECTOR DE MARCAS MEDIANTE CONSULTA DINÁMICA A BASE DE DATOS (POSTGRESQL) * * *
pub async fn detect_vendor_by_ip(pool: &PgPool, ip: &str) -> Vendor {
    let query_result = sqlx::query(
        "SELECT vendor FROM network_nodes WHERE ip_address = $1"
    )
    .bind(ip)
    .fetch_optional(pool)
    .await;

    match query_result {
        Ok(Some(row)) => {
            let v_str: Option<String> = row.try_get("vendor").unwrap_or(None);
            match v_str {
                Some(ref s) if !s.trim().is_empty() => Vendor::from_str(s).unwrap_or(Vendor::Unknown),
                _ => {
                    if is_local_topology(ip) {
                        Vendor::SwitchRadius
                    } else {
                        Vendor::Unknown
                    }
                }
            }
        }
        _ => {
            if is_local_topology(ip) {
                Vendor::SwitchRadius
            } else {
                Vendor::Unknown
            }
        }
    }
}

// * * * 4. REGISTRO PREVIO DEL INCIDENTE EN SECURITY_ALERTS * * *
pub async fn register_security_alert(
    pool: &PgPool,
    ip: &str,
    mac: &str,
    vendor: Vendor,
) -> Option<i64> {
    // 1. Obtener ID de tipo de amenaza según contexto o usar UNAUTHORIZED_ACCESS por defecto
    let threat_name = match vendor {
        Vendor::AclBlackhole => "DDOS_SYN_FLOOD",
        Vendor::DnsSinkhole => "DATA_EXFILTRATION",
        Vendor::NGFW => "PORT_SCAN",
        Vendor::SwitchRadius => "UNAUTHORIZED_ACCESS",
        _ => "UNAUTHORIZED_ACCESS",
    };

    let threat_id_opt: Option<i32> = sqlx::query_scalar(
        "SELECT threat_id FROM threat_types WHERE threat_name = $1 LIMIT 1"
    )
    .bind(threat_name)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);
    let threat_id = match threat_id_opt {
        Some(tid) => tid,
        None => {
            if vendor == Vendor::DnsSinkhole {
                1
            } else if vendor == Vendor::AclBlackhole {
                3
            } else {
                4
            }
        }
    };

    // 2. Estado inicial: CONFIRMED_THREAT (2) o PENDING_ANALYSIS (1)
    let status_id_opt: Option<i32> = sqlx::query_scalar(
        "SELECT status_id FROM alert_statuses WHERE status_name = 'CONFIRMED_THREAT' LIMIT 1"
    )
    .fetch_optional(pool)
    .await
    .unwrap_or(None);
    let status_id = status_id_opt.unwrap_or(2);

    let anomaly_score: f64 = match vendor {
        Vendor::DnsSinkhole => 0.98,
        Vendor::AclBlackhole => 0.96,
        _ => 0.95,
    };

    let insert_res = sqlx::query(
        "INSERT INTO security_alerts (feature_id, threat_id, status_id, anomaly_score)
         VALUES (NULL, $1, $2, $3)
         RETURNING alert_id"
    )
    .bind(threat_id)
    .bind(status_id)
    .bind(anomaly_score)
    .fetch_optional(pool)
    .await;

    match insert_res {
        Ok(Some(row)) => {
            let alert_id: i64 = row.try_get("alert_id").unwrap_or(0);
            println!(
                "|- ORQUESTADOR MULTI-VENDOR -| [ALERTA BD] Incidente registrado en security_alerts (ID: [{}]) para IP [{}] / MAC [{}] [{}]",
                alert_id, ip, mac, vendor.as_str()
            );
            Some(alert_id)
        }
        Ok(None) => None,
        Err(e) => {
            eprintln!(
                "|- ORQUESTADOR MULTI-VENDOR -| [WARN] No se pudo persistir en security_alerts ({}). Continuando flujo de contingencia.",
                e
            );
            None
        }
    }
}


// * * * 4.b COINCIDENCIA DE ACLs APROBADAS EN SANDBOX PARA FUTURAS DETECCIONES * * *
// Cuando una amenaza se detecta (o se registra un incidente), el orquestador
// consulta approved_acl_rules: si existe una ACL ya validada en la red GNS3
// sandbox (VLAN 999) para esa threat_name o para la IP del atacante, se puede
// re-aplicar directamente sin volver a pasar por pruebas. Incrementa hit_count.
pub async fn find_approved_acls_for(
    pool: &PgPool,
    threat_name: &str,
    attacker_ip: &str,
) -> Vec<crate::infrastructure::repo_acl::ApprovedAclRule> {
    use crate::infrastructure::repo_acl::AclRepository;
    let repo = AclRepository::new(pool.clone());
    match repo.match_for_detection(threat_name, attacker_ip).await {
        Ok(rules) => {
            if !rules.is_empty() {
                println!(
                    "|- ORQUESTADOR MULTI-VENDOR -| [ACL SANDBOX] {} regla(s) aprobada(s) en VLAN 999 coinciden con '{}' / IP [{}]: {:?}",
                    rules.len(),
                    threat_name,
                    attacker_ip,
                    rules.iter().map(|r| r.rule_name.clone()).collect::<Vec<_>>()
                );
            }
            rules
        }
        Err(e) => {
            eprintln!(
                "|- ORQUESTADOR MULTI-VENDOR -| [WARN] Consulta de ACLs aprobadas falló ({}). Continuando con mitigación estándar.",
                e
            );
            Vec::new()
        }
    }
}

// * * * 5. PERSISTENCIA DE INSTANTÁNEAS DE RESTAURACIÓN (ROLLBACK SNAPSHOTS) * * *
pub async fn save_rollback_snapshot(
    pool: &PgPool,
    alert_id: i64,
    ip: &str,
    mac: &str,
    layer_target: &str,
    action_name: &str,
    rollback_payload: Value,
) {
    // 1. Obtener ID del nodo de red
    let node_id_opt: Option<i32> = sqlx::query_scalar(
        "SELECT node_id FROM network_nodes WHERE ip_address = $1 LIMIT 1"
    )
    .bind(ip)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    // 2. Asegurar o consultar acción de mitigación correspondiente
    let action_desc = format!("Mitigación orquestada en {} mediante {}", layer_target, action_name);
    let action_id_opt: Option<i32> = sqlx::query_scalar(
        "INSERT INTO mitigation_actions (action_name, layer_target, description)
         VALUES ($1, $2, $3)
         ON CONFLICT (action_name) DO UPDATE SET
             layer_target = EXCLUDED.layer_target,
             description = EXCLUDED.description
         RETURNING action_id"
    )
    .bind(action_name)
    .bind(layer_target)
    .bind(&action_desc)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    // 3. Estado de tarea COMPLETED (por defecto 3)
    let status_id_opt: Option<i32> = sqlx::query_scalar(
        "SELECT status_id FROM task_statuses WHERE status_name = 'COMPLETED' LIMIT 1"
    )
    .fetch_optional(pool)
    .await
    .unwrap_or(None);
    let status_id = status_id_opt.unwrap_or(3);

    // 4. Encolar y registrar tarea ejecutada en execution_queue
    let queue_res = sqlx::query(
        "INSERT INTO execution_queue (alert_id, action_id, node_id, status_id, executed_at)
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP)
         RETURNING task_id"
    )
    .bind(alert_id)
    .bind(action_id_opt)
    .bind(node_id_opt)
    .bind(status_id)
    .fetch_optional(pool)
    .await;

    let task_id = match queue_res {
        Ok(Some(row)) => row.try_get::<i64, _>("task_id").unwrap_or(0),
        _ => 0,
    };

    if task_id > 0 {
        // 5. Estado previo estructurado (pre_state_json)
        let pre_state = serde_json::json!({
            "ip_address": ip,
            "mac_address": mac,
            "layer_target": layer_target,
            "authorization_status": "UNRESTRICTED",
            "captured_at": chrono::Utc::now().to_rfc3339()
        });

        // 6. Guardar en rollback_snapshots
        let snapshot_res = sqlx::query(
            "INSERT INTO rollback_snapshots (task_id, pre_state_json, rollback_payload)
             VALUES ($1, $2, $3)
             ON CONFLICT (task_id) DO UPDATE SET
                 pre_state_json = EXCLUDED.pre_state_json,
                 rollback_payload = EXCLUDED.rollback_payload
             RETURNING snapshot_id"
        )
        .bind(task_id)
        .bind(pre_state)
        .bind(&rollback_payload)
        .fetch_optional(pool)
        .await;

        match snapshot_res {
            Ok(Some(row)) => {
                let snap_id: i64 = row.try_get("snapshot_id").unwrap_or(0);
                println!(
                    "|- ORQUESTADOR MULTI-VENDOR -| [ROLLBACK SNAPSHOT] Snapshot de reversión {} guardado con éxito en BD (Snapshot ID: [{}], Task ID: [{}])",
                    layer_target, snap_id, task_id
                );
            }
            Ok(None) => {}
            Err(e) => {
                eprintln!(
                    "|- ORQUESTADOR MULTI-VENDOR -| [WARN] No se pudo guardar rollback_snapshots en BD: {}",
                    e
                );
            }
        }

        // 7. Actualizar security_alerts a MITIGATED
        let _ = sqlx::query(
            "UPDATE security_alerts
             SET status_id = (SELECT status_id FROM alert_statuses WHERE status_name = 'MITIGATED')
             WHERE alert_id = $1"
        )
        .bind(alert_id)
        .execute(pool)
        .await;
    }
}

/// Función auxiliar de compatibilidad para rollback L2
pub async fn save_l2_rollback_snapshot(
    pool: &PgPool,
    alert_id: i64,
    ip: &str,
    mac: &str,
    rollback_payload: Value,
) {
    save_rollback_snapshot(pool, alert_id, ip, mac, "L2", "ISOLATE_L2_RADIUS_COA", rollback_payload).await;
}

// * * * 6. ENCOLADO DE MITIGACIÓN MANUAL POR BLAST RADIUS (IMPACTO MASIVO EN TODA LA RED) * * *
pub async fn queue_pending_manual_mitigation(
    pool: &PgPool,
    alert_id: i64,
    ip: &str,
    mac: &str,
    layer_target: &str,
    action_name: &str,
    rollback_payload: Value,
) -> Option<i64> {
    // 1. Obtener ID del nodo de red
    let node_id_opt: Option<i32> = sqlx::query_scalar(
        "SELECT node_id FROM network_nodes WHERE ip_address = $1 LIMIT 1"
    )
    .bind(ip)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    // 2. Asegurar o consultar acción de mitigación correspondiente
    let action_desc = format!(
        "Mitigación orquestada en {} mediante {} (Blast Radius: Impacto Masivo / Detención Manual)",
        layer_target, action_name
    );
    let action_id_opt: Option<i32> = sqlx::query_scalar(
        "INSERT INTO mitigation_actions (action_name, layer_target, description)
         VALUES ($1, $2, $3)
         ON CONFLICT (action_name) DO UPDATE SET
             layer_target = EXCLUDED.layer_target,
             description = EXCLUDED.description
         RETURNING action_id"
    )
    .bind(action_name)
    .bind(layer_target)
    .bind(&action_desc)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    // 3. Asegurar estado PENDING_MANUAL en task_statuses
    let status_id_opt: Option<i32> = sqlx::query_scalar(
        "INSERT INTO task_statuses (status_name) VALUES ('PENDING_MANUAL')
         ON CONFLICT (status_name) DO UPDATE SET status_name = EXCLUDED.status_name
         RETURNING status_id"
    )
    .fetch_optional(pool)
    .await
    .unwrap_or(None);
    let status_id = status_id_opt.unwrap_or(1);

    // 4. Encolar tarea en execution_queue con estado PENDING_MANUAL (executed_at = NULL por requerir autorización manual)
    let queue_res = sqlx::query(
        "INSERT INTO execution_queue (alert_id, action_id, node_id, status_id, queued_at, executed_at)
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, NULL)
         RETURNING task_id"
    )
    .bind(alert_id)
    .bind(action_id_opt)
    .bind(node_id_opt)
    .bind(status_id)
    .fetch_optional(pool)
    .await;

    let task_id = match queue_res {
        Ok(Some(row)) => row.try_get::<i64, _>("task_id").unwrap_or(0),
        _ => 0,
    };

    if task_id > 0 {
        // 5. Estado previo estructurado (pre_state_json)
        let pre_state = serde_json::json!({
            "ip_address": ip,
            "mac_address": mac,
            "layer_target": layer_target,
            "blast_radius": "MASSIVE_NETWORK_WIDE",
            "execution_mode": "PENDING_MANUAL",
            "authorization_status": "AWAITING_OPERATOR_APPROVAL",
            "captured_at": chrono::Utc::now().to_rfc3339()
        });

        // 6. Guardar en rollback_snapshots
        let snapshot_res = sqlx::query(
            "INSERT INTO rollback_snapshots (task_id, pre_state_json, rollback_payload)
             VALUES ($1, $2, $3)
             ON CONFLICT (task_id) DO UPDATE SET
                 pre_state_json = EXCLUDED.pre_state_json,
                 rollback_payload = EXCLUDED.rollback_payload
             RETURNING snapshot_id"
        )
        .bind(task_id)
        .bind(pre_state)
        .bind(&rollback_payload)
        .fetch_optional(pool)
        .await;

        match snapshot_res {
            Ok(Some(row)) => {
                let snap_id: i64 = row.try_get("snapshot_id").unwrap_or(0);
                println!(
                    "|- ORQUESTADOR MULTI-VENDOR -| [BLAST RADIUS SNAPSHOT] Snapshot de reversión {} guardado con éxito en BD (Snapshot ID: [{}], Task ID: [{}], Estado: PENDING_MANUAL)",
                    layer_target, snap_id, task_id
                );
            }
            Ok(None) => {}
            Err(e) => {
                eprintln!(
                    "|- ORQUESTADOR MULTI-VENDOR -| [WARN] No se pudo guardar rollback_snapshots en BD para mitigación manual: {}",
                    e
                );
            }
        }

        Some(task_id)
    } else {
        None
    }
}

// * * * 7. ORQUESTADOR Y ENRUTADOR PRINCIPAL DE ACCIONES DE MITIGACIÓN * * *
pub async fn dispatch_quarantine(pool: &PgPool, ip: &str, mac: &str) -> Result<Vendor, String> {
    // 1. Resolver dirección MAC real de Capa 2 desde la base de datos si vino genérica
    let resolved_mac = if mac.is_empty() || mac == "00:00:00:00:00:00" {
        get_mac_by_ip(pool, ip).await.unwrap_or_else(|| mac.to_string())
    } else {
        mac.to_string()
    };

    let mut vendor = detect_vendor_by_ip(pool, ip).await;

    // Priorizar aislamiento de Capa 2 vía RADIUS cuando se detecte un nodo en la topología local,
    // usándolo antes de recurrir a políticas perimetrales de Capa 3, excepto si es DNS Sinkhole o NGFW específico.
    if (vendor == Vendor::Unknown || is_local_topology(ip))
        && vendor != Vendor::Cisco
        && vendor != Vendor::OpenWrt
        && vendor != Vendor::MikroTik
        && vendor != Vendor::NGFW
        && vendor != Vendor::DnsSinkhole
        && vendor != Vendor::AclBlackhole
    {
        println!(
            "|- ORQUESTADOR MULTI-VENDOR -| [CAPA 2 PRIORITARIA] Nodo IP [{}] localizado en topología local. Priorizando aislamiento L2 (RADIUS CoA) sobre perimetral L3.",
            ip
        );
        vendor = Vendor::SwitchRadius;
    }

    let blast_radius = evaluate_blast_radius(vendor);

    println!(
        "|- ORQUESTADOR MULTI-VENDOR -| Fabricante / Estrategia seleccionada para IP [{}]: [{}] | Blast Radius: [{}]",
        ip,
        vendor.as_str(),
        blast_radius.as_str()
    );

    // 2. Registrar el evento en security_alerts antes de la mitigación
    let alert_id = register_security_alert(pool, ip, &resolved_mac, vendor).await;

    // 3. Ejecución de la acción de mitigación según el fabricante y evaluación de Blast Radius
    match vendor {
        Vendor::AclBlackhole => {
            println!(
                "|- ORQUESTADOR MULTI-VENDOR -| [CAPAS 3 Y 4 AUTOMÁTICA] Blast Radius singular (Host IP: [{}]). Ejecutando Blackholing (Null Route) y ACL dinámicas.",
                ip
            );
            let client = AclBlackhole::default();
            let rollback_json = client.apply_full_mitigation(ip, &resolved_mac).await?;

            if let Some(aid) = alert_id {
                save_rollback_snapshot(pool, aid, ip, &resolved_mac, "L3_L4", "ACL_BLACKHOLE_DYNAMIC_INJECTION", rollback_json).await;
            } else {
                println!(
                    "|- ORQUESTADOR MULTI-VENDOR -| [ROLLBACK] Snapshot ACL/Blackhole generado localmente (sin persistencia BD por falta de alert_id)."
                );
            }
        }
        Vendor::DnsSinkhole => {
            println!(
                "|- ORQUESTADOR MULTI-VENDOR -| [BLAST RADIUS EVALUADO: IMPACTO MASIVO] Mitigación Capa 7 (DNS Sinkholing) afecta globalmente la resolución de la red."
            );
            println!(
                "|- ORQUESTADOR MULTI-VENDOR -| [DETENCIÓN POR BLAST RADIUS] Acción NO ejecutada automáticamente para prevenir denegación de servicio colateral."
            );
            println!(
                "|- ORQUESTADOR MULTI-VENDOR -| [ENCOLANDO ACCIÓN MANUAL] Registrando en execution_queue como 'PENDING_MANUAL' y generando Rollback JSON."
            );
            let client = DnsSinkhole::default();
            let rollback_json = client.generate_rollback_payload(ip, &resolved_mac);

            if let Some(aid) = alert_id {
                queue_pending_manual_mitigation(
                    pool,
                    aid,
                    ip,
                    &resolved_mac,
                    "L7",
                    "DNS_SINKHOLE_L7_INTERCEPTION",
                    rollback_json,
                )
                .await;
            } else {
                println!(
                    "|- ORQUESTADOR MULTI-VENDOR -| [ROLLBACK] Snapshot L7 DNS generado localmente (sin persistencia BD por falta de alert_id): {}",
                    rollback_json
                );
            }
        }
        Vendor::SwitchRadius => {
            println!(
                "|- ORQUESTADOR MULTI-VENDOR -| [CAPA 2 AUTOMÁTICA] Impacto acotado a 1 host (IP: [{}], MAC: [{}]). Ejecutando automáticamente aislamiento vía RADIUS CoA.",
                ip, resolved_mac
            );
            let client = RadiusClient::default();
            let rollback_json = client.send_disconnect_request(ip, &resolved_mac).await?;

            if let Some(aid) = alert_id {
                save_rollback_snapshot(pool, aid, ip, &resolved_mac, "L2", "ISOLATE_L2_RADIUS_COA", rollback_json).await;
            } else {
                println!(
                    "|- ORQUESTADOR MULTI-VENDOR -| [ROLLBACK] Snapshot L2 generado localmente (sin persistencia BD por falta de alert_id)."
                );
            }
        }
        Vendor::NGFW => {
            println!(
                "|- ORQUESTADOR MULTI-VENDOR -| [PERIMETRAL AUTOMÁTICO] Blast Radius singular (1 IP atacante: [{}]). Ejecutando automáticamente bloqueo RESTCONF/YANG.",
                ip
            );
            let client = NgfwClient::default();
            let rollback_json = client.block_ip(ip, &resolved_mac).await?;

            if let Some(aid) = alert_id {
                save_rollback_snapshot(pool, aid, ip, &resolved_mac, "L3", "BLOCK_L3_NGFW_RESTCONF", rollback_json).await;
            } else {
                println!(
                    "|- ORQUESTADOR MULTI-VENDOR -| [ROLLBACK] Snapshot NGFW generado localmente (sin persistencia BD por falta de alert_id)."
                );
            }
        }
        Vendor::Cisco => {
            let client = CiscoIse::default();
            client.quarantine_ip(ip, &resolved_mac).await?;
        }
        Vendor::GNS3 => {
            let client = Gns3Alpine::default();
            client.quarantine_ip(ip, &resolved_mac).await?;
        }
        Vendor::OpenWrt => {
            let client = OpenWrtRouter::default();
            client.quarantine_ip(ip, &resolved_mac).await?;
        }
        Vendor::MikroTik => {
            let client = MikroTikRouter::default();
            client.quarantine_ip(ip, &resolved_mac).await?;
        }
        Vendor::Unknown => {
            println!(
                "|- ORQUESTADOR MULTI-VENDOR -| [WARN] Fabricante no identificado en BD para IP [{}]. Aplicando mitigación perimetral por defecto en GNS3.",
                ip
            );
            let client = Gns3Alpine::default();
            client.quarantine_ip(ip, &resolved_mac).await?;
        }
    }

    Ok(vendor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vendor_from_str() {
        assert_eq!(Vendor::from_str("Switch RADIUS"), Ok(Vendor::SwitchRadius));
        assert_eq!(Vendor::from_str("SwitchRadius"), Ok(Vendor::SwitchRadius));
        assert_eq!(Vendor::from_str("radius"), Ok(Vendor::SwitchRadius));
        assert_eq!(Vendor::from_str("radius_coa"), Ok(Vendor::SwitchRadius));
        assert_eq!(Vendor::from_str("switch"), Ok(Vendor::SwitchRadius));
        assert_eq!(Vendor::from_str("NGFW"), Ok(Vendor::NGFW));
        assert_eq!(Vendor::from_str("RESTCONF"), Ok(Vendor::NGFW));
        assert_eq!(Vendor::from_str("FortiGate"), Ok(Vendor::NGFW));
        assert_eq!(Vendor::from_str("PaloAlto"), Ok(Vendor::NGFW));
        assert_eq!(Vendor::from_str("DNS Sinkhole"), Ok(Vendor::DnsSinkhole));
        assert_eq!(Vendor::from_str("dns_sinkhole"), Ok(Vendor::DnsSinkhole));
        assert_eq!(Vendor::from_str("DnsSinkhole"), Ok(Vendor::DnsSinkhole));
        assert_eq!(Vendor::from_str("sinkhole"), Ok(Vendor::DnsSinkhole));
        assert_eq!(Vendor::from_str("dns"), Ok(Vendor::DnsSinkhole));
        assert_eq!(Vendor::from_str("AclBlackhole"), Ok(Vendor::AclBlackhole));
        assert_eq!(Vendor::from_str("blackhole"), Ok(Vendor::AclBlackhole));
        assert_eq!(Vendor::from_str("nullroute"), Ok(Vendor::AclBlackhole));
        assert_eq!(Vendor::from_str("acl"), Ok(Vendor::AclBlackhole));
        assert_eq!(Vendor::from_str("Cisco ISE"), Ok(Vendor::Cisco));
        assert_eq!(Vendor::from_str("cisco"), Ok(Vendor::Cisco));
        assert_eq!(Vendor::from_str("MikroTik RouterOS"), Ok(Vendor::MikroTik));
        assert_eq!(Vendor::from_str("routerboard"), Ok(Vendor::MikroTik));
        assert_eq!(Vendor::from_str("OpenWrt"), Ok(Vendor::OpenWrt));
        assert_eq!(Vendor::from_str("GNS3 Alpine"), Ok(Vendor::GNS3));
        assert_eq!(Vendor::from_str("Docker"), Ok(Vendor::GNS3));
        assert_eq!(Vendor::from_str("Invalido"), Ok(Vendor::Unknown));
    }

    #[test]
    fn test_blast_radius_evaluation() {
        assert_eq!(evaluate_blast_radius(Vendor::DnsSinkhole), BlastRadius::MassiveManual);
        assert_eq!(evaluate_blast_radius(Vendor::AclBlackhole), BlastRadius::SingularAuto);
        assert_eq!(evaluate_blast_radius(Vendor::SwitchRadius), BlastRadius::SingularAuto);
        assert_eq!(evaluate_blast_radius(Vendor::NGFW), BlastRadius::SingularAuto);
        assert_eq!(evaluate_blast_radius(Vendor::Cisco), BlastRadius::SingularAuto);
    }

    #[tokio::test]
    async fn test_dispatch_quarantine_mock() {
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/tigo_soc".to_string());
        if let Ok(pool) = sqlx::PgPool::connect(&db_url).await {
            // Asegurar nodo Cisco de prueba
            let _ = sqlx::query(
                "INSERT INTO network_nodes (hostname, ip_address, vendor)
                 VALUES ('TestCisco', '192.168.100.50', 'Cisco ISE')
                 ON CONFLICT (ip_address) DO UPDATE SET vendor = 'Cisco ISE'"
            )
            .execute(&pool)
            .await;

            let res_cisco = dispatch_quarantine(&pool, "192.168.100.50", "00:11:22:33:44:55").await;
            assert_eq!(res_cisco, Ok(Vendor::Cisco));

            // Asegurar nodo Switch RADIUS de prueba
            let _ = sqlx::query(
                "INSERT INTO network_nodes (hostname, ip_address, vendor)
                 VALUES ('TestSwitchRadius', '192.168.1.50', 'Switch RADIUS')
                 ON CONFLICT (ip_address) DO UPDATE SET vendor = 'Switch RADIUS'"
            )
            .execute(&pool)
            .await;

            let res_radius = dispatch_quarantine(&pool, "192.168.1.50", "00:11:22:33:44:55").await;
            assert_eq!(res_radius, Ok(Vendor::SwitchRadius));

            // Asegurar nodo NGFW RESTCONF de prueba
            let _ = sqlx::query(
                "INSERT INTO network_nodes (hostname, ip_address, vendor)
                 VALUES ('TestNgfw', '192.168.100.1', 'NGFW')
                 ON CONFLICT (ip_address) DO UPDATE SET vendor = 'NGFW'"
            )
            .execute(&pool)
            .await;

            let res_ngfw = dispatch_quarantine(&pool, "192.168.100.1", "00:11:22:33:44:99").await;
            assert_eq!(res_ngfw, Ok(Vendor::NGFW));

            // Asegurar nodo DNS Sinkhole de prueba
            let _ = sqlx::query(
                "INSERT INTO network_nodes (hostname, ip_address, vendor)
                 VALUES ('TestDnsSinkhole', '192.168.100.200', 'DNS Sinkhole')
                 ON CONFLICT (ip_address) DO UPDATE SET vendor = 'DNS Sinkhole'"
            )
            .execute(&pool)
            .await;

            let res_dns = dispatch_quarantine(&pool, "192.168.100.200", "00:11:22:33:44:88").await;
            assert_eq!(res_dns, Ok(Vendor::DnsSinkhole));

            // Asegurar nodo ACL & Blackholing de prueba
            let _ = sqlx::query(
                "INSERT INTO network_nodes (hostname, ip_address, vendor)
                 VALUES ('TestAclBlackhole', '192.168.100.210', 'AclBlackhole')
                 ON CONFLICT (ip_address) DO UPDATE SET vendor = 'AclBlackhole'"
            )
            .execute(&pool)
            .await;

            let res_blackhole = dispatch_quarantine(&pool, "192.168.100.210", "00:11:22:33:44:77").await;
            assert_eq!(res_blackhole, Ok(Vendor::AclBlackhole));
        }
    }
}
