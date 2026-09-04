use sqlx::{PgPool, Error};
use serde_json::Value;
use crate::domain::models::{NetworkLog, FeatureStore};

#[derive(Clone)]
pub struct TelemetryRepository {
    pub pool: PgPool,
}

impl TelemetryRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    // * * * NETWORK LOGS * * *
    pub async fn insert_log(&self, src_ip: &str, dst_ip: &str, proto: &str, size: i32, payload: Option<Value>) -> Result<i64, Error> {
        let res = sqlx::query!("INSERT INTO network_logs (source_ip, dest_ip, protocol, packet_size, payload_data) VALUES ($1, $2, $3, $4, $5) RETURNING id", src_ip, dst_ip, proto, size, payload)
            .fetch_one(&self.pool).await?;
        Ok(res.id)
    }
    pub async fn get_recent_logs(&self, limit: i64) -> Result<Vec<NetworkLog>, Error> {
        Ok(sqlx::query_as!(NetworkLog, "SELECT * FROM network_logs ORDER BY timestamp DESC LIMIT $1", limit).fetch_all(&self.pool).await?)
    }

    // * * * FEATURE STORE * * *
    pub async fn insert_features(&self, log_id: i64, features: Value) -> Result<i64, Error> {
        let res = sqlx::query!("INSERT INTO feature_store (network_log_id, extracted_features) VALUES ($1, $2) RETURNING id", log_id, features)
            .fetch_one(&self.pool).await?;
        Ok(res.id)
    }
}