// * * * MÓDULO DE MITIGACIÓN MULTI-VENDOR Y ARQUITECTURA AGNÓSTICA DE HARDWARE * * *
#![allow(async_fn_in_trait, dead_code)]

pub mod cisco;
pub mod gns3;
pub mod mikrotik;
pub mod openwrt;
pub mod radius_coa;

use std::str::FromStr;
use sqlx::{PgPool, Row};

pub use cisco::CiscoIse;
pub use gns3::Gns3Alpine;
pub use mikrotik::MikroTikRouter;
pub use openwrt::OpenWrtRouter;
pub use radius_coa::RadiusClient;

// * * * 1. TRAIT CENTRAL PARA DISPOSITIVOS DE MITIGACIÓN AGNÓSTICOS * * *
pub trait MitigationDevice: Send + Sync {
    async fn quarantine_ip(&self, ip: &str, mac: &str) -> Result<(), String>;
}

// * * * 2. ENUM DE MARCAS Y FABRICANTES SOPORTADOS * * *
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vendor {
    SwitchRadius,
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

// * * * 4. ORQUESTADOR Y ENRUTADOR PRINCIPAL DE ACCIONES DE MITIGACIÓN * * *
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

    match vendor {
        Vendor::SwitchRadius => {
            println!(
                "|- ORQUESTADOR MULTI-VENDOR -| [CAPA 2] Ejecutando aislamiento RADIUS CoA para IP [{}] / MAC [{}]",
                ip, resolved_mac
            );
            let client = RadiusClient::default();
            client.quarantine_ip(ip, &resolved_mac).await?;
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
        }
    }
}
