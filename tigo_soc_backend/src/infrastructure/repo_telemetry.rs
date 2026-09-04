#![allow(dead_code)]
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{Error, PgPool, Row};
use crate::domain::models::{FeatureStore, NetworkLog, NetworkTrafficMetric};

#[derive(Clone)]
pub struct TelemetryRepository {
    pub pool: PgPool,
}

impl TelemetryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // =========================================================================
    // 1. TELEMETRÍA AGREGADA (Mundo Estadístico / Masivo)
    // =========================================================================

    pub async fn record_metric(
        &self,
        node_id: Option<i32>,
        window_start: DateTime<Utc>,
        window_end: DateTime<Utc>,
        packets: i64,
        bytes: i64,
    ) -> Result<i64, Error> {
        let row = sqlx::query(
            "INSERT INTO network_traffic_metrics (node_id, window_start, window_end, total_packets, total_bytes)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (node_id, window_start) DO UPDATE SET
                 total_packets = network_traffic_metrics.total_packets + EXCLUDED.total_packets,
                 total_bytes = network_traffic_metrics.total_bytes + EXCLUDED.total_bytes,
                 window_end = EXCLUDED.window_end
             RETURNING metric_id"
        )
        .bind(node_id)
        .bind(window_start)
        .bind(window_end)
        .bind(packets)
        .bind(bytes)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("metric_id"))
    }

    pub async fn get_recent_metrics(&self, limit: i64) -> Result<Vec<NetworkTrafficMetric>, Error> {
        sqlx::query_as::<_, NetworkTrafficMetric>(
            "SELECT metric_id, node_id, window_start, window_end, total_packets, total_bytes, created_at
             FROM network_traffic_metrics
             ORDER BY window_start DESC
             LIMIT $1"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    // =========================================================================
    // 2. LOGS FORENSES (Mundo Forense / Alertas)
    // =========================================================================

    pub async fn insert_log(
        &self,
        node_id: Option<i32>,
        source_ip: &str,
        destination_ip: &str,
        protocol: &str,
        packet_size: i32,
        flags: Option<&str>,
    ) -> Result<i64, Error> {
        let row = sqlx::query(
            "INSERT INTO network_logs (node_id, source_ip, destination_ip, protocol, packet_size, flags)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING log_id"
        )
        .bind(node_id)
        .bind(source_ip)
        .bind(destination_ip)
        .bind(protocol)
        .bind(packet_size)
        .bind(flags)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("log_id"))
    }

    pub async fn get_recent_logs(&self, limit: i64) -> Result<Vec<NetworkLog>, Error> {
        sqlx::query_as::<_, NetworkLog>(
            "SELECT log_id, node_id, source_ip, destination_ip, protocol, packet_size, flags, timestamp
             FROM network_logs
             ORDER BY timestamp DESC
             LIMIT $1"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    // =========================================================================
    // 3. FEATURE STORE (ML / LightGBM)
    // =========================================================================

    pub async fn insert_features(&self, log_id: i64, feature_vector: Value) -> Result<i64, Error> {
        let row = sqlx::query(
            "INSERT INTO feature_store (log_id, feature_vector)
             VALUES ($1, $2)
             ON CONFLICT (log_id) DO UPDATE SET
                 feature_vector = EXCLUDED.feature_vector,
                 processed_at = CURRENT_TIMESTAMP
             RETURNING feature_id"
        )
        .bind(log_id)
        .bind(feature_vector)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("feature_id"))
    }

    pub async fn get_features_by_log(&self, log_id: i64) -> Result<Option<FeatureStore>, Error> {
        sqlx::query_as::<_, FeatureStore>(
            "SELECT feature_id, log_id, feature_vector, processed_at
             FROM feature_store
             WHERE log_id = $1"
        )
        .bind(log_id)
        .fetch_optional(&self.pool)
        .await
    }
}
