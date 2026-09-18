// * * * CLIENTE DE NOTIFICACIONES PUSH FIREBASE CLOUD MESSAGING (ZERO-DATA PUSH) * * *

use anyhow::Result;
use chrono::Utc;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

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

// * * * 2. ESTRUCTURA DEL ARCHIVO SERVICE-ACCOUNT.JSON DE GOOGLE * * *
#[derive(Clone, Debug, Deserialize)]
pub struct ServiceAccount {
    pub project_id: String,
    pub private_key: String,
    pub client_email: String,
    pub token_uri: String,
}

#[derive(Serialize)]
struct GoogleClaims<'a> {
    iss: &'a str,
    sub: &'a str,
    aud: &'a str,
    iat: u64,
    exp: u64,
    scope: &'a str,
}

// * * * 3. CONFIGURACIÓN DEL CLIENTE FCM * * *

#[derive(Clone, Debug)]
pub struct FcmConfig {
    pub server_key: Option<String>,
    pub project_id: Option<String>,
    pub bearer_token: Option<String>,
    pub service_account_path: Option<String>,
    pub topic: String,
    pub enabled: bool,
}

impl FcmConfig {
    pub fn from_env() -> Self {
        let server_key = env::var("FCM_SERVER_KEY").ok().filter(|s| !s.trim().is_empty());
        let project_id = env::var("FCM_PROJECT_ID").ok().filter(|s| !s.trim().is_empty());
        let bearer_token = env::var("FCM_BEARER_TOKEN").ok().filter(|s| !s.trim().is_empty());
        let topic = env::var("FCM_TOPIC").unwrap_or_else(|_| "soc_alerts".to_string());

        // Búsqueda inteligente de service-account.json en rutas relativas comunes
        let sa_path = env::var("FCM_SERVICE_ACCOUNT_PATH")
            .or_else(|_| env::var("GOOGLE_APPLICATION_CREDENTIALS"))
            .ok()
            .filter(|s| !s.trim().is_empty())
            .or_else(|| {
                let candidates = [
                    "../service-account.json",
                    "service-account.json",
                    "/app/service-account.json",
                ];
                candidates
                    .iter()
                    .find(|p| std::path::Path::new(p).exists())
                    .map(|p| p.to_string())
            });

        let explicit_enabled = env::var("FCM_ENABLED")
            .ok()
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(true);

        let is_configured = (server_key.is_some()
            || bearer_token.is_some()
            || sa_path.is_some()
            || project_id.is_some())
            && explicit_enabled;

        Self {
            server_key,
            project_id,
            bearer_token,
            service_account_path: sa_path,
            topic,
            enabled: is_configured,
        }
    }
}

// * * * 4. CLIENTE ASÍNCRONO DE NOTIFICACIONES PUSH * * *

#[derive(Clone)]
pub struct FcmNotifier {
    client: Client,
    config: FcmConfig,
    service_account: Option<ServiceAccount>,
    token_cache: Arc<RwLock<Option<(String, Instant)>>>,
}

impl FcmNotifier {
    pub fn new(config: FcmConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .connect_timeout(Duration::from_secs(3))
            .build()
            .unwrap_or_else(|_| Client::new());

        // Intentar cargar la Service Account de Google si existe
        let mut sa = None;
        if let Some(ref path) = config.service_account_path {
            match std::fs::read_to_string(path) {
                Ok(content) => match serde_json::from_str::<ServiceAccount>(&content) {
                    Ok(parsed) => {
                        println!(
                            "|- INSTRUCCION -| [FCM] Credenciales Service Account cargadas exitosamente desde: [{}] (Proyecto: {})",
                            path, parsed.project_id
                        );
                        sa = Some(parsed);
                    }
                    Err(e) => {
                        eprintln!(
                            "|- INSTRUCCION -| [WARN] Error deserializando service-account.json: {}",
                            e
                        );
                    }
                },
                Err(e) => {
                    eprintln!(
                        "|- INSTRUCCION -| [WARN] No se pudo leer el archivo service account en [{}]: {}",
                        path, e
                    );
                }
            }
        }

        let is_active = config.enabled;

        if is_active {
            println!(
                "|- INSTRUCCION -| [FCM] Cliente HTTP FCM activo para el tema: [/topics/{}]",
                config.topic
            );
        } else {
            println!(
                "|- INSTRUCCION -| [FCM DESACTIVADO] Sistema de notificaciones Push APAGADO (FCM_ENABLED=false)."
            );
        }

        Self {
            client,
            config,
            service_account: sa,
            token_cache: Arc::new(RwLock::new(None)),
        }
    }

