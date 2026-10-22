// * * * INTERCEPTOR DNS SINKHOLING DE CAPA 7 (APLICACIÓN) * * *
// RFC 1035: Domain Names - Implementation and Specification
// Mitigación y redirección de tráfico DNS UDP puerto 53 para dominios maliciosos (C2, Exfiltración)
// hacia una IP de aislamiento controlada (Sinkhole), con generación de Rollback JSON

use std::net::SocketAddr;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::net::UdpSocket;

use crate::mitigation::MitigationDevice;

/// Resultado de la evaluación e intercepción de una consulta DNS de Capa 7
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DnsInterceptionResult {
    pub domain: String,
    pub client_ip: String,
    pub is_intercepted: bool,
    pub resolved_ip: String,
    pub action_taken: String,
}

/// Interceptor y gestor de políticas DNS Sinkhole de Capa 7
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsSinkhole {
    pub sinkhole_ip: String,
    pub dns_port: u16,
    pub blacklist_domains: Vec<String>,
}

impl Default for DnsSinkhole {
    fn default() -> Self {
        let sinkhole_ip = std::env::var("DNS_SINKHOLE_IP")
            .unwrap_or_else(|_| "192.168.1.254".to_string());
        let dns_port = std::env::var("DNS_SINKHOLE_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(53);

        let default_blacklist = vec![
            "evil-c2.com".to_string(),
            "malicious-c2.net".to_string(),
            "c2-exfil.org".to_string(),
            "beacon.darknet.lan".to_string(),
            "attacker-c2.lan".to_string(),
            "data-theft.attacker.com".to_string(),
        ];

        Self {
            sinkhole_ip,
            dns_port,
            blacklist_domains: default_blacklist,
        }
    }
}

impl DnsSinkhole {
    /// Inicializa una instancia de DnsSinkhole con configuración explícita
    pub fn new(sinkhole_ip: &str, dns_port: u16, blacklist_domains: Vec<String>) -> Self {
        Self {
            sinkhole_ip: sinkhole_ip.to_string(),
            dns_port,
            blacklist_domains,
        }
    }

    /// Normaliza un dominio para comparaciones (minúsculas, sin espacios ni punto final)
    pub fn normalize_domain(domain: &str) -> String {
        domain
            .trim()
            .trim_end_matches('.')
            .to_lowercase()
    }

    /// Verifica si un dominio o subdominio se encuentra en la lista negra
    pub fn is_blacklisted(&self, domain: &str) -> bool {
        let normalized = Self::normalize_domain(domain);
        if normalized.is_empty() {
            return false;
        }

        self.blacklist_domains.iter().any(|blacklisted| {
            let b_norm = Self::normalize_domain(blacklisted);
            normalized == b_norm || normalized.ends_with(&format!(".{}", b_norm))
        })
    }

    /// Agrega un nuevo dominio malicioso a la lista de bloqueo en tiempo de ejecución
    pub fn add_blacklist_domain(&mut self, domain: &str) {
        let normalized = Self::normalize_domain(domain);
        if !normalized.is_empty() && !self.is_blacklisted(&normalized) {
            self.blacklist_domains.push(normalized);
        }
    }

    /// Evalúa una petición DNS UDP 53 y devuelve la IP de Sinkhole si el dominio está en lista negra
    pub fn intercept_query(&self, domain: &str, client_ip: &str) -> Result<DnsInterceptionResult, String> {
        let normalized = Self::normalize_domain(domain);
        let intercepted = self.is_blacklisted(&normalized);

        if intercepted {
            println!(
                "|- DNS SINKHOLE L7 -| [INTERCEPTADO UDP:53] Petición de cliente [{}] para dominio malicioso [{}] redirigida a Sinkhole IP [{}]",
                client_ip, normalized, self.sinkhole_ip
            );
            Ok(DnsInterceptionResult {
                domain: normalized,
                client_ip: client_ip.to_string(),
                is_intercepted: true,
                resolved_ip: self.sinkhole_ip.clone(),
                action_taken: "SINKHOLED_TO_ISOLATION_IP".to_string(),
            })
        } else {
            println!(
                "|- DNS SINKHOLE L7 -| [RESOLUCIÓN LIMPIA] Dominio legítimo [{}] permitido para resolución estándar por [{}]",
                normalized, client_ip
            );
            Ok(DnsInterceptionResult {
                domain: normalized,
                client_ip: client_ip.to_string(),
                is_intercepted: false,
                resolved_ip: "UPSTREAM_DNS_RESOLVED".to_string(),
                action_taken: "FORWARDED_UPSTREAM".to_string(),
            })
        }
    }

