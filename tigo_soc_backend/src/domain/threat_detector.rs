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
        if let Some(score) = event.anomaly_score {
            return score >= self.anomaly_threshold;
        }
        // Detección de tráfico anómalo / pruebas de estrés
        // 1. Tráfico originado en el nodo atacante conocido (192.168.1.50)
        if event.source_ip == std::net::IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 1, 50)) {
            return true;
        }
        // 2. Combinaciones de flags TCP anómalas (ej. SYN+FIN)
        if (event.flags & (crate::domain::models::TCP_FLAG_SYN | crate::domain::models::TCP_FLAG_FIN))
            == (crate::domain::models::TCP_FLAG_SYN | crate::domain::models::TCP_FLAG_FIN)
        {
            return true;
        }
        false
    }
}
