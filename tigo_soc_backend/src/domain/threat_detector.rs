// * * * MOTOR DE INFERENCIA Y EVALUACIÓN DE AMENAZAS LIGHTGBM * * *

use anyhow::Result;
use lightgbm3::Booster;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::domain::models::{ExtractedFeatures, ThreatEvaluation};

pub struct BoosterWrapper(pub Booster);
unsafe impl Send for BoosterWrapper {}
unsafe impl Sync for BoosterWrapper {}

#[derive(Debug, Default, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreatDetectorStats {
    pub total_evaluations: u64,
    pub total_anomalies: u64,
    pub avg_feature_time_us: f64,
    pub avg_inference_time_us: f64,
    pub avg_total_time_us: f64,
    pub min_inference_time_us: f64,
    pub max_inference_time_us: f64,
    pub last_probability: f32,
    pub last_latency_us: f64,
}

#[derive(Clone)]
pub struct ThreatDetector {
    booster: Arc<BoosterWrapper>,
    pub anomaly_threshold: f32,
    evaluations_counter: Arc<AtomicU64>,
    anomalies_counter: Arc<AtomicU64>,
    cum_feature_time_ns: Arc<AtomicU64>,
    cum_inference_time_ns: Arc<AtomicU64>,
    stats_mutex: Arc<Mutex<(f64, f64, f32, f64)>>,
}

impl ThreatDetector {
    // * * * CARGAR MODELO PRE-ENTRENADO LIGHTGBM DESDE ARCHIVO TXT * * *
    pub fn new_from_file(model_path: &str, threshold: f32) -> Result<Self> {
        let path = if fs::metadata(model_path).is_ok() {
            model_path.to_string()
        } else if fs::metadata(format!("/app/{}", model_path)).is_ok() {
            format!("/app/{}", model_path)
        } else if fs::metadata("models/mejor_modelo_kfold.txt").is_ok() {
            "models/mejor_modelo_kfold.txt".to_string()
        } else if fs::metadata("../ModelTrainedLightGBM/core_engine/models/mejor_modelo_kfold.txt").is_ok() {
            "../ModelTrainedLightGBM/core_engine/models/mejor_modelo_kfold.txt".to_string()
        } else {
            return Err(anyhow::anyhow!(
                "No se encontró el modelo LightGBM en '{}' ni en rutas estándar",
                model_path
            ));
        };

        println!("|- INSTRUCCION -| [ML] Cargando modelo LightGBM pre-entrenado desde: [{}]", path);
        let booster = Booster::from_file(&path)
            .map_err(|e| anyhow::anyhow!("Error al deserializar booster LightGBM: {:?}", e))?;

        println!("|- INSTRUCCION -| [ML] Modelo LightGBM cargado con éxito en RAM (23 características, K-Fold validado).");

        Ok(Self {
            booster: Arc::new(BoosterWrapper(booster)),
            anomaly_threshold: threshold,
            evaluations_counter: Arc::new(AtomicU64::new(0)),
            anomalies_counter: Arc::new(AtomicU64::new(0)),
            cum_feature_time_ns: Arc::new(AtomicU64::new(0)),
            cum_inference_time_ns: Arc::new(AtomicU64::new(0)),
            stats_mutex: Arc::new(Mutex::new((f64::MAX, 0.0, 0.0, 0.0))),
        })
    }

    // * * * INFERENCIA ULTRA-RÁPIDA CON CRONOMETRAJE EN SUB-MICROSEGUNDOS * * *
    pub fn evaluate_features(
        &self,
        features: &ExtractedFeatures,
        source_ip: &str,
        feature_time_us: f64,
    ) -> ThreatEvaluation {
        let feature_vector = features.to_vector();
        let inf_start = Instant::now();

        let preds = self
            .booster
            .0
            .predict(&feature_vector, 23, false)
            .unwrap_or_else(|_| vec![0.0]);

        let probability = preds.first().copied().unwrap_or(0.0) as f32;
        let inf_elapsed_ns = inf_start.elapsed().as_nanos() as u64;
        let inf_time_us = inf_elapsed_ns as f64 / 1000.0;
        let total_time_us = feature_time_us + inf_time_us;

        // * * * ACTUALIZAR CONTADORES ATÓMICOS DE RENDIMIENTO * * *
        let _ev_count = self.evaluations_counter.fetch_add(1, Ordering::Relaxed) + 1;
        self.cum_feature_time_ns
            .fetch_add((feature_time_us * 1000.0) as u64, Ordering::Relaxed);
        self.cum_inference_time_ns
            .fetch_add(inf_elapsed_ns, Ordering::Relaxed);

        if let Ok(mut lock) = self.stats_mutex.lock() {
            if inf_time_us < lock.0 {
                lock.0 = inf_time_us;
            }
            if inf_time_us > lock.1 {
                lock.1 = inf_time_us;
            }
            lock.2 = probability;
            lock.3 = inf_time_us;
        }

        // * * * CALIBRACIÓN BAYESIANA DE PRIOR (COMPENSACIÓN DE SESGO DE DATASET) * * *
        let calibrated_score = if probability <= 0.9914 {
            ((probability - 0.980).max(0.0) * 5.0).min(0.25)
        } else {
            0.50 + ((probability - 0.9914) / (1.0 - 0.9914)).min(1.0) * 0.50
        };

        let is_attack = calibrated_score >= self.anomaly_threshold;
        if is_attack {
            self.anomalies_counter.fetch_add(1, Ordering::Relaxed);
        }

        // * * * POLÍTICA DE CATEGORIZACIÓN HEURÍSTICA Y REGLAS CARRIER TIGO * * *
        let (threat_name, threat_id, description, tech_details) = Self::classify_threat(features, calibrated_score);
        let (severity, impact, resolution) = Self::evaluate_policy(source_ip, calibrated_score, is_attack);

        ThreatEvaluation {
            is_attack,
            probability: calibrated_score,
            threat_name,
            threat_id,
            severity,
            impact,
            resolution,
            description,
            technical_details: tech_details,
            features: feature_vector,
            feature_time_us,
            inference_time_us: inf_time_us,
            total_time_us,
        }
    }

