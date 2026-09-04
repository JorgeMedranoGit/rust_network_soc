use sqlx::{PgPool, Error};
use crate::domain::models::{DeviceType, AlertStatus, MgmtProtocol, ThreatType, Role};

#[derive(Clone)]
pub struct CatalogsRepository {
    pub pool: PgPool,
}

impl CatalogsRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    // --- DEVICE TYPES ---
    pub async fn create_device_type(&self, name: &str, description: Option<&str>) -> Result<(), Error> {
        sqlx::query!("INSERT INTO device_types (name, description) VALUES ($1, $2) ON CONFLICT (name) DO NOTHING", name, description)
            .execute(&self.pool).await?;
        Ok(())
    }
    pub async fn get_device_types(&self) -> Result<Vec<DeviceType>, Error> {
        Ok(sqlx::query_as!(DeviceType, "SELECT * FROM device_types").fetch_all(&self.pool).await?)
    }

    // --- ROLES ---
    pub async fn create_role(&self, name: &str, description: Option<&str>) -> Result<(), Error> {
        sqlx::query!("INSERT INTO roles (name, description) VALUES ($1, $2) ON CONFLICT (name) DO NOTHING", name, description)
            .execute(&self.pool).await?;
        Ok(())
    }
    pub async fn get_roles(&self) -> Result<Vec<Role>, Error> {
        Ok(sqlx::query_as!(Role, "SELECT * FROM roles").fetch_all(&self.pool).await?)
    }

    // --- ALERT STATUSES ---
    pub async fn create_alert_status(&self, name: &str, description: Option<&str>) -> Result<(), Error> {
        sqlx::query!("INSERT INTO alert_statuses (name, description) VALUES ($1, $2) ON CONFLICT (name) DO NOTHING", name, description)
            .execute(&self.pool).await?;
        Ok(())
    }

    // --- THREAT TYPES ---
    pub async fn create_threat_type(&self, name: &str, severity_level: i32, description: Option<&str>) -> Result<(), Error> {
        sqlx::query!("INSERT INTO threat_types (name, severity_level, description) VALUES ($1, $2, $3) ON CONFLICT (name) DO NOTHING", name, severity_level, description)
            .execute(&self.pool).await?;
        Ok(())
    }
}