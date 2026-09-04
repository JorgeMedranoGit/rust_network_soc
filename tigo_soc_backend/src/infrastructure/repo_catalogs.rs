#![allow(dead_code)]
use sqlx::{Error, PgPool};
use crate::domain::models::{AlertStatus, DeviceType, MgmtProtocol, Role, TaskStatus, ThreatType};

#[derive(Clone)]
pub struct CatalogsRepository {
    pub pool: PgPool,
}

impl CatalogsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // --- DEVICE TYPES ---
    pub async fn create_device_type(&self, type_name: &str) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO device_types (type_name) VALUES ($1) ON CONFLICT (type_name) DO NOTHING"
        )
        .bind(type_name)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_device_types(&self) -> Result<Vec<DeviceType>, Error> {
        sqlx::query_as::<_, DeviceType>("SELECT type_id, type_name FROM device_types ORDER BY type_id ASC")
            .fetch_all(&self.pool)
            .await
    }

    // --- MANAGEMENT PROTOCOLS ---
    pub async fn create_mgmt_protocol(&self, protocol_name: &str) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO mgmt_protocols (protocol_name) VALUES ($1) ON CONFLICT (protocol_name) DO NOTHING"
        )
        .bind(protocol_name)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_mgmt_protocols(&self) -> Result<Vec<MgmtProtocol>, Error> {
        sqlx::query_as::<_, MgmtProtocol>("SELECT protocol_id, protocol_name FROM mgmt_protocols ORDER BY protocol_id ASC")
            .fetch_all(&self.pool)
            .await
    }

    // --- THREAT TYPES ---
    pub async fn create_threat_type(&self, threat_name: &str, severity_level: i32) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO threat_types (threat_name, severity_level) VALUES ($1, $2) ON CONFLICT (threat_name) DO NOTHING"
        )
        .bind(threat_name)
        .bind(severity_level)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_threat_types(&self) -> Result<Vec<ThreatType>, Error> {
        sqlx::query_as::<_, ThreatType>("SELECT threat_id, threat_name, severity_level FROM threat_types ORDER BY threat_id ASC")
            .fetch_all(&self.pool)
            .await
    }

    // --- ALERT STATUSES ---
    pub async fn create_alert_status(&self, status_name: &str) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO alert_statuses (status_name) VALUES ($1) ON CONFLICT (status_name) DO NOTHING"
        )
        .bind(status_name)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_alert_statuses(&self) -> Result<Vec<AlertStatus>, Error> {
        sqlx::query_as::<_, AlertStatus>("SELECT status_id, status_name FROM alert_statuses ORDER BY status_id ASC")
            .fetch_all(&self.pool)
            .await
    }

    // --- TASK STATUSES ---
    pub async fn create_task_status(&self, status_name: &str) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO task_statuses (status_name) VALUES ($1) ON CONFLICT (status_name) DO NOTHING"
        )
        .bind(status_name)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_task_statuses(&self) -> Result<Vec<TaskStatus>, Error> {
        sqlx::query_as::<_, TaskStatus>("SELECT status_id, status_name FROM task_statuses ORDER BY status_id ASC")
            .fetch_all(&self.pool)
            .await
    }

    // --- ROLES ---
    pub async fn create_role(&self, role_name: &str, description: Option<&str>) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO roles (role_name, description) VALUES ($1, $2) ON CONFLICT (role_name) DO NOTHING"
        )
        .bind(role_name)
        .bind(description)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_roles(&self) -> Result<Vec<Role>, Error> {
        sqlx::query_as::<_, Role>("SELECT role_id, role_name, description FROM roles ORDER BY role_id ASC")
            .fetch_all(&self.pool)
            .await
    }
}
