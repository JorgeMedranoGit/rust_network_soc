pub struct NetworkAdapter {
    pub interface_name: String,
}

impl NetworkAdapter {
    pub fn new(interface_name: &str) -> Self {
        Self {
            interface_name: interface_name.to_string(),
        }
    }

    pub async fn start_listening(&self) {
        println!("[INFRASTRUCTURE] Adaptador de red inicializado en la interfaz: {}", self.interface_name);
        println!("[INFRASTRUCTURE] Listo para capturar tráfico de GNS3 (Preparado para la fase de Polars/LightGBM).");
    }
}