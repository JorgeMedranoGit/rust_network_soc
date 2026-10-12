// * * * MÓDULO DE MITIGACIÓN MULTI-VENDOR Y ARQUITECTURA AGNÓSTICA DE HARDWARE * * *
#![allow(async_fn_in_trait, dead_code)]

pub mod cisco;
pub mod gns3;
pub mod mikrotik;
pub mod ngfw_restconf;
pub mod openwrt;
pub mod radius_coa;

use std::str::FromStr;
use serde_json::Value;
use sqlx::{PgPool, Row};

pub use cisco::CiscoIse;
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
    let threat_id = threat_id_opt.unwrap_or(4);

    // 2. Estado inicial: CONFIRMED_THREAT (2) o PENDING_ANALYSIS (1)
    let status_id_opt: Option<i32> = sqlx::query_scalar(
        "SELECT status_id FROM alert_statuses WHERE status_name = 'CONFIRMED_THREAT' LIMIT 1"
    )
    .fetch_optional(pool)
    .await
    .unwrap_or(None);
    let status_id = status_id_opt.unwrap_or(2);

    let anomaly_score: f64 = 0.95;

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

// * * * 6. ORQUESTADOR Y ENRUTADOR PRINCIPAL DE ACCIONES DE MITIGACIÓN * * *
pub async fn dispatch_quarantine(pool: &PgPool, ip: &str, mac: &str) -> Result<Vendor, String> {
    // 1. Resolver dirección MAC real de Capa 2 desde la base de datos si vino genérica
    let resolved_mac = if mac.is_empty() || mac == "00:00:00:00:00:00" {
        get_mac_by_ip(pool, ip).await.unwrap_or_else(|| mac.to_string())
    } else {
        mac.to_string()
    };

    let mut vendor = detect_vendor_by_ip(pool, ip).await;

    // Priorizar aislamiento de Capa 2 vía RADIUS cuando se detecte un nodo en la topología local,
    // usándolo antes de recurrir a políticas perimetrales de Capa 3.
    if (vendor == Vendor::Unknown || is_local_topology(ip))
        && vendor != Vendor::Cisco
        && vendor != Vendor::OpenWrt
        && vendor != Vendor::MikroTik
        && vendor != Vendor::NGFW
    {
        println!(
            "|- ORQUESTADOR MULTI-VENDOR -| [CAPA 2 PRIORITARIA] Nodo IP [{}] localizado en topología local. Priorizando aislamiento L2 (RADIUS CoA) sobre perimetral L3.",
            ip
        );
        vendor = Vendor::SwitchRadius;
    }

    println!(
        "|- ORQUESTADOR MULTI-VENDOR -| Fabricante / Estrategia seleccionada para IP [{}]: [{}]",
        ip,
        vendor.as_str()
    );

    // 2. Registrar el evento en security_alerts antes de la mitigación
    let alert_id = register_security_alert(pool, ip, &resolved_mac, vendor).await;

    // 3. Ejecución de la acción de mitigación según el fabricante
    match vendor {
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
        assert_eq!(Vendor::from_str("Cisco ISE"), Ok(Vendor::Cisco));
        assert_eq!(Vendor::from_str("cisco"), Ok(Vendor::Cisco));
        assert_eq!(Vendor::from_str("MikroTik RouterOS"), Ok(Vendor::MikroTik));
        assert_eq!(Vendor::from_str("routerboard"), Ok(Vendor::MikroTik));
        assert_eq!(Vendor::from_str("OpenWrt"), Ok(Vendor::OpenWrt));
        assert_eq!(Vendor::from_str("GNS3 Alpine"), Ok(Vendor::GNS3));
        assert_eq!(Vendor::from_str("Docker"), Ok(Vendor::GNS3));
        assert_eq!(Vendor::from_str("Invalido"), Ok(Vendor::Unknown));
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
        }
    }
}
