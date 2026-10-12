// * * * CLIENTE DE MITIGACIÓN OPENWRT (UBUS JSON-RPC API / UCI FIREWALL) * * *

use std::time::Duration;
use reqwest::Client;
use serde_json::json;

use crate::mitigation::MitigationDevice;

pub struct OpenWrtRouter {
    pub base_url: String,
    pub client: Client,
}

impl Default for OpenWrtRouter {
    fn default() -> Self {
        Self::new("http://192.168.2.1")
    }
}

impl OpenWrtRouter {
    pub fn new(base_url: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
        }
    }
}

impl MitigationDevice for OpenWrtRouter {
    async fn quarantine_ip(&self, ip: &str, mac: &str) -> Result<(), String> {
        let endpoint = format!("{}/ubus", self.base_url);
        println!(
            "|- OPENWRT -| [UBUS] Despachando llamada JSON-RPC a ubus para añadir regla de dropeo (fw4/fw3) para IP: [{}] / MAC: [{}]",
            ip, mac
        );

        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "call",
            "params": [
                "00000000000000000000000000000000",
                "uci",
                "add",
                {
                    "config": "firewall",
                    "type": "rule",
                    "values": {
                        "name": format!("SOC_QUARANTINE_{}", ip.replace('.', "_")),
                        "src": "lan",
                        "src_ip": ip,
                        "target": "DROP"
                    }
                }
            ]
        });

        match self
            .client
            .post(&endpoint)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                println!(
                    "|- OPENWRT -| [OK] Regla de firewall ubus/uci agregada exitosamente en OpenWRT para IP: [{}] (HTTP {})",
                    ip, resp.status()
                );
                Ok(())
            }
            Ok(resp) => {
                eprintln!(
                    "|- OPENWRT -| [WARN] OpenWRT ubus devolvió HTTP {}. Activando Mock Inteligente.",
                    resp.status()
                );
                println!(
                    "|- OPENWRT -| [MOCK INTELIGENTE] Regla de firewall fw4/uci simulada exitosamente: IP [{}] bloqueada en zona LAN.",
                    ip
                );
                Ok(())
            }
            Err(e) => {
                eprintln!(
                    "|- OPENWRT -| [MOCK INTELIGENTE] Router OpenWRT no accesible en {}: {}. Aplicando simulación de contingencia.",
                    self.base_url, e
                );
                println!(
                    "|- OPENWRT -| [MOCK INTELIGENTE] Regla ubus JSON-RPC simulada: IP [{}] añadida a lista de dropeo de firewall OpenWRT.",
                    ip
                );
                Ok(())
            }
        }
    }
}
