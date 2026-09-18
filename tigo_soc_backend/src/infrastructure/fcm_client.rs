// * * * CLIENTE DE NOTIFICACIONES PUSH FIREBASE CLOUD MESSAGING (ZERO-DATA PUSH) * * *

use anyhow::Result;
use chrono::Utc;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::time::Duration;

// * * * 1. MODELO DE PAYLOAD OPACO "ZERO-DATA PUSH" (COMPLIANCE DEVSECOPS & PRIVACIDAD) * * *
// En conformidad con estándares de seguridad Carrier-Grade y Zero-Trust (GDPR, ISO 27001):
// NUNCA se transmiten direcciones IP, payloads de paquetes, hostnames de víctimas ni topología interna
// a través de servicios Push de terceros (Google FCM). Se envía un identificador opaco para que el
// operador o técnico descargue la evidencia autenticándose directamente contra la API interna del SOC.

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ZeroDataPayload {
    pub alert_id: String,
    pub event_type: String,
    pub threat_category: String,
    pub severity: String,
    pub timestamp: String,
    pub verification_token: String,
}

// * * * 2. CONFIGURACIÓN DEL CLIENTE FCM * * *

#[derive(Clone, Debug)]
pub struct FcmConfig {
    pub server_key: Option<String>,
    pub project_id: Option<String>,
    pub bearer_token: Option<String>,
    pub topic: String,
    pub enabled: bool,
}

impl FcmConfig {
    pub fn from_env() -> Self {
        let server_key = env::var("FCM_SERVER_KEY").ok().filter(|s| !s.trim().is_empty());
        let project_id = env::var("FCM_PROJECT_ID").ok().filter(|s| !s.trim().is_empty());
        let bearer_token = env::var("FCM_BEARER_TOKEN").ok().filter(|s| !s.trim().is_empty());
        let topic = env::var("FCM_TOPIC").unwrap_or_else(|_| "soc_alerts".to_string());
        let is_configured = server_key.is_some() || bearer_token.is_some() || project_id.is_some();

        Self {
            server_key,
            project_id,
            bearer_token,
            topic,
            enabled: is_configured,
        }
    }
}

// * * * 3. CLIENTE ASÍNCRONO DE NOTIFICACIONES PUSH * * *

#[derive(Clone)]
pub struct FcmNotifier {
    client: Client,
    config: FcmConfig,
}

impl FcmNotifier {
    pub fn new(config: FcmConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .connect_timeout(Duration::from_secs(3))
            .build()
            .unwrap_or_else(|_| Client::new());

        if config.enabled {
            println!(
                "|- INSTRUCCION -| [FCM] Cliente HTTP FCM inicializado para el tema: [/topics/{}]",
                config.topic
            );
        } else {
            println!(
                "|- INSTRUCCION -| [FCM] Modo Simulación ACTIVO (Configure FCM_SERVER_KEY o FCM_BEARER_TOKEN para envío en vivo)."
            );
        }

        Self { client, config }
    }

