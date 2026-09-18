// * * * MOTOR PRINCIPAL Y ORQUESTADOR CENTRAL DE TIGOSOC BACKEND * * *

mod domain;
mod infrastructure;
mod presentation;

use std::collections::{HashMap, VecDeque};
use std::env;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use chrono::Utc;
use tokio::net::TcpListener;
use tokio::time::Instant;

use domain::{
    feature_engine::PolarsFeatureEngine,
    ml_trainer::LightGBMTrainer,
    models::{NetworkEvent, TCP_FLAG_SYN},
    threat_detector::ThreatDetector,
};
use infrastructure::{
    db_connection,
    fcm_client::{FcmConfig, FcmNotifier},
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

// * * * ESTRUCTURA DE RASTREO Y MUESTREO ADAPTATIVO POR FLUJO (HIGH-THROUGHPUT) * * *
struct FlowTracker {
    window: VecDeque<NetworkEvent>,
    packet_count: u64,
    last_eval_instant: Instant,
    last_packet_instant: Instant,
    last_alert_instant: Option<Instant>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();

    // * * * 0. MODOS DE EJECUCIÓN CLI (SERVIDOR VS ENTRENAMIENTO) * * *
    let args: Vec<String> = env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("server");

    match mode {
        "train" => {
            let data_arg = args.get(2).map(|s| s.as_str());
            println!("|- INSTRUCCION -| Modo de entrenamiento seleccionado (K-Fold + Polars + LightGBM CPU).");
            LightGBMTrainer::run_training(false, data_arg)?;
            return Ok(());
        }
        "train-gpu" => {
            let data_arg = args.get(2).map(|s| s.as_str());
            println!("|- INSTRUCCION -| Modo de entrenamiento GPU seleccionado.");
            LightGBMTrainer::run_training(true, data_arg)?;
            return Ok(());
        }
        "server" => {}
        other => {
            println!("|- INSTRUCCION -| [WARN] Argumento desconocido '{}'. Modos válidos: server | train | train-gpu", other);
        }
    }

    println!("|- INSTRUCCION -| Iniciando Motor Core de TigoSOC Backend (Rust + Polars + LightGBM)...");

    // * * * 1. INICIALIZACIÓN DE CONEXIÓN A BASE DE DATOS POSTGRESQL * * *
    let db_pool = match db_connection::init_pool().await {
        Ok(pool) => {
            println!("|- INSTRUCCION -| Conexión establecida con éxito en PostgreSQL.");
            pool
        }
        Err(e) => {
            eprintln!("|- INSTRUCCION -| [FATAL] Falló la conexión a PostgreSQL: {}", e);
            return Err(e.into());
        }
    };

    // * * * 2. VERIFICACIÓN Y SEMBRADO DE CATÁLOGOS E INVENTARIO TOPOLÓGICO * * *
    let catalogs = CatalogsRepository::new(db_pool.clone());
    seed_initial_catalogs(&catalogs).await;

    let inventory = InventoryRepository::new(db_pool.clone());
    seed_initial_inventory(&inventory, &catalogs).await;

    // * * * 3. CARGA DE CACHÉ DE TOPOLOGÍA EN MEMORIA RAM (O(1) RESOLUTION) * * *
    let ip_cache = Arc::new(load_ip_cache(&inventory).await);
    println!("|- INSTRUCCION -| Topología cargada en memoria RAM: {} nodos indexados O(1).", ip_cache.len());

    // * * * 4. CARGA DEL MODELO LIGHTGBM Y MOTOR DE DETECCIÓN * * *
    let model_path = env::var("ML_MODEL_PATH").unwrap_or_else(|_| "models/mejor_modelo_kfold.txt".to_string());
    let ml_threshold = env::var("ML_THRESHOLD")
        .unwrap_or_else(|_| "0.50".to_string())
        .parse::<f32>()
        .unwrap_or(0.50);

    let threat_detector = match ThreatDetector::new_from_file(&model_path, ml_threshold) {
        Ok(td) => {
            println!("|- INSTRUCCION -| [ML] ThreatDetector inicializado con umbral de detección: [{:.2}]", ml_threshold);
            Some(td)
        }
        Err(e) => {
            eprintln!("|- INSTRUCCION -| [WARN] No se pudo cargar el modelo LightGBM: {}. El sistema continuará sin ML activo.", e);
            None
        }
    };

    // * * * 5. CANAL DE TRANSMISIÓN DE EVENTOS EN TIEMPO REAL * * *
    let stream_handler = TelemetryStreamHandler::new(1024);

    // * * * 6. INICIAR CAPTURA ASÍNCRONA DE RED (PATRÓN PRODUCTOR-CONSUMIDOR) * * *
    let capture_iface = env::var("CAPTURE_INTERFACE").unwrap_or_else(|_| "any".to_string());
    let adapter = NetworkAdapter::new(&capture_iface);
    let mut rx_events = adapter.start_capture(65536);
    println!("|- INSTRUCCION -| [SNIFFER] Escuchando interfaz de red activa: [{}]", capture_iface);

    // * * * 7. CANAL DESACOPLADO PARA INFERENCIA FORENSE AISLADA * * *
    let (forensic_tx, mut forensic_rx) = tokio::sync::mpsc::channel::<NetworkEvent>(65536);

    let telemetry = Arc::new(TelemetryRepository::new(db_pool.clone()));
    let orchestration = Arc::new(OrchestrationRepository::new(db_pool.clone()));

    // * * * CLIENTE DE NOTIFICACIONES PUSH ZERO-DATA (FCM) * * *
    let fcm_config = FcmConfig::from_env();
    let fcm_notifier = Arc::new(FcmNotifier::new(fcm_config));

    // * * * WORKER FORENSE AISLADO (FEATURE ENGINEERING CON POLARS + LIGHTGBM) * * *
    let telemetry_forensic = Arc::clone(&telemetry);
    let orchestration_forensic = Arc::clone(&orchestration);
    let ip_cache_forensic = Arc::clone(&ip_cache);
    let fcm_forensic = Arc::clone(&fcm_notifier);
    let detector_opt = threat_detector.clone();

    tokio::spawn(async move {
        if let Some(detector) = detector_opt {
            println!("|- INSTRUCCION -| [WORKER FORENSE] Motor LightGBM + Polars Feature Engineering activo en hilo aislado.");

            // * * * BÚFERES DE VENTANAS DESLIZANTES POR FLUJO (SOURCE IP) CON MUESTREO ADAPTATIVO * * *
            let mut flow_trackers: HashMap<IpAddr, FlowTracker> = HashMap::with_capacity(512);
            let mut eval_counter: u64 = 0;

            while let Some(event) = forensic_rx.recv().await {
                let tracker = flow_trackers.entry(event.source_ip).or_insert_with(|| FlowTracker {
                    window: VecDeque::with_capacity(16),
                    packet_count: 0,
                    last_eval_instant: Instant::now(),
                    last_packet_instant: Instant::now(),
                    last_alert_instant: None,
                });

                // * * * EXPIRACIÓN DE FLUJO CARRIER-GRADE (TIGO SOC) * * *
                // Si la ráfaga anterior terminó hace más de 1.5s, la micro-ventana de 15 paquetes
                // está caduca y debe limpiarse para no contaminar sesiones o protocolos nuevos (ej. ICMP ping).
                if tracker.last_packet_instant.elapsed() > Duration::from_millis(1500) {
                    tracker.window.clear();
                    tracker.packet_count = 0;
                }
                tracker.last_packet_instant = Instant::now();

                tracker.packet_count += 1;
                if tracker.window.len() >= 15 {
                    tracker.window.pop_front();
                }
                tracker.window.push_back(event);

                // * * * POLÍTICA DE CADENCIA DE INFERENCIA PARA ALTO RENDIMIENTO (+40,000 PPS) * * *
                // 1. Requiere al menos 5 paquetes para validez estadística
                if tracker.packet_count < 5 {
                    continue;
                }

                // 2. Evaluar de inmediato al alcanzar 5 paquetes (clasificación inicial rápida)
                // 3. O evaluar cada 15 paquetes acumulados (renovación de ventana)
                // 4. O evaluar si han transcurrido más de 100ms desde la última inferencia en este flujo
                // 5. O evaluar si se detecta ráfaga de flags SYN o tamaño de paquete inusual
                let is_cadence_tick = tracker.packet_count == 5
                    || tracker.packet_count % 15 == 0
                    || tracker.last_eval_instant.elapsed() >= Duration::from_millis(100)
                    || ((event.flags & TCP_FLAG_SYN) != 0 && tracker.packet_count % 5 == 0);

                if !is_cadence_tick {
                    continue;
                }

                tracker.last_eval_instant = Instant::now();
                eval_counter += 1;

                // * * * CONVERTIR MICRO-VENTANA A SLICE CONTINUO EN RAM * * *
                let window: Vec<NetworkEvent> = tracker.window.iter().copied().collect();

                // * * * EXTRAER 23 CARACTERÍSTICAS COLUMNARES CON POLARS * * *
                let (features, polars_time_us) = match PolarsFeatureEngine::extract_features_columnar(&window) {
                    Ok(res) => res,
                    Err(_) => continue,
                };

                let src_str = event.source_ip.to_string();
                let dst_str = event.destination_ip.to_string();

                // * * * INFERENCIA ULTRA-RÁPIDA CON LIGHTGBM * * *
                let evaluation = detector.evaluate_features(&features, &src_str, polars_time_us);

                // * * * LOG PERIÓDICO DE MONITOREO DEL MOTOR ML * * *
                if eval_counter % 200 == 1 {
                    println!(
                        "|- INSTRUCCION -| [ML] Evaluado: {} -> {} | Prob: {:.4} | Polars: {:.0} ns | LightGBM: {:.0} ns | Total: {:.0} ns",
                        src_str, dst_str, evaluation.probability, evaluation.feature_time_us * 1000.0, evaluation.inference_time_us * 1000.0, evaluation.total_time_us * 1000.0
                    );
                }

                // * * * EJECUTAR PIPELINE FORENSE COMPLETO ANTE AMENAZAS DETECTADAS * * *
                if evaluation.is_attack {
                    // Throttling de alertas por flujo (máximo 1 alerta/despacho cada 2s para proteger BD y FCM)
                    let should_dispatch = match tracker.last_alert_instant {
                        Some(t) => t.elapsed() >= Duration::from_secs(2),
                        None => true,
                    };

                    if !should_dispatch {
                        continue;
                    }
                    tracker.last_alert_instant = Some(Instant::now());

                    let node_id = ip_cache_forensic
                        .get(&event.source_ip)
                        .or_else(|| ip_cache_forensic.get(&event.destination_ip))
                        .copied()
                        .unwrap_or(6);

                    println!(
                        "|- INSTRUCCION -| [ALERTA] AMENAZA DETECTADA: [{}] (Prob: {:.2}%) en Nodo ID [{}] | Flujo: {} -> {}",
                        evaluation.threat_name, evaluation.probability * 100.0, node_id, src_str, dst_str
                    );
                    println!(
                        "|- INSTRUCCION -| [ALERTA] Severidad: [{}] | Impacto: [{}] | Resolución: [{}] | Latencia: {:.0} ns",
                        evaluation.severity, evaluation.impact, evaluation.resolution, evaluation.total_time_us * 1000.0
                    );
                    println!("|- INSTRUCCION -| [ALERTA] Detalles: {}", evaluation.technical_details);

                    let telemetry_db = Arc::clone(&telemetry_forensic);
                    let orch_db = Arc::clone(&orchestration_forensic);
                    let fcm_push = Arc::clone(&fcm_forensic);
                    let proto_str = event.protocol.as_str().to_string();
                    let flags_str = event.flags_to_string();
                    let psize = event.packet_size as i32;
                    let feature_val = features.to_json();
                    let threat_id = evaluation.threat_id;
                    let threat_name = evaluation.threat_name.clone();
                    let threat_sev = evaluation.severity.clone();
                    let score = evaluation.probability as f64;

                    // * * * DESACOPLAR I/O DE POSTGRESQL Y PUSH NOTIFICATION EN SEGUNDO PLANO * * *
                    tokio::spawn(async move {
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
                                eprintln!("|- INSTRUCCION -| [DB ERROR] Falló el registro forense en network_logs: {}", e);
                                return;
                            }
                        };

                        let feature_id = match telemetry_db.insert_features(log_id, feature_val).await {
                            Ok(fid) => fid,
                            Err(e) => {
                                eprintln!("|- INSTRUCCION -| [DB ERROR] Falló la inserción en feature_store: {}", e);
                                return;
                            }
                        };

                        let alert_id = match orch_db.create_alert(Some(feature_id), Some(threat_id), Some(1), score).await {
                            Ok(aid) => aid,
                            Err(e) => {
                                eprintln!("|- INSTRUCCION -| [DB ERROR] Falló la creación de security_alerts: {}", e);
                                return;
                            }
                        };

                        // * * * ESTRATEGIA ZERO-DATA PUSH: NOTIFICACIÓN OPACA A FIREBASE (TOPIC: /topics/soc_alerts) * * *
                        if let Err(e) = fcm_push.send_opaque_alert(alert_id, &threat_sev, &threat_name).await {
                            eprintln!("|- INSTRUCCION -| [FCM ERROR] Falló el envío del webhook Push: {}", e);
                        }
                    });
                }
            }
        } else {
            println!("|- INSTRUCCION -| [WORKER FORENSE] Motor LightGBM en espera (sin modelo cargado).");
        }
    });

    // * * * TELEMETRÍA PERIÓDICA DE RENDIMIENTO DEL MOTOR ML * * *
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
                        "|- INSTRUCCION -| [ML STATS] Evaluaciones: {} | Anomalías: {} | Polars AVG: {:.0} ns | LightGBM AVG: {:.0} ns | Latencia Total AVG: {:.0} ns | Mín: {:.0} ns | Máx: {:.0} ns",
                        stats.total_evaluations, stats.total_anomalies, stats.avg_feature_time_us * 1000.0, stats.avg_inference_time_us * 1000.0, stats.avg_total_time_us * 1000.0, stats.min_inference_time_us * 1000.0, stats.max_inference_time_us * 1000.0
                    );
                }
            }
        }
    });

    // * * * CONSUMIDOR PRINCIPAL (MUNDO ESTADÍSTICO CON TOKIO::SELECT! Y VOLCADO ATÓMICO) * * *
    let telemetry_metrics = Arc::clone(&telemetry);
    let ip_cache_main = Arc::clone(&ip_cache);
    let stream_consumer = stream_handler.clone();

    tokio::spawn(async move {
        println!("|- INSTRUCCION -| [WORKER ESTADÍSTICO] Pipeline de agregación atómica en RAM iniciado.");

        let mut accumulator: HashMap<i32, TrafficStats> = HashMap::with_capacity(128);
        let mut window_start = Utc::now();

        let mut flush_interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
        flush_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        let mut packet_counter: u64 = 0;

        loop {
            tokio::select! {
                Some(event) = rx_events.recv() => {
                    packet_counter += 1;

                    // * * * RESOLUCIÓN O(1) EN MEMORIA RAM CON NODO 6 POR DEFECTO * * *
                    let node_id = ip_cache_main
                        .get(&event.source_ip)
                        .or_else(|| ip_cache_main.get(&event.destination_ip))
                        .copied()
                        .unwrap_or(6);

                    let entry = accumulator.entry(node_id).or_default();
                    entry.packets += 1;
                    entry.bytes += event.packet_size as u64;

                    // * * * ENVIAR EVENTO AL CANAL DE INFERENCIA ML * * *
                    let _ = forensic_tx.try_send(event);
                    stream_consumer.broadcast_event(event);

                    if packet_counter % 500 == 1 {
                        println!(
                            "|- INSTRUCCION -| [PACKET] Total: {} pkts | Flujo: {}:{} -> {}:{} | Proto: [{}] | Size: {} bytes",
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
                                    eprintln!("|- INSTRUCCION -| [DB ERROR] Falló el volcado atómico en network_traffic_metrics: {}", e);
                                }
                            }
                        });
                    }
                }
            }
        }
    });

    // * * * 8. INICIAR SERVIDOR WEB AXUM * * *
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    let state = AppState {
        pool: db_pool.clone(),
        stream_handler,
        threat_detector,
        fcm_notifier: Arc::clone(&fcm_notifier),
    };

    let app = create_router(state);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(&addr).await?;

    println!("|- INSTRUCCION -| [API REST] Servidor expuesto y escuchando en: http://{}", addr);
    println!("|- INSTRUCCION -| [API REST] Rutas operativas disponibles:");
    println!("|- INSTRUCCION -|    - GET /health");
    println!("|- INSTRUCCION -|    - GET /api/v1/status");
    println!("|- INSTRUCCION -|    - GET /api/v1/catalogs/device-types");
    println!("|- INSTRUCCION -|    - GET /api/v1/inventory/nodes");
    println!("|- INSTRUCCION -|    - GET /api/v1/telemetry/metrics");
    println!("|- INSTRUCCION -|    - GET /api/v1/telemetry/logs");
    println!("|- INSTRUCCION -|    - GET /api/v1/alerts");
    println!("|- INSTRUCCION -|    - GET /api/v1/alerts/:id");
    println!("|- INSTRUCCION -|    - POST /api/v1/alerts/simulate-push");
    println!("|- INSTRUCCION -|    - GET /api/v1/ml/stats");
    println!("|- INSTRUCCION -|    - GET /api/v1/ml/model-info");
    println!("|- INSTRUCCION -|    - GET /pwa (Receptor Web Push PWA)");

    axum::serve(listener, app).await?;

    Ok(())
}

// * * * CARGA DE TABLA DE TOPOLOGÍA EN MEMORIA RAM * * *
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

// * * * SEMBRADO INICIAL DEL INVENTARIO DE NODOS DE RED * * *
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

// * * * SEMBRADO INICIAL DE CATÁLOGOS BASE DEL SISTEMA * * *
async fn seed_initial_catalogs(repo: &CatalogsRepository) {
    println!("|- INSTRUCCION -| [INIT] Verificando integridad y sembrado de catálogos base...");

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

    println!("|- INSTRUCCION -| [INIT] Catálogos base sincronizados correctamente.");
}
