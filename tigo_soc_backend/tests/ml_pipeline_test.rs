// * * * PRUEBAS DE INTEGRACIÓN Y BENCHMARKS DEL MOTOR ML Y POLARS * * *

use chrono::Utc;
use std::net::IpAddr;
use std::str::FromStr;

use tigo_soc_backend::domain::{
    feature_engine::PolarsFeatureEngine,
    models::{L4Protocol, NetworkEvent, TCP_FLAG_ACK, TCP_FLAG_SYN},
    threat_detector::ThreatDetector,
};

// * * * PRUEBA 1: EXTRACCIÓN COLUMNAR DE CARACTERÍSTICAS CON POLARS * * *
#[test]
fn test_polars_feature_engineering_columnar() {
    let mut events = Vec::new();
    let src = IpAddr::from_str("192.168.1.50").unwrap();
    let dst = IpAddr::from_str("192.168.1.10").unwrap();

    // * * * CREAR 10 PAQUETES SIMULANDO RÁFAGA DE TRÁFICO * * *
    for i in 0..10 {
        events.push(NetworkEvent {
            source_ip: src,
            destination_ip: dst,
            source_port: 45000 + i,
            destination_port: 80,
            protocol: L4Protocol::TCP,
            packet_size: 120 + (i * 10),
            header_length: 20,
            ttl: 64,
            flags: TCP_FLAG_SYN | TCP_FLAG_ACK,
            anomaly_score: None,
            timestamp: Utc::now(),
        });
    }

    let (features, time_us) = PolarsFeatureEngine::extract_features_columnar(&events).unwrap();

    println!("|- INSTRUCCION -| [TEST] Tiempo Polars Feature Engineering: {:.2} µs", time_us);
    assert_eq!(features.number, 10.0);
    assert!(features.rate > 0.0);
    assert_eq!(features.http, 1.0);
    assert_eq!(features.tcp, 1.0);
    assert_eq!(features.min_size, 120.0);
    assert_eq!(features.max_size, 210.0);
    assert_eq!(features.ack_count, 10.0);
    assert_eq!(features.syn_count, 10.0);
    assert_eq!(features.to_vector().len(), 23);
}

// * * * PRUEBA 2: LATENCIA DE INFERENCIA EN TIEMPO REAL CON LIGHTGBM * * *
#[test]
fn test_lightgbm_inference_latency() {
    let detector = ThreatDetector::new_from_file("models/mejor_modelo_kfold.txt", 0.50)
        .expect("El modelo mejor_modelo_kfold.txt debe cargarse correctamente");

    let src = IpAddr::from_str("192.168.1.50").unwrap();
    let dst = IpAddr::from_str("192.168.1.10").unwrap();

    let mut events = Vec::new();
    for _ in 0..15 {
        events.push(NetworkEvent {
            source_ip: src,
            destination_ip: dst,
            source_port: 4444,
            destination_port: 80,
            protocol: L4Protocol::TCP,
            packet_size: 1400,
            header_length: 20,
            ttl: 64,
            flags: TCP_FLAG_SYN,
            anomaly_score: None,
            timestamp: Utc::now(),
        });
    }

    let (features, polars_time_us) = PolarsFeatureEngine::extract_features_columnar(&events).unwrap();
    let evaluation = detector.evaluate_features(&features, "192.168.1.50", polars_time_us);

    println!(
        "|- INSTRUCCION -| [TEST] Inferencia completada: Probabilidad = {:.4}, Tiempo Inferencia = {:.2} µs, Tiempo Total = {:.2} µs",
        evaluation.probability, evaluation.inference_time_us, evaluation.total_time_us
    );

    assert!(evaluation.inference_time_us > 0.0);
    assert_eq!(evaluation.features.len(), 23);
}

// * * * PRUEBA 3: PARIDAD ZERO-ALLOCATION (ARRAY EN EL STACK VS VECTOR) * * *
#[test]
fn test_zero_allocation_array_and_vector_parity() {
    let src = IpAddr::from_str("192.168.1.50").unwrap();
    let dst = IpAddr::from_str("192.168.1.10").unwrap();

    let events = vec![NetworkEvent {
        source_ip: src,
        destination_ip: dst,
        source_port: 80,
        destination_port: 80,
        protocol: L4Protocol::TCP,
        packet_size: 500,
        header_length: 20,
        ttl: 64,
        flags: TCP_FLAG_SYN,
        anomaly_score: None,
        timestamp: Utc::now(),
    }];

    let (features, _) = PolarsFeatureEngine::extract_features_columnar(&events).unwrap();
    let array = features.to_array();
    let vector = features.to_vector();

    assert_eq!(array.len(), 23);
    assert_eq!(vector.len(), 23);
    for i in 0..23 {
        assert_eq!(array[i], vector[i]);
    }
}

// * * * PRUEBA 4: ESTRATEGIA ZERO-DATA PUSH FCM (PAYLOAD OPACO SIN PII NI IPS) * * *
#[tokio::test]
async fn test_zero_data_push_fcm_notification() {
    use tigo_soc_backend::infrastructure::fcm_client::{FcmConfig, FcmNotifier};

    let config = FcmConfig {
        server_key: None,
        project_id: None,
        bearer_token: None,
        service_account_path: None,
        topic: "soc_alerts".to_string(),
        enabled: false, // Modo simulación seguro
    };

    let notifier = FcmNotifier::new(config);
    let result = notifier
        .send_opaque_alert(42, "CRITICAL", "DATA_EXFILTRATION")
        .await;

    assert!(result.is_ok(), "El despacho Zero-Data Push debe ejecutarse sin errores en simulación");
}

// * * * PRUEBA 5: AUTENTICACIÓN GOOGLE SERVICE ACCOUNT Y OAUTH2 TOKEN RENEWAL * * *
#[tokio::test]
async fn test_google_service_account_oauth2_token() {
    use tigo_soc_backend::infrastructure::fcm_client::{FcmConfig, FcmNotifier};

    let sa_exists = std::path::Path::new("../service-account.json").exists()
        || std::path::Path::new("service-account.json").exists();
    if !sa_exists {
        return;
    }

    let config = FcmConfig::from_env();
    let notifier = FcmNotifier::new(config);
    let token_result = notifier.get_valid_bearer_token().await;
    assert!(token_result.is_ok(), "Debe poder generar un bearer token OAuth2 válido desde service-account.json: {:?}", token_result.err());
    let token = token_result.unwrap();
    assert!(!token.is_empty());
    assert!(token.starts_with("ya29."));
    println!("|- INSTRUCCION -| [TEST] Token OAuth2 Bearer autogenerado con éxito (Prefijo: {}...)", &token[..15]);
}

