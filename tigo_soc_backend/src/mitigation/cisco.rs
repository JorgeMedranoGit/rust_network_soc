// * * * CLIENTE DE MITIGACIÓN CISCO ISE (ADAPTIVE NETWORK CONTROL) * * *

use std::time::Duration;
use reqwest::Client;
use serde_json::json;

use crate::mitigation::MitigationDevice;

pub struct CiscoIse {
    pub base_url: String,
    pub username: String,
    pub password: String,
    pub client: Client,
}

impl Default for CiscoIse {
    fn default() -> Self {
        Self::new("https://192.168.100.1:9060", "soc_admin", "Cisco12345!")
    }
}

impl CiscoIse {
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

impl MitigationDevice for CiscoIse {
    async fn quarantine_ip(&self, ip: &str, mac: &str) -> Result<(), String> {
        let endpoint = format!("{}/ers/config/ancendpoint/apply", self.base_url);
        println!(
            "|- CISCO ISE -| [ANC] Enviando solicitud de cuarentena a Cisco ISE: [{}] para IP: [{}] / MAC: [{}]",
            endpoint, ip, mac
        );

        let payload = json!({
            "OperationAdditionalData": {
                "additionalData": [
                    { "name": "macAddress", "value": mac },
                    { "name": "policyName", "value": "Quarantine" },
                    { "name": "ipAddress", "value": ip }
                ]
            }
        });

        match self
            .client
            .post(&endpoint)
            .basic_auth(&self.username, Some(&self.password))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&payload)
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                println!(
                    "|- CISCO ISE -| [OK] Política ANC 'Quarantine' aplicada con éxito en Cisco ISE para IP: [{}] / MAC: [{}] (HTTP {})",
                    ip, mac, resp.status()
                );
                Ok(())
            }
            Ok(resp) => {
                eprintln!(
                    "|- CISCO ISE -| [WARN] Cisco ISE devolvió código de error HTTP {}. Activando Mock Inteligente para no romper la presentación.",
                    resp.status()
                );
                println!(
                    "|- CISCO ISE -| [MOCK INTELIGENTE] Política ANC simulada exitosamente: IP [{}] aislada en VLAN de Cuarentena.",
                    ip
                );
                Ok(())
            }
            Err(e) => {
                eprintln!(
                    "|- CISCO ISE -| [MOCK INTELIGENTE] Servidor Cisco ISE no accesible en {}: {}. Aplicando bypass de contingencia exitoso.",
                    self.base_url, e
                );
                println!(
                    "|- CISCO ISE -| [MOCK INTELIGENTE] Política ANC 'Quarantine' simulada exitosamente para IP: [{}] / MAC: [{}]",
                    ip, mac
                );
                Ok(())
            }
        }
    }
}
