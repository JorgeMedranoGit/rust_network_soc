mod domain;
mod infrastructure;
mod presentation;

use std::collections::HashMap;
use std::env;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use chrono::Utc;
use tokio::net::TcpListener;

use domain::{
    models::NetworkEvent,
    threat_detector::ThreatDetector,
};
use infrastructure::{
    db_connection,
    network_adapter::NetworkAdapter,
    repo_catalogs::CatalogsRepository,
    repo_inventory::InventoryRepository,
    repo_telemetry::TelemetryRepository,
};
use presentation::{
    api_routes::{create_router, AppState},
    telemetry_stream::TelemetryStreamHandler,
};

#[derive(Default, Debug, Clone, Copy)]
struct TrafficStats {
    packets: u64,
    bytes: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();

    println!("============================================================");
    println!(" [INFO] Iniciando Tigo SOC Backend (Pipeline Alto Rendimiento)");
    println!("============================================================");

    // 1. Conexión a Base de Datos
    let db_pool = match db_connection::init_pool().await {
        Ok(pool) => {
            println!("[INFO] Conexión a PostgreSQL establecida exitosamente.");
            pool
        }
        Err(e) => {
            eprintln!("[ERROR] Falló la conexión a PostgreSQL: {}", e);
            return Err(e.into());
        }
    };

    // 2. Inicializar y Sembrar Catálogos e Inventario de Topología
    let catalogs = CatalogsRepository::new(db_pool.clone());
    seed_initial_catalogs(&catalogs).await;

    let inventory = InventoryRepository::new(db_pool.clone());
    seed_initial_inventory(&inventory, &catalogs).await;

    // 3. Cargar Caché de Topología en Memoria (IP -> node_id para resolución O(1))
    let ip_cache = Arc::new(load_ip_cache(&inventory).await);
    println!("[INFO] Caché de topología en memoria cargada con {} nodos.", ip_cache.len());

    // 4. Canal de retransmisión de eventos
    let stream_handler = TelemetryStreamHandler::new(1024);

    // 5. Iniciar Captura de Red (Sniffer con Patrón Productor-Consumidor)
    let capture_iface = env::var("CAPTURE_INTERFACE").unwrap_or_else(|_| "any".to_string());
    let adapter = NetworkAdapter::new(&capture_iface);
    let mut rx_events = adapter.start_capture(65536);

    // 6. Canal desacoplado para Inferencia Aislada (LightGBM / Heurística)
    let (forensic_tx, mut forensic_rx) = tokio::sync::mpsc::channel::<NetworkEvent>(65536);

    let telemetry = Arc::new(TelemetryRepository::new(db_pool.clone()));

    // =========================================================================
    // WORKER DE INFERENCIA AISLADA (Ruta Forense - No bloquea la ingesta)
    // =========================================================================
    let telemetry_forensic = Arc::clone(&telemetry);
    let ip_cache_forensic = Arc::clone(&ip_cache);
    tokio::spawn(async move {
        let detector = ThreatDetector::new(0.80);
        println!("[INFERENCE WORKER] Motor de detección e inferencia activo en hilo aislado.");

        while let Some(event) = forensic_rx.recv().await {
            // Evaluación de ML / Reglas de umbral (Aislada del hilo de paquetes)
            if detector.evaluate_event(&event) {
                println!(
                    "[ALERTA FORENSE] Amenaza detectada: {} -> {} [{}] (Clasificado sospechoso)",
                    event.source_ip, event.destination_ip, event.protocol.as_str()
                );

                let node_id = ip_cache_forensic
                    .get(&event.source_ip)
                    .or_else(|| ip_cache_forensic.get(&event.destination_ip))
                    .copied();

                let telemetry_db = Arc::clone(&telemetry_forensic);
                let src_str = event.source_ip.to_string();
                let dst_str = event.destination_ip.to_string();
                let proto_str = event.protocol.as_str().to_string();
                let flags_str = event.flags_to_string();
                let psize = event.packet_size as i32;

                // Delegar el I/O hacia PostgreSQL a una tarea independiente sin frenar el worker
                tokio::spawn(async move {
                    if let Err(e) = telemetry_db
                        .insert_log(
                            node_id,
                            &src_str,
                            &dst_str,
                            &proto_str,
                            psize,
                            flags_str.as_deref(),
                        )
                        .await
                    {
                        eprintln!("[FORENSIC DB ERROR] Falló la inserción en network_logs: {}", e);
                    }
                });
            }
        }
    });

    // =========================================================================
    // CONSUMIDOR PRINCIPAL (Ruta Estadística con tokio::select! y std::mem::take)
    // =========================================================================
    let telemetry_metrics = Arc::clone(&telemetry);
    let ip_cache_main = Arc::clone(&ip_cache);
    let stream_consumer = stream_handler.clone();

    tokio::spawn(async move {
        println!("[CONSUMER] Ingestor de paquetes activo con agregación atómica en RAM.");

        // Acumulador en RAM por node_id (Option<i32>)
        let mut accumulator: HashMap<Option<i32>, TrafficStats> = HashMap::with_capacity(128);
        let mut window_start = Utc::now();

        // Temporizador de volcado cada 10 segundos
        let mut flush_interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
        flush_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        let mut packet_counter: u64 = 0;

        loop {
            tokio::select! {
                // RUTA A: Recepción continua de paquetes a máxima velocidad (Sub-microsegundo, 0 Heap Allocs)
                Some(event) = rx_events.recv() => {
                    packet_counter += 1;

                    // Resolución rápida en RAM O(1) de nodo de topología
                    let node_id = ip_cache_main
                        .get(&event.source_ip)
                        .or_else(|| ip_cache_main.get(&event.destination_ip))
                        .copied();

                    // Acumulación en RAM (Ruta Estadística)
                    let entry = accumulator.entry(node_id).or_default();
                    entry.packets += 1;
                    entry.bytes += event.packet_size as u64;

                    // Despacho no bloqueante a la ruta de inferencia forense
                    let _ = forensic_tx.try_send(event);

                    // Retransmisión de telemetría a clientes conectados
                    stream_consumer.broadcast_event(event);

                    if packet_counter % 500 == 1 {
                        println!(
                            "[SNIFFER] Ingesta: {} pkts | Último: {} -> {} [{}] ({} bytes)",
                            packet_counter,
                            event.source_ip,
                            event.destination_ip,
                            event.protocol.as_str(),
                            event.packet_size
                        );
                    }
                }

                // RUTA B: Volcado (Flush) periódico de la ventana estadística a PostgreSQL
                _ = flush_interval.tick() => {
                    let window_end = Utc::now();
                    // Vaciado atómico de memoria: transfiere propiedad y previene memory leaks
                    let batch = std::mem::take(&mut accumulator);
                    let current_start = window_start;
                    window_start = window_end;

                    if !batch.is_empty() {
                        let telemetry_db = Arc::clone(&telemetry_metrics);
                        // Desacoplar I/O de PostgreSQL en tarea secundaria para nunca bloquear la ingesta
                        tokio::spawn(async move {
                            for (node_id, stats) in batch {
                                if let Err(e) = telemetry_db
                                    .record_metric(
                                        node_id,
                                        current_start,
                                        window_end,
                                        stats.packets as i64,
                                        stats.bytes as i64,
                                    )
                                    .await
                                {
                                    eprintln!("[METRICS FLUSH ERROR] Falló volcado en network_traffic_metrics: {}", e);
                                }
                            }
                        });
                    }
                }
            }
        }
    });

    // 7. Iniciar Servidor Web Axum
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    let state = AppState {
        pool: db_pool.clone(),
        stream_handler,
    };

    let app = create_router(state);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(&addr).await?;