    // * * * EVALUACIÓN DE POLÍTICAS DE MITIGACIÓN Y SLA CARRIER * * *
    fn evaluate_policy(source_ip: &str, prob: f32, is_attack: bool) -> (String, String, String) {
        if !is_attack {
            return ("LOW".to_string(), "LOW".to_string(), "PASS".to_string());
        }
        let is_carrier = source_ip.starts_with("186.")
            || source_ip.starts_with("190.")
            || source_ip.starts_with("200.");
        let severity = if prob > 0.85 {
            "CRITICAL"
        } else if prob > 0.65 {
            "HIGH"
        } else {
            "MEDIUM"
        };
        let impact = if is_carrier { "HIGH" } else { "LOW" };
        let resolution = if is_carrier {
            "MANUAL_REQUIRED"
        } else {
            "AUTO_MITIGATE"
        };
        (severity.to_string(), impact.to_string(), resolution.to_string())
    }

    // * * * CLASIFICACIÓN GRANULAR DE AMENAZAS DETECTADAS * * *
    fn classify_threat(features: &ExtractedFeatures, _prob: f32) -> (String, i32, String, String) {
        if features.rate > 800.0 || features.syn_count > 50.0 {
            (
                "DDOS_SYN_FLOOD".to_string(),
                3,
                "Inundación masiva de tráfico detectada (DoS/DDoS)".to_string(),
                format!("Tasa de paquetes: {:.1} PPS | SYN: {:.0}", features.rate, features.syn_count),
            )
        } else if features.tot_size > 5000.0 || (features.tot_sum > 2000.0 && features.avg_size > 1000.0) {
            (
                "DATA_EXFILTRATION".to_string(),
                1,
                "Fuga o exfiltración volumétrica anómala de datos".to_string(),
                format!("Bytes transferidos: {:.0} bytes | AVG: {:.1}", features.tot_size, features.avg_size),
            )
        } else if features.rst_count > 20.0 || (features.rate > 100.0 && features.tot_size < 1000.0) {
            (
                "PORT_SCAN".to_string(),
                2,
                "Escaneo de puertos / Reconocimiento activo".to_string(),
                format!("RST count: {:.0} | PPS: {:.1}", features.rst_count, features.rate),
            )
        } else {
            (
                "UNAUTHORIZED_ACCESS".to_string(),
                4,
                "Comportamiento anómalo en canal de gestión / C2 Beaconing".to_string(),
                format!("IAT: {:.4}s | Varianza: {:.1}", features.iat, features.variance),
            )
        }
    }

    // * * * OBTENER ESTADÍSTICAS CONSOLIDADAS DE RENDIMIENTO * * *
    pub fn get_performance_stats(&self) -> ThreatDetectorStats {
        let evs = self.evaluations_counter.load(Ordering::Relaxed);
        let anoms = self.anomalies_counter.load(Ordering::Relaxed);
        let cum_feat = self.cum_feature_time_ns.load(Ordering::Relaxed);
        let cum_inf = self.cum_inference_time_ns.load(Ordering::Relaxed);

        let (min_us, max_us, last_prob, last_us) = match self.stats_mutex.lock() {
            Ok(lock) => *lock,
            Err(_) => (0.0, 0.0, 0.0, 0.0),
        };

        let count_f = if evs > 0 { evs as f64 } else { 1.0 };
        let avg_feat_us = (cum_feat as f64 / count_f) / 1000.0;
        let avg_inf_us = (cum_inf as f64 / count_f) / 1000.0;

        ThreatDetectorStats {
            total_evaluations: evs,
            total_anomalies: anoms,
            avg_feature_time_us: avg_feat_us,
            avg_inference_time_us: avg_inf_us,
            avg_total_time_us: avg_feat_us + avg_inf_us,
            min_inference_time_us: if min_us == f64::MAX { 0.0 } else { min_us },
            max_inference_time_us: max_us,
            last_probability: last_prob,
            last_latency_us: last_us,
        }
    }
}
