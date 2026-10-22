// * * * TRANSMISOR DE TELEMETRÍA EN TIEMPO REAL (BROADCAST CHANNEL) * * *
#![allow(dead_code)]
use tokio::sync::broadcast;
use crate::domain::models::NetworkEvent;

#[derive(Clone)]
pub struct TelemetryStreamHandler {
    sender: broadcast::Sender<NetworkEvent>,
}

impl TelemetryStreamHandler {
    // * * * CREAR CANAL DE TRANSMISIÓN DE EVENTOS DE RED * * *
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    // * * * EMITIR EVENTO A TODOS LOS CONSUMIDORES CONECTADOS * * *
    pub fn broadcast_event(&self, event: NetworkEvent) {
        let _ = self.sender.send(event);
    }

    // * * * SUSCRIPCIÓN AL FLUJO DE TELEMETRÍA EN VIVO * * *
    pub fn subscribe(&self) -> broadcast::Receiver<NetworkEvent> {
        self.sender.subscribe()
    }
}
