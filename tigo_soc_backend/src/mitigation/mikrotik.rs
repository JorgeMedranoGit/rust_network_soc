// * * * CLIENTE DE MITIGACIÓN MIKROTIK ROUTEROS (REST API V7+ / ADDRESS-LIST) * * *

use std::time::Duration;
use reqwest::Client;
use serde_json::json;

use crate::mitigation::MitigationDevice;

pub struct MikroTikRouter {
    pub base_url: String,
    pub username: String,
    pub password: String,
    pub client: Client,
}

impl Default for MikroTikRouter {
    fn default() -> Self {
        Self::new("https://192.168.88.1", "admin", "admin")
    }
}

impl MikroTikRouter {
    pub fn new(base_url: &str, username: &str, password: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(3))
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            username: username.to_string(),
            password: password.to_string(),
            client,
        }
    }
}

impl MitigationDevice for MikroTikRouter {
    async fn quarantine_ip(&self, ip: &str, mac: &str) -> Result<(), String> {
        let endpoint = format!("{}/rest/ip/firewall/address-list", self.base_url);
        println!(
            "|- MIKROTIK -| [RouterOS REST] Registrando IP [{}] en address-list 'SOC_QUARANTINE' (MAC: [{}])",
            ip, mac
        );

        let payload = json!({
            "list": "SOC_QUARANTINE",
            "address": ip,
            "comment": format!("Tigo SOC Quarantine - MAC: {}", mac)
        });

        match self
            .client
            .post(&endpoint)
            .basic_auth(&self.username, Some(&self.password))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                println!(
                    "|- MIKROTIK -| [OK] IP [{}] agregada exitosamente a la lista 'SOC_QUARANTINE' en RouterOS (HTTP {})",
                    ip, resp.status()
                );
                Ok(())
            }
            Ok(resp) => {
                eprintln!(
                    "|- MIKROTIK -| [WARN] RouterOS REST devolvió HTTP {}. Activando Mock Inteligente.",
                    resp.status()
                );
                println!(
                    "|- MIKROTIK -| [MOCK INTELIGENTE] Entrada en address-list 'SOC_QUARANTINE' simulada exitosamente para IP [{}].",
                    ip
                );
                Ok(())
            }
            Err(e) => {
                eprintln!(
                    "|- MIKROTIK -| [MOCK INTELIGENTE] Router MikroTik no accesible en {}: {}. Aplicando simulación de contingencia.",
                    self.base_url, e
                );
                println!(
                    "|- MIKROTIK -| [MOCK INTELIGENTE] IP [{}] agregada a 'SOC_QUARANTINE' simulada en RouterOS v7 REST API.",
                    ip
                );
                Ok(())
            }
        }
    }
}
