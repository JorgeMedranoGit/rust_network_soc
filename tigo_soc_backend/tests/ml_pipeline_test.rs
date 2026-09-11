use chrono::Utc;
use std::net::IpAddr;
use std::str::FromStr;

// Import modules from the main crate if library or directly
use tigo_soc_backend::domain::{
    feature_engine::PolarsFeatureEngine,
    models::{L4Protocol, NetworkEvent, TCP_FLAG_ACK, TCP_FLAG_SYN},
    threat_detector::ThreatDetector,
};

#[test]
fn test_polars_feature_engineering_columnar() {
    let mut events = Vec::new();
    let src = IpAddr::from_str("192.168.1.50").unwrap();
    let dst = IpAddr::from_str("192.168.1.10").unwrap();

    // Crear 10 paquetes simulando una ráfaga
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

    println!("|- TEST -| Tiempo Polars Feature Engineering: {:.2} µs", time_us);
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
        "|- TEST -| Inferencia completada: Probabilidad = {:.4}, Tiempo Inferencia = {:.2} µs, Tiempo Total = {:.2} µs",
        evaluation.probability, evaluation.inference_time_us, evaluation.total_time_us
    );

    assert!(evaluation.inference_time_us > 0.0);
    assert!(evaluation.features.len() == 23);
}