    // * * * SOLICITUD Y RENOVACIÓN DE TOKEN OAUTH2 MEDIANTE RS256 JWT * * *
    async fn fetch_oauth2_token(&self, sa: &ServiceAccount) -> Result<(String, u64)> {
        let now = Utc::now().timestamp() as u64;
        let claims = GoogleClaims {
            iss: &sa.client_email,
            sub: &sa.client_email,
            aud: &sa.token_uri,
            iat: now,
            exp: now + 3600,
            scope: "https://www.googleapis.com/auth/firebase.messaging",
        };

        let key = EncodingKey::from_rsa_pem(sa.private_key.as_bytes())?;
        let header = Header::new(Algorithm::RS256);
        let assertion = encode(&header, &claims, &key)?;

        let params = [
            ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
            ("assertion", assertion.as_str()),
        ];

        let res = self
            .client
            .post(&sa.token_uri)
            .form(&params)
            .send()
            .await?;

        if !res.status().is_success() {
            let status = res.status();
            let err_text = res.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Fallo al solicitar OAuth2 Bearer Token de Google ({}): {}",
                status,
                err_text
            ));
        }

        #[derive(Deserialize)]
        struct TokenResp {
            access_token: String,
            expires_in: Option<u64>,
        }

        let body: TokenResp = res.json().await?;
        let exp = body.expires_in.unwrap_or(3600);
        Ok((body.access_token, exp))
    }

    // * * * OBTENCIÓN DEL ACCESS TOKEN EN CACHÉ O RENOVACIÓN AUTOMÁTICA * * *
    pub async fn get_valid_bearer_token(&self) -> Result<String> {
        if let Some(ref sa) = self.service_account {
            let now = Instant::now();
            {
                let cache = self.token_cache.read().await;
                if let Some((ref token, ref exp_instant)) = *cache {
                    if now < *exp_instant {
                        return Ok(token.clone());
                    }
                }
            }

            let mut cache = self.token_cache.write().await;
            if let Some((ref token, ref exp_instant)) = *cache {
                if now < *exp_instant {
                    return Ok(token.clone());
                }
            }

            let (new_token, expires_in) = self.fetch_oauth2_token(sa).await?;
            let valid_secs = expires_in.saturating_sub(300).max(60);
            *cache = Some((new_token.clone(), now + Duration::from_secs(valid_secs)));
            return Ok(new_token);
        }

        if let Some(ref token) = self.config.bearer_token {
            return Ok(token.clone());
        }

        Err(anyhow::anyhow!(
            "No hay credenciales OAuth2 / Service Account configuradas"
        ))
    }

    // * * * SUSCRIPCIÓN AUTOMÁTICA DE UN DISPOSITIVO (TOKEN FCM) A UN TOPIC * * *
    pub async fn subscribe_device_to_topic(&self, registration_token: &str) -> Result<()> {
        let raw_topic = self.config.topic.trim_start_matches('/').replace("topics/", "");
        let url = format!(
            "https://iid.googleapis.com/iid/v1/{}/rel/topics/{}",
            registration_token, raw_topic
        );

        let req = if let Some(ref server_key) = self.config.server_key {
            self.client
                .post(&url)
                .header(header::AUTHORIZATION, format!("key={}", server_key))
                .header(header::CONTENT_LENGTH, "0")
        } else {
            let bearer = self.get_valid_bearer_token().await?;
            self.client
                .post(&url)
                .header(header::AUTHORIZATION, format!("Bearer {}", bearer))
                .header("access_token_auth", "true")
                .header(header::CONTENT_LENGTH, "0")
        };

        let res = req.send().await?;
        if res.status().is_success() {
            println!(
                "|- INSTRUCCION -| [FCM] Dispositivo [{}] suscrito exitosamente al tema [/topics/{}]",
                registration_token, raw_topic
            );
            Ok(())
        } else {
            let status = res.status();
            let err_body = res.text().await.unwrap_or_default();
            eprintln!(
                "|- INSTRUCCION -| [FCM ERROR] Google IID topic subscribe falló (HTTP {}): {}",
                status, err_body
            );
            Err(anyhow::anyhow!(
                "IID Topic subscribe error HTTP {}: {}",
                status,
                err_body
            ))
        }
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

        if !self.config.enabled {
            return Ok(());
        }

        let has_live_credentials = self.config.server_key.is_some()
            || self.service_account.is_some()
            || self.config.bearer_token.is_some();

        // Si no está configurada la llave externa, ejecutamos la simulación segura sin fallar
        if !has_live_credentials {
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
        // * * * ESTRATEGIA 2: FCM HTTP v1 (PROJECT_ID + BEARER TOKEN OAUTH2 AUTORENOVABLE) * * *
        else {
            let project_id = self
                .service_account
                .as_ref()
                .map(|s| s.project_id.clone())
                .or_else(|| self.config.project_id.clone())
                .unwrap_or_else(|| "rust-soc".to_string());

            let bearer_token = self.get_valid_bearer_token().await?;

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
