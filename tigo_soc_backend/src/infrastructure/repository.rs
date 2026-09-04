use sqlx::{PgPool, Error};
use serde_json::Value;
use crate::domain::models::{SecurityAlert, ExecutionQueue, RollbackSnapshot, SystemAuditLog};

#[derive(Clone)]
pub struct OrchestrationRepository {
    pub pool: PgPool,
}

impl OrchestrationRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    // * * * SECURITY ALERTS * * *
    pub async fn create_alert(&self, log_id: i64, threat_id: i32, score: f64, status_id: i32) -> Result<i64, Error> {
        let res = sqlx::query!("INSERT INTO security_alerts (network_log_id, threat_type_id, anomaly_score, status_id) VALUES ($1, $2, $3, $4) RETURNING id", log_id, threat_id, score, status_id)
            .fetch_one(&self.pool).await?;
        Ok(res.id)
    }
    pub async fn update_alert_status(&self, alert_id: i64, new_status_id: i32) -> Result<(), Error> {
        sqlx::query!("UPDATE security_alerts SET status_id = $1, resolved_at = CURRENT_TIMESTAMP WHERE id = $2", new_status_id, alert_id)
            .execute(&self.pool).await?;
        Ok(())
    }

    // * * * EXECUTION QUEUE * * *
    pub async fn queue_task(&self, alert_id: i64, action: &str, target: Option<&str>) -> Result<i64, Error> {
        let res = sqlx::query!("INSERT INTO execution_queue (alert_id, action_type, target_ip) VALUES ($1, $2, $3) RETURNING id", alert_id, action, target)
            .fetch_one(&self.pool).await?;
        Ok(res.id)
    }

    // * * * ROLLBACK * * *
    pub async fn create_snapshot(&self, task_id: i64, state: Value) -> Result<i64, Error> {
        let res = sqlx::query!("INSERT INTO rollback_snapshots (task_id, previous_state) VALUES ($1, $2) RETURNING id", task_id, state)
            .fetch_one(&self.pool).await?;
        Ok(res.id)
    }

    // * * * SYSTEM AUDIT LOGS * * *
    pub async fn log_audit(&self, user_id: i32, action: &str, details: Option<&str>) -> Result<(), Error> {
        sqlx::query!("INSERT INTO system_audit_logs (user_id, action, details) VALUES ($1, $2, $3)", user_id, action, details)
            .execute(&self.pool).await?;
        Ok(())
    }
}