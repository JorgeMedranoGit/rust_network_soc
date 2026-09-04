#![allow(dead_code)]
use tokio::sync::broadcast;
use crate::domain::models::NetworkEvent;

#[derive(Clone)]
pub struct TelemetryStreamHandler {
    sender: broadcast::Sender<NetworkEvent>,
}

impl TelemetryStreamHandler {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn broadcast_event(&self, event: NetworkEvent) {
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<NetworkEvent> {
        self.sender.subscribe()
    }
}
