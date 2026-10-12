// * * * MOTOR DE DESCUBRIMIENTO PASIVO DE ACTIVOS (PASSIVE ASSET DISCOVERY & OUI RESOLUTION) * * *

use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use sqlx::PgPool;
use tokio::sync::mpsc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredAsset {
    pub ip: String,
    pub mac: String,
    pub vendor: String,
}

/// Resolución rápida OUI comparando los primeros 3 bytes de la dirección MAC
pub fn resolve_oui(mac: &[u8; 6]) -> &'static str {
    let prefix = (mac[0], mac[1], mac[2]);

    match prefix {
        // Cisco Systems OUIs conocidas
        (0x00, 0x00, 0x0c)
        | (0x00, 0x01, 0x42) | (0x00, 0x01, 0x43) | (0x00, 0x01, 0x63) | (0x00, 0x01, 0x64)
        | (0x00, 0x01, 0x96) | (0x00, 0x01, 0x97) | (0x00, 0x0a, 0xb7) | (0x00, 0x0a, 0xb8)
        | (0x00, 0x0b, 0xfc) | (0x00, 0x0c, 0x30) | (0x00, 0x0c, 0x85) | (0x00, 0x0d, 0xbc)
        | (0x00, 0x0e, 0x38) | (0x00, 0x11, 0x20) | (0x00, 0x11, 0x21) | (0x00, 0x11, 0x22)
        | (0x00, 0x11, 0xbb) | (0x00, 0x11, 0xbc) | (0x00, 0x12, 0x00) | (0x00, 0x12, 0x01)
        | (0x00, 0x1b, 0x0c) | (0x00, 0x1b, 0x0d) | (0x00, 0x1b, 0x2a) | (0x00, 0x1b, 0x53)
        | (0x00, 0x1b, 0xd4) | (0x00, 0x1b, 0xd5) | (0x00, 0x40, 0x96) | (0x00, 0x50, 0x0f)
        | (0x00, 0x60, 0x09) | (0x00, 0x60, 0x2f) | (0x00, 0x60, 0x3e) | (0x00, 0x60, 0x47)
        | (0x00, 0xe0, 0x14) | (0x00, 0xe0, 0x1e) | (0x00, 0xe0, 0x34) | (0x00, 0xe0, 0x4f)
        | (0x28, 0xac, 0x9e) | (0x50, 0x0f, 0x80) | (0x70, 0x81, 0x05) | (0xc4, 0x72, 0x95)
        | (0xf4, 0x4f, 0x82) => "Cisco ISE",

        // MikroTik RouterOS OUIs
        (0x00, 0x0c, 0x42) // Routerboard.com oficial
        | (0x48, 0x8f, 0x5a) | (0x64, 0xd1, 0x54) | (0x74, 0x4d, 0x28) | (0xcc, 0x2d, 0xe0)
        | (0xdc, 0x2c, 0x6e) | (0xe4, 0x8d, 0x8c) | (0x08, 0x55, 0x31) | (0x18, 0xfd, 0x74)
        | (0x2c, 0xc8, 0x1b) | (0x4c, 0x5e, 0x0c) | (0x6c, 0x3b, 0x6b) | (0xb8, 0x69, 0xf4)
        | (0xc4, 0xad, 0x34) | (0xd4, 0x01, 0xc3) => "MikroTik RouterOS",

        // OpenWRT OUIs comunes y dispositivos embebidos
        (0x00, 0x18, 0x0a) | (0xa4, 0x2b, 0xb0) | (0x00, 0x19, 0xe0) | (0x04, 0xa1, 0x51)
        | (0x00, 0x14, 0xbf) | (0xd8, 0x58, 0xd7) | (0xc0, 0x4a, 0x00) => "OpenWrt",

        // GNS3 / Docker / Emuladores y Virtualización
        (0x02, 0x42, 0xac) // Docker bridge estándar
        | (0x52, 0x54, 0x00) // QEMU / KVM
        | (0x08, 0x00, 0x27) // VirtualBox
        | (0x00, 0x50, 0x79) // GNS3 VPCS / Dynamips
        | (0x00, 0x0c, 0x29) | (0x00, 0x50, 0x56) | (0x00, 0x05, 0x69) // VMware
        => "GNS3 Alpine",

        // Bit Locally Administered Address (bit 1 del octeto más significativo)
        (b0, _, _) if (b0 & 0x02) != 0 => "GNS3 Alpine",

        _ => "Desconocido",
    }
}

pub fn format_mac(mac: &[u8; 6]) -> String {
    format!(
        "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    )
}

#[derive(Clone)]
pub struct AssetDiscoveryEngine {
    // Caché en RAM de IPs observadas para no generar contención ni queries redundantes
    seen_cache: Arc<RwLock<HashSet<String>>>,
    tx: mpsc::Sender<DiscoveredAsset>,
}

impl AssetDiscoveryEngine {
    pub fn new(buffer_size: usize) -> (Self, mpsc::Receiver<DiscoveredAsset>) {
        let (tx, rx) = mpsc::channel(buffer_size);
        (
            Self {
                seen_cache: Arc::new(RwLock::new(HashSet::with_capacity(1024))),
                tx,
            },
            rx,
        )
    }

    /// Observación no bloqueante y de ultra-bajo overhead ejecutada desde el sniffer
    pub fn observe_asset(&self, ip: &str, mac: [u8; 6]) {
        // Fast path: verificación atómica en RAM (<20ns) con candado de lectura
        if let Ok(guard) = self.seen_cache.read() {
            if guard.contains(ip) {
                return;
            }
        }

        // Slow path: registrar en caché con candado de escritura y notificar al canal asíncrono
        if let Ok(mut guard) = self.seen_cache.write() {
            if !guard.insert(ip.to_string()) {
                return;
            }
        }

        let mac_str = format_mac(&mac);
        let vendor = resolve_oui(&mac);

        let asset = DiscoveredAsset {
            ip: ip.to_string(),
            mac: mac_str,
            vendor: vendor.to_string(),
        };

        let _ = self.tx.try_send(asset);
    }

    /// Worker asíncrono en segundo plano para persistir en PostgreSQL mediante UPSERT
    pub fn spawn_db_persister(mut rx: mpsc::Receiver<DiscoveredAsset>, pool: PgPool) {
        tokio::spawn(async move {
            println!("|- ASSET DISCOVERY -| [WORKER] Escuchador de nuevos activos pasivos iniciado.");
            while let Some(asset) = rx.recv().await {
                let hostname = format!("node-{}", asset.ip.replace('.', "-"));
                println!(
                    "|- ASSET DISCOVERY -| [OUI] Nuevo activo descubierto: IP [{}] | MAC [{}] | Fabricante [{}]",
                    asset.ip, asset.mac, asset.vendor
                );

                let res = sqlx::query(
                    "INSERT INTO network_nodes (hostname, ip_address, mac_address, vendor)
                     VALUES ($1, $2, $3, $4)
                     ON CONFLICT (ip_address) DO UPDATE SET
                         mac_address = EXCLUDED.mac_address,
                         vendor = EXCLUDED.vendor"
                )
                .bind(&hostname)
                .bind(&asset.ip)
                .bind(&asset.mac)
                .bind(&asset.vendor)
                .execute(&pool)
                .await;

                if let Err(e) = res {
                    eprintln!("|- ASSET DISCOVERY -| [DB ERROR] Falló el registro de {}: {}", asset.ip, e);
                }
            }
        });
    }
}
