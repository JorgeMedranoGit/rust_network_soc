use crate::domain::models::NetworkEvent;

pub struct ThreatDetector {
    pub anomaly_threshold: f32,
}

impl ThreatDetector {
    pub fn new(threshold: f32) -> Self {
        Self {
            anomaly_threshold: threshold,
        }
    }

    pub fn evaluate_event(&self, event: &NetworkEvent) -> bool {
        if let Some(score) = event.anomaly_score {
            if score >= self.anomaly_threshold {
                println!("[DOMAIN ALERT] ¡Amenaza detectada! IP Origen: {} -> IP Destino: {} con puntaje: {}", 
                    event.source_ip, event.dest_ip, score);
                return true;
            }
        }
        false
    }
}