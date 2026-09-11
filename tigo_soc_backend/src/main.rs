mod domain;
mod infrastructure;
mod presentation;

use std::collections::{HashMap, VecDeque};
use std::env;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use chrono::Utc;
use tokio::net::TcpListener;

use domain::{
    feature_engine::PolarsFeatureEngine,
    ml_trainer::LightGBMTrainer,
    models::NetworkEvent,
    threat_detector::ThreatDetector,
};
use infrastructure::{
    db_connection,
    network_adapter::NetworkAdapter,
    repo_catalogs::CatalogsRepository,
    repo_inventory::InventoryRepository,
    repo_orchestration::OrchestrationRepository,
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

    // * * * 0. CLI Execution Modes (Server vs Training) * * *
    let args: Vec<String> = env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("server");

    match mode {
        "train" => {
            let data_arg = args.get(2).map(|s| s.as_str());
            println!("|- INFO -| Modo de entrenamiento seleccionado (K-Fold + Polars + LightGBM CPU).");
            LightGBMTrainer::run_training(false, data_arg)?;
            return Ok(());
        }
        "train-gpu" => {
            let data_arg = args.get(2).map(|s| s.as_str());
            println!("|- INFO -| Modo de entrenamiento GPU seleccionado.");
            LightGBMTrainer::run_training(true, data_arg)?;
            return Ok(());
        }
        "server" => {}
        other => {
            println!("|- WARN -| Argumento desconocido '{}'. Modos válidos: server | train | train-gpu", other);
        }
    }

    println!("|- INFO -| Iniciando Motor Core de TigoSOC Backend (Rust + Polars + LightGBM)...");

    // * * * 1. Database Connection * * *
    let db_pool = match db_connection::init_pool().await {
        Ok(pool) => {
            println!("|- DB -| Conexión establecida con éxito en PostgreSQL.");
            pool
        }
        Err(e) => {
            eprintln!("|- FATAL -| Falló la conexión a PostgreSQL: {}", e);
            return Err(e.into());
        }
    };

    // * * * 2. Initialize and Seed Catalogs & Topology Inventory * * *
    let catalogs = CatalogsRepository::new(db_pool.clone());
    seed_initial_catalogs(&catalogs).await;

    let inventory = InventoryRepository::new(db_pool.clone());
    seed_initial_inventory(&inventory, &catalogs).await;

    // * * * 3. Load Topology Cache into Memory (IP -> node_id for O(1) resolution) * * *
    let ip_cache = Arc::new(load_ip_cache(&inventory).await);
    println!("|- CACHE -| Topología cargada en memoria RAM: {} nodos indexados O(1).", ip_cache.len());

    // * * * 4. Load LightGBM Model & Feature Engine * * *
    let model_path = env::var("ML_MODEL_PATH").unwrap_or_else(|_| "models/mejor_modelo_kfold.txt".to_string());
    let ml_threshold = env::var("ML_THRESHOLD")
        .unwrap_or_else(|_| "0.50".to_string())
        .parse::<f32>()
        .unwrap_or(0.50);

    let threat_detector = match ThreatDetector::new_from_file(&model_path, ml_threshold) {
        Ok(td) => {
            println!("|- ML -| ThreatDetector inicializado con umbral de detección: [{:.2}]", ml_threshold);
            Some(td)
        }
        Err(e) => {
            eprintln!("|- WARN -| No se pudo cargar el modelo LightGBM: {}. El sistema continuará sin ML activo.", e);
            None
        }
    };

    // * * * 5. Event Broadcast Channel * * *
    let stream_handler = TelemetryStreamHandler::new(1024);

    // * * * 6. Start Network Capture (Sniffer with Producer-Consumer Pattern) * * *
    let capture_iface = env::var("CAPTURE_INTERFACE").unwrap_or_else(|_| "any".to_string());
    let adapter = NetworkAdapter::new(&capture_iface);
    let mut rx_events = adapter.start_capture(65536);
    println!("|- SNIFFER -| Escuchando interfaz de red activa: [{}]", capture_iface);

    // * * * 7. Decoupled Channel for Isolated Inference (LightGBM + Polars) * * *
    let (forensic_tx, mut forensic_rx) = tokio::sync::mpsc::channel::<NetworkEvent>(65536);

    let telemetry = Arc::new(TelemetryRepository::new(db_pool.clone()));
    let orchestration = Arc::new(OrchestrationRepository::new(db_pool.clone()));

    // * * * ISOLATED INFERENCE WORKER (Polars Feature Extraction + LightGBM Evaluation) * * *
    let telemetry_forensic = Arc::clone(&telemetry);
    let orchestration_forensic = Arc::clone(&orchestration);
    let ip_cache_forensic = Arc::clone(&ip_cache);
    let detector_opt = threat_detector.clone();

    tokio::spawn(async move {
        if let Some(detector) = detector_opt {
            println!("|- WORKER FORENSE -| Motor LightGBM + Polars Feature Engineering activo en hilo aislado.");

            // Búfer de micro-ventana deslizante por flujo (source_ip) para cálculo columnar en Polars
            let mut flow_buffers: HashMap<IpAddr, VecDeque<NetworkEvent>> = HashMap::with_capacity(256);
            let mut eval_counter: u64 = 0;

            while let Some(event) = forensic_rx.recv().await {
                eval_counter += 1;

                // 1. Acumular evento en la ventana deslizante del flujo (máx 15 paquetes)
                let buffer = flow_buffers.entry(event.source_ip).or_insert_with(|| VecDeque::with_capacity(16));
                if buffer.len() >= 15 {
                    buffer.pop_front();
                }
                buffer.push_back(event);

                // Convertir ventana a slice continuo
                let window: Vec<NetworkEvent> = buffer.iter().copied().collect();

                // 2. Extraer 23 características columnares estructuradas con Polars
                let (features, polars_time_us) = match PolarsFeatureEngine::extract_features_columnar(&window) {
                    Ok(res) => res,
                    Err(_) => continue,
                };

                let src_str = event.source_ip.to_string();
                let dst_str = event.destination_ip.to_string();

                // 3. Inferencia de anomalía en LightGBM
                let evaluation = detector.evaluate_features(&features, &src_str, polars_time_us);

                // Log periódico de rendimiento del motor de IA
                if eval_counter % 200 == 1 {
                    println!(
                        "|- ML -| Evaluado: {} -> {} | Prob: {:.4} | Polars: {:.1} µs | LightGBM: {:.1} µs | Total: {:.1} µs",
                        src_str, dst_str, evaluation.probability, evaluation.feature_time_us, evaluation.inference_time_us, evaluation.total_time_us
                    );
                }

                // 4. Si clasifica como ataque, ejecutar pipeline forense completo
                if evaluation.is_attack {
                    let node_id = ip_cache_forensic
                        .get(&event.source_ip)
                        .or_else(|| ip_cache_forensic.get(&event.destination_ip))
                        .copied()
                        .unwrap_or(6);

                    println!(
                        "|- ALERTA -| [!] AMENAZA DETECTADA: [{}] (Prob: {:.2}%) en Nodo ID [{}] | Flujo: {} -> {}",
                        evaluation.threat_name, evaluation.probability * 100.0, node_id, src_str, dst_str
                    );
                    println!(
                        "|- ALERTA -| Severidad: [{}] | Impacto: [{}] | Resolución: [{}] | Latencia: {:.1} µs",
                        evaluation.severity, evaluation.impact, evaluation.resolution, evaluation.total_time_us
                    );
                    println!("|- ALERTA -| Detalles: {}", evaluation.technical_details);

                    let telemetry_db = Arc::clone(&telemetry_forensic);
                    let orch_db = Arc::clone(&orchestration_forensic);
                    let proto_str = event.protocol.as_str().to_string();
                    let flags_str = event.flags_to_string();
                    let psize = event.packet_size as i32;
                    let feature_val = features.to_json();
                    let threat_id = evaluation.threat_id;
                    let score = evaluation.probability as f64;

                    // Desacoplar I/O de PostgreSQL en tarea secundaria para garantizar Cero Latencia
                    tokio::spawn(async move {
                        // A. Inserción en network_logs
                        let log_id = match telemetry_db
                            .insert_log(
                                Some(node_id),
                                &src_str,
                                &dst_str,
                                &proto_str,
                                psize,
                                flags_str.as_deref(),
                            )
                            .await
                        {
                            Ok(id) => id,
                            Err(e) => {
                                eprintln!("|- DB ERROR -| Falló el registro forense en network_logs: {}", e);
                                return;
                            }
                        };

                        // B. Inserción del vector columnar en feature_store
                        let feature_id = match telemetry_db.insert_features(log_id, feature_val).await {
                            Ok(fid) => fid,
                            Err(e) => {
                                eprintln!("|- DB ERROR -| Falló la inserción en feature_store: {}", e);
                                return;
                            }
                        };

                        // C. Inserción de la alerta en security_alerts
                        if let Err(e) = orch_db.create_alert(Some(feature_id), Some(threat_id), Some(1), score).await {
                            eprintln!("|- DB ERROR -| Falló la creación de security_alerts: {}", e);
                        }
                    });
                }
            }
        } else {
            println!("|- WORKER FORENSE -| Motor LightGBM en espera (sin modelo cargado).");
        }
    });

    // * * * PERIODIC BENCHMARK TELEMETRY TICKER * * *
    let detector_bench = threat_detector.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(15));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            interval.tick().await;
            if let Some(ref det) = detector_bench {
                let stats = det.get_performance_stats();
                if stats.total_evaluations > 0 {
                    println!(
                        "|- ML STATS -| Evaluaciones: {} | Anomalías: {} | Polars AVG: {:.1} µs | LightGBM AVG: {:.1} µs | Latencia Total AVG: {:.1} µs | Mín: {:.1} µs | Máx: {:.1} µs",
                        stats.total_evaluations, stats.total_anomalies, stats.avg_feature_time_us, stats.avg_inference_time_us, stats.avg_total_time_us, stats.min_inference_time_us, stats.max_inference_time_us
                    );
                }
            }
        }
    });

    // * * * MAIN CONSUMER (Statistical Route with tokio::select! and std::mem::take) * * *
    let telemetry_metrics = Arc::clone(&telemetry);
    let ip_cache_main = Arc::clone(&ip_cache);
    let stream_consumer = stream_handler.clone();

    tokio::spawn(async move {
        println!("|- WORKER ESTADÍSTICO -| Pipeline de agregación atómica en RAM iniciado.");

        let mut accumulator: HashMap<i32, TrafficStats> = HashMap::with_capacity(128);
        let mut window_start = Utc::now();

        let mut flush_interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
        flush_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        let mut packet_counter: u64 = 0;

        loop {
            tokio::select! {
                Some(event) = rx_events.recv() => {
                    packet_counter += 1;

                    // Resolve O(1) in RAM; fallback to ID 6 if unmapped
                    let node_id = ip_cache_main
                        .get(&event.source_ip)
                        .or_else(|| ip_cache_main.get(&event.destination_ip))
                        .copied()
                        .unwrap_or(6);

                    let entry = accumulator.entry(node_id).or_default();
                    entry.packets += 1;
                    entry.bytes += event.packet_size as u64;

                    // Enviar evento al canal de inferencia ML
                    let _ = forensic_tx.try_send(event);
                    stream_consumer.broadcast_event(event);

                    if packet_counter % 500 == 1 {
                        println!(
                            "|- PACKET -| Total: {} pkts | Flujo: {}:{} -> {}:{} | Proto: [{}] | Size: {} bytes",
                            packet_counter,
                            event.source_ip,
                            event.source_port,
                            event.destination_ip,
                            event.destination_port,
                            event.protocol.as_str(),
                            event.packet_size
                        );
                    }
                }

                _ = flush_interval.tick() => {
                    let window_end = Utc::now();
                    let batch = std::mem::take(&mut accumulator);
                    let current_start = window_start;
                    window_start = window_end;

                    if !batch.is_empty() {
                        let telemetry_db = Arc::clone(&telemetry_metrics);
                        tokio::spawn(async move {
                            for (node_id, stats) in batch {
                                if let Err(e) = telemetry_db
                                    .record_metric(
                                        Some(node_id),
                                        current_start,
                                        window_end,
                                        stats.packets as i64,
                                        stats.bytes as i64,
                                    )
                                    .await
                                {
                                    eprintln!("|- DB ERROR -| Falló el volcado atómico en network_traffic_metrics: {}", e);
                                }
                            }
                        });
                    }
                }
            }
        }
    });

    // * * * 8. Start Axum Web Server * * *
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    let state = AppState {
        pool: db_pool.clone(),
        stream_handler,
        threat_detector,
    };

    let app = create_router(state);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(&addr).await?;

    println!("|- API REST -| Servidor expuesto y escuchando en: http://{}", addr);
    println!("|- API REST -| Rutas operativas disponibles:");
    println!("       - GET /health");
    println!("       - GET /api/v1/status");
    println!("       - GET /api/v1/catalogs/device-types");
    println!("       - GET /api/v1/inventory/nodes");
    println!("       - GET /api/v1/telemetry/metrics");
    println!("       - GET /api/v1/telemetry/logs");
    println!("       - GET /api/v1/alerts");
    println!("       - GET /api/v1/ml/stats");
    println!("       - GET /api/v1/ml/model-info");

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
        ("AttackerKali", "192.168.1.50", get_tid("ATTACKER_KALI"), get_pid("SSH")),
        ("VictimeAlpine1", "192.168.1.10", get_tid("WORKSTATION_VICTIM"), get_pid("SSH")),
        ("VictimeAlpine2", "192.168.1.20", get_tid("WORKSTATION_VICTIM"), get_pid("SSH")),
        ("pfSense_Gateway", "192.168.1.1", get_tid("FIREWALL_PERIMETER"), get_pid("SSH")),
        ("Router_Core_L3", "192.168.1.254", get_tid("ROUTER_CORE_L3"), get_pid("SSH")),
    ];

    for (hostname, ip, type_id, proto_id) in nodes {
        let _ = inventory.create_node(hostname, ip, type_id, proto_id).await;
    }
}

async fn seed_initial_catalogs(repo: &CatalogsRepository) {
    println!("|- INIT -| Verificando integridad y sembrado de catálogos base...");

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

    println!("|- INIT -| Catálogos base sincronizados correctamente.");
}