    // * * * ENVÍO DEL WEBHOOK CON PAYLOAD OPACO AL TEMA /topics/soc_alerts * * *
    pub async fn send_opaque_alert(
        &self,
        alert_id: i64,
        severity: &str,
        threat_category: &str,
    ) -> Result<()> {
        let timestamp = Utc::now().to_rfc3339();
        let topic_target = format!("/topics/{}", self.config.topic.trim_start_matches('/'));

        // Generación de firma opaca de integridad (token de verificación)
        let raw_signature = format!("{}:{}:{}:tigo_soc_salt", alert_id, threat_category, timestamp);
        let verification_token = md5::compute_hex(raw_signature.as_bytes());

        let payload = ZeroDataPayload {
            alert_id: alert_id.to_string(),
            event_type: "SECURITY_ALERT".to_string(),
            threat_category: threat_category.to_string(),
            severity: severity.to_string(),
            timestamp,
            verification_token,
        };

        // Si no está configurada la llave externa, ejecutamos la simulación segura sin fallar
        if !self.config.enabled {
            println!(
                "|- INSTRUCCION -| [FCM SIMULACIÓN] Zero-Data Push generado para [{}]: Alert ID #{}, Categoría: {}, Severidad: {}",
                topic_target, payload.alert_id, payload.threat_category, payload.severity
            );
            return Ok(());
        }

        // * * * ESTRATEGIA 1: FCM HTTP LEGACY (SERVER KEY) * * *
        if let Some(ref server_key) = self.config.server_key {
            let url = "https://fcm.googleapis.com/fcm/send";
            let body = json!({
                "to": topic_target,
                "priority": "high",
                "data": {
                    "alertId": payload.alert_id,
                    "eventType": payload.event_type,
                    "threatCategory": payload.threat_category,
                    "severity": payload.severity,
                    "timestamp": payload.timestamp,
                    "verificationToken": payload.verification_token
                },
                "notification": {
                    "title": format!("🚨 [Tigo SOC] Incidente #{}", payload.alert_id),
                    "body": format!("Amenaza: {} | Severidad: {}. Consulte la consola para análisis forense.", payload.threat_category, payload.severity),
                    "icon": "/icons/icon-192.png",
                    "click_action": "/"
                }
            });

            let response = self
                .client
                .post(url)
                .header(header::AUTHORIZATION, format!("key={}", server_key))
                .header(header::CONTENT_TYPE, "application/json")
                .json(&body)
                .send()
                .await;

            match response {
                Ok(res) if res.status().is_success() => {
                    let text = res.text().await.unwrap_or_default();
                    println!(
                        "|- INSTRUCCION -| [FCM] Zero-Data Push entregado con éxito a [{}]. Alert ID #{}. Respuesta: {}",
                        topic_target, alert_id, text
                    );
                    Ok(())
                }
                Ok(res) => {
                    let status = res.status();
                    let err_body = res.text().await.unwrap_or_default();
                    eprintln!(
                        "|- INSTRUCCION -| [FCM ERROR] FCM Legacy respondió con HTTP {}: {}",
                        status, err_body
                    );
                    Err(anyhow::anyhow!("FCM Legacy error HTTP {}: {}", status, err_body))
                }
                Err(e) => {
                    eprintln!("|- INSTRUCCION -| [FCM ERROR] Fallo de conexión de red hacia FCM: {}", e);
                    Err(e.into())
                }
            }
        }
        // * * * ESTRATEGIA 2: FCM HTTP v1 (PROJECT_ID + BEARER TOKEN OAUTH2) * * *
        else if let (Some(ref project_id), Some(ref bearer_token)) =
            (&self.config.project_id, &self.config.bearer_token)
        {
            let url = format!(
                "https://fcm.googleapis.com/v1/projects/{}/messages:send",
                project_id
            );
            let raw_topic = self.config.topic.trim_start_matches('/').replace("topics/", "");
            let body = json!({
                "message": {
                    "topic": raw_topic,
                    "data": {
                        "alertId": payload.alert_id,
                        "eventType": payload.event_type,
                        "threatCategory": payload.threat_category,
                        "severity": payload.severity,
                        "timestamp": payload.timestamp,
                        "verificationToken": payload.verification_token
                    },
                    "notification": {
                        "title": format!("🚨 [Tigo SOC] Incidente #{}", payload.alert_id),
                        "body": format!("Amenaza: {} | Severidad: {}. Consulte la consola para análisis forense.", payload.threat_category, payload.severity)
                    }
                }
            });

            let response = self
                .client
                .post(&url)
                .header(header::AUTHORIZATION, format!("Bearer {}", bearer_token))
                .header(header::CONTENT_TYPE, "application/json")
                .json(&body)
                .send()
                .await;

            match response {
                Ok(res) if res.status().is_success() => {
                    let text = res.text().await.unwrap_or_default();
                    println!(
                        "|- INSTRUCCION -| [FCM v1] Zero-Data Push entregado con éxito a [{}]. Alert ID #{}. Resp: {}",
                        topic_target, alert_id, text
                    );
                    Ok(())
                }
                Ok(res) => {
                    let status = res.status();
                    let err_body = res.text().await.unwrap_or_default();
                    eprintln!(
                        "|- INSTRUCCION -| [FCM ERROR] FCM v1 respondió con HTTP {}: {}",
                        status, err_body
                    );
                    Err(anyhow::anyhow!("FCM v1 error HTTP {}: {}", status, err_body))
                }
                Err(e) => {
                    eprintln!("|- INSTRUCCION -| [FCM ERROR] Fallo de red hacia FCM v1: {}", e);
                    Err(e.into())
                }
            }
        } else {
            Err(anyhow::anyhow!("Configuración de credenciales FCM incompleta"))
        }
    }
}

// Implementación de cálculo de hash simplificado localmente para la firma de integridad
mod md5 {
    pub fn compute_hex(data: &[u8]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;
        let mut hasher = DefaultHasher::new();
        hasher.write(data);
        let h1 = hasher.finish();
        hasher.write(b"tigo_soc_salt");
        let h2 = hasher.finish();
        format!("{:016x}{:016x}", h1, h2)
    }
}