    println!("[INFO] Servidor API REST escuchando en http://{}", addr);
    println!("[INFO] Endpoints disponibles:");
    println!("       - GET /health");
    println!("       - GET /api/v1/status");
    println!("       - GET /api/v1/catalogs/device-types");
    println!("       - GET /api/v1/inventory/nodes");
    println!("       - GET /api/v1/telemetry/metrics");
    println!("       - GET /api/v1/telemetry/logs");
    println!("       - GET /api/v1/alerts");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn load_ip_cache(inventory: &InventoryRepository) -> HashMap<IpAddr, i32> {
    let mut map = HashMap::new();
    if let Ok(nodes) = inventory.get_nodes().await {
        for node in nodes {
            if let Some(nid) = node.node_id {
                if let Ok(ip) = IpAddr::from_str(&node.ip_address) {
                    map.insert(ip, nid);
                }
            }
        }
    }
    map
}

async fn seed_initial_inventory(
    inventory: &InventoryRepository,
    catalogs: &CatalogsRepository,
) {
    let dev_types = catalogs.get_device_types().await.unwrap_or_default();
    let protocols = catalogs.get_mgmt_protocols().await.unwrap_or_default();

    let get_tid = |name: &str| dev_types.iter().find(|d| d.type_name == name).and_then(|d| d.type_id);
    let get_pid = |name: &str| protocols.iter().find(|p| p.protocol_name == name).and_then(|p| p.protocol_id);

    let nodes = [
        ("AttackerKali", "192.168.0.100", get_tid("ATTACKER_KALI"), get_pid("SSH")),
        ("VictimeAlpine1", "192.168.0.101", get_tid("WORKSTATION_VICTIM"), get_pid("SSH")),
        ("VictimeAlpine2", "192.168.0.102", get_tid("WORKSTATION_VICTIM"), get_pid("SSH")),
        ("pfSense_Gateway", "192.168.0.1", get_tid("FIREWALL_PERIMETER"), get_pid("SSH")),
        ("Router_Core_L3", "192.168.0.254", get_tid("ROUTER_CORE_L3"), get_pid("SSH")),
    ];

    for (hostname, ip, type_id, proto_id) in nodes {
        let _ = inventory.create_node(hostname, ip, type_id, proto_id).await;
    }
}

async fn seed_initial_catalogs(repo: &CatalogsRepository) {
    println!("[INFO] Verificando y sembrando catálogos iniciales...");

    let device_types = [
        "ROUTER_CORE_L3",
        "FIREWALL_PERIMETER",
        "SWITCH_DISTRIBUTION",
        "WORKSTATION_VICTIM",
        "ATTACKER_KALI",
    ];
    for dt in device_types {
        let _ = repo.create_device_type(dt).await;
    }

    let protocols = ["SSH", "NETCONF", "RESTCONF", "SNMP"];
    for proto in protocols {
        let _ = repo.create_mgmt_protocol(proto).await;
    }

    let alert_statuses = [
        "PENDING_ANALYSIS",
        "CONFIRMED_THREAT",
        "FALSE_POSITIVE",
        "MITIGATED",
    ];
    for st in alert_statuses {
        let _ = repo.create_alert_status(st).await;
    }

    let task_statuses = ["QUEUED", "EXECUTING", "COMPLETED", "FAILED", "ROLLED_BACK"];
    for ts in task_statuses {
        let _ = repo.create_task_status(ts).await;
    }

    let threat_types = [
        ("DATA_EXFILTRATION", 5),
        ("PORT_SCAN", 2),
        ("DDOS_SYN_FLOOD", 4),
        ("UNAUTHORIZED_ACCESS", 3),
    ];
    for (name, sev) in threat_types {
        let _ = repo.create_threat_type(name, sev).await;
    }

    let roles = [
        ("SOC_ADMIN", Some("Administrador general del SOC")),
        ("SOC_ANALYST_L1", Some("Analista L1 - Monitoreo y triaje")),
        ("SOC_ANALYST_L2", Some("Analista L2 - Respuesta a incidentes")),
    ];
    for (name, desc) in roles {
        let _ = repo.create_role(name, desc).await;
    }

    println!("[INFO] Catálogos base verificados correctamente.");
}
