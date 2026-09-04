mod domain;
mod infrastructure;
mod presentation;

use std::env;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use domain::threat_detector::ThreatDetector;
use infrastructure::{
    db_connection,
    network_adapter::NetworkAdapter,
    repo_catalogs::CatalogsRepository,
    repo_telemetry::TelemetryRepository,
};
use presentation::{
    api_routes::{create_router, AppState},
    telemetry_stream::TelemetryStreamHandler,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();

    println!("============================================================");
    println!(" [INFO] Iniciando Tigo SOC Backend (Arquitectura Capa 3)");
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

    // 2. Inicializar y Sembrar Catálogos Base (si no existen)
    let catalogs = CatalogsRepository::new(db_pool.clone());
    seed_initial_catalogs(&catalogs).await;

    // 3. Canal de retransmisión de eventos
    let stream_handler = TelemetryStreamHandler::new(1024);

    // 4. Iniciar Captura de Red (Sniffer con Patrón Productor-Consumidor)
    let capture_iface = env::var("CAPTURE_INTERFACE").unwrap_or_else(|_| "any".to_string());
    let adapter = NetworkAdapter::new(&capture_iface);
    let mut rx_events = adapter.start_capture(2048);

    // Consumidor asíncrono de eventos de red
    let pool_consumer = db_pool.clone();
    let stream_consumer = stream_handler.clone();
    tokio::spawn(async move {
        let detector = ThreatDetector::new(0.80);
        let telemetry = TelemetryRepository::new(pool_consumer);

        println!("[CONSUMER] Tarea asíncrona de procesamiento de telemetría iniciada.");
        let mut packet_counter: u64 = 0;

        while let Some(event) = rx_events.recv().await {
            packet_counter += 1;

            // Retransmitir al canal de eventos
            stream_consumer.broadcast_event(event.clone());

            // Log periódico de actividad para telemetría
            if packet_counter % 100 == 1 {
                println!(
                    "[SNIFFER] Paquetes procesados: {} | Último: {} -> {} [{}] ({} bytes)",
                    packet_counter, event.source_ip, event.destination_ip, event.protocol, event.packet_size
                );
            }

            // Detección de anomalías
            if detector.evaluate_event(&event) {
                println!(
                    "[ALERTA] Anomalía sospechosa detectada en tráfico: {} -> {}",
                    event.source_ip, event.destination_ip
                );
                let _ = telemetry
                    .insert_log(
                        None,
                        &event.source_ip,
                        &event.destination_ip,
                        &event.protocol,
                        event.packet_size,
                        event.flags.as_deref(),
                    )
                    .await;
            }
        }
    });

    // 5. Iniciar Servidor Web Axum
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
