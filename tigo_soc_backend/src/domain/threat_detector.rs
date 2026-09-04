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

    /// Evalúa el evento con soporte de inferencia rápida en CPU.
    #[inline]
    pub fn evaluate_event(&self, event: &NetworkEvent) -> bool {
        // Todo el tráfico se clasifica como normal hasta la integración del modelo LightGBM
        if let Some(score) = event.anomaly_score {
            return score >= self.anomaly_threshold;
        }
        false
    }
}