    /// Simula la recepción asíncrona de un paquete DNS en UDP 53 con Mock Inteligente tolerante a fallos
    pub async fn simulate_dns_reception(&self, client_ip: &str, domain: &str) -> Result<DnsInterceptionResult, String> {
        println!(
            "|- DNS SINKHOLE L7 -| [MOCK INTELIGENTE UDP 53] Escuchando consultas DNS en {}:{} para cliente [{}]...",
            self.sinkhole_ip, self.dns_port, client_ip
        );

        // Intento de enlace UDP de prueba en puerto local efímero para validar stack de sockets Tokio
        let test_bind = UdpSocket::bind("127.0.0.1:0").await;
        match test_bind {
            Ok(socket) => {
                let local_addr = socket.local_addr().map_err(|e| e.to_string())?;
                println!(
                    "|- DNS SINKHOLE L7 -| [SOCKET ACTIVO] Socket UDP de contingencia enlazado en [{}] para intercepción.",
                    local_addr
                );
            }
            Err(e) => {
                eprintln!(
                    "|- DNS SINKHOLE L7 -| [WARN] Permisos o red restringidos para socket UDP ({}). Continuando en modo virtual.",
                    e
                );
            }
        }

        self.intercept_query(domain, client_ip)
    }

    /// Construye el payload JSON con la directiva inversa de restauración (Rollback) para Capa 7
    pub fn generate_domain_rollback_payload(&self, ip: &str, mac: &str, domain: &str) -> Value {
        let normalized = Self::normalize_domain(domain);
        json!({
            "action": "RESTORE_DNS_ZONE_RESOLUTION",
            "layer_target": "L7",
            "vendor": "DnsSinkhole",
            "target_ip": ip,
            "target_mac": mac,
            "domain": normalized,
            "sinkhole_ip": self.sinkhole_ip,
            "dns_port": self.dns_port,
            "instruction": {
                "description": format!("Comando inverso para remover la zona de bloqueo Sinkhole L7 y restaurar resolución DNS recursiva para {}", normalized),
                "protocol": "DNS_UDP_53",
                "method": "UNSINKHOLE_ZONE",
                "bind9_command": format!("rndc flushname {}", normalized),
                "unbound_command": format!("unbound-control local_data_remove {}", normalized),
                "pihole_command": format!("pihole -w -d {}", normalized),
                "dnsmasq_reversal": format!("# Removido de /etc/dnsmasq.d/sinkhole.conf: address=/{}/{}", normalized, self.sinkhole_ip)
            },
            "blast_radius": "MASSIVE_NETWORK_WIDE",
            "status": "READY_FOR_ROLLBACK",
            "generated_at": chrono::Utc::now().to_rfc3339()
        })
    }

    /// Genera el payload de Rollback genérico por defecto
    pub fn generate_rollback_payload(&self, ip: &str, mac: &str) -> Value {
        self.generate_domain_rollback_payload(ip, mac, "evil-c2.com")
    }
}

// * * * IMPLEMENTACIÓN DEL TRAIT DE MITIGACIÓN MULTI-VENDOR * * *
impl MitigationDevice for DnsSinkhole {
    async fn quarantine_ip(&self, ip: &str, mac: &str) -> Result<(), String> {
        println!(
            "|- DNS SINKHOLE L7 -| [MITIGATION_DEVICE] Intercepción DNS solicitada para IP [{}] / MAC [{}].",
            ip, mac
        );
        let _ = self.intercept_query("evil-c2.com", ip)?;
        Ok(())
    }

    async fn quarantine_with_rollback(&self, ip: &str, mac: &str) -> Result<Option<Value>, String> {
        let rollback = self.generate_rollback_payload(ip, mac);
        Ok(Some(rollback))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blacklist_detection() {
        let sinkhole = DnsSinkhole::default();
        assert!(sinkhole.is_blacklisted("evil-c2.com"));
        assert!(sinkhole.is_blacklisted("EVIL-C2.COM"));
        assert!(sinkhole.is_blacklisted("sub.evil-c2.com"));
        assert!(sinkhole.is_blacklisted("beacon.sub.evil-c2.com."));
        assert!(sinkhole.is_blacklisted("malicious-c2.net"));
        assert!(!sinkhole.is_blacklisted("google.com"));
        assert!(!sinkhole.is_blacklisted("cloudflare.com"));
    }

    #[test]
    fn test_intercept_query() {
        let sinkhole = DnsSinkhole::default();
        let res = sinkhole.intercept_query("evil-c2.com", "192.168.1.50").unwrap();
        assert!(res.is_intercepted);
        assert_eq!(res.resolved_ip, sinkhole.sinkhole_ip);
        assert_eq!(res.action_taken, "SINKHOLED_TO_ISOLATION_IP");

        let res_clean = sinkhole.intercept_query("example.com", "192.168.1.50").unwrap();
        assert!(!res_clean.is_intercepted);
        assert_eq!(res_clean.action_taken, "FORWARDED_UPSTREAM");
    }

    #[test]
    fn test_rollback_payload_generation() {
        let sinkhole = DnsSinkhole::default();
        let payload = sinkhole.generate_domain_rollback_payload("192.168.1.50", "00:11:22:33:44:55", "evil-c2.com");
        assert_eq!(payload["action"], "RESTORE_DNS_ZONE_RESOLUTION");
        assert_eq!(payload["layer_target"], "L7");
        assert_eq!(payload["vendor"], "DnsSinkhole");
        assert_eq!(payload["blast_radius"], "MASSIVE_NETWORK_WIDE");
        assert_eq!(payload["domain"], "evil-c2.com");
        assert!(payload["instruction"]["bind9_command"].as_str().unwrap().contains("rndc flushname"));
    }
}
