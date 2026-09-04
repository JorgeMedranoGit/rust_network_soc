#![allow(dead_code)]
use sqlx::{Error, PgPool, Row};
use crate::domain::models::NetworkNode;

#[derive(Clone)]
pub struct InventoryRepository {
    pub pool: PgPool,
}

impl InventoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_node(
        &self,
        hostname: &str,
        ip_address: &str,
        type_id: Option<i32>,
        protocol_id: Option<i32>,
    ) -> Result<i32, Error> {
        let row = sqlx::query(
            "INSERT INTO network_nodes (hostname, ip_address, type_id, protocol_id)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (hostname) DO UPDATE SET
                 ip_address = EXCLUDED.ip_address,
                 type_id = EXCLUDED.type_id,
                 protocol_id = EXCLUDED.protocol_id
             RETURNING node_id"
        )
        .bind(hostname)
        .bind(ip_address)
        .bind(type_id)
        .bind(protocol_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("node_id"))
    }

    pub async fn get_nodes(&self) -> Result<Vec<NetworkNode>, Error> {
        sqlx::query_as::<_, NetworkNode>(
            "SELECT node_id, hostname, ip_address, type_id, protocol_id FROM network_nodes ORDER BY node_id ASC"
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_node_by_ip(&self, ip_address: &str) -> Result<Option<NetworkNode>, Error> {
        sqlx::query_as::<_, NetworkNode>(
            "SELECT node_id, hostname, ip_address, type_id, protocol_id FROM network_nodes WHERE ip_address = $1"
        )
        .bind(ip_address)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn get_node_by_id(&self, node_id: i32) -> Result<Option<NetworkNode>, Error> {
        sqlx::query_as::<_, NetworkNode>(
            "SELECT node_id, hostname, ip_address, type_id, protocol_id FROM network_nodes WHERE node_id = $1"
        )
        .bind(node_id)
        .fetch_optional(&self.pool)
        .await
    }
}
