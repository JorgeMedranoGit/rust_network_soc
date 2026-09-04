/// * * * MODULO PARA VISUALIZAR LOS EVENTOS DE RED * * *
pub struct TelemetryStreamHandler {
    pub active_stream: bool,
}

impl TelemetryStreamHandler {
    pub fn new() -> Self {
        Self {
            active_stream: true,
        }
    }

    pub fn notify_event(&self, message: &str) {
        if self.active_stream {
            println!("Evento de red canalizado: {}", message);
        }
    }
}