#![allow(dead_code)]
use serde_json::Value;
use sqlx::{Error, PgPool, Row};
use crate::domain::models::{MitigationAction, SecurityAlert};

#[derive(Clone)]
pub struct OrchestrationRepository {
    pub pool: PgPool,
}

impl OrchestrationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // =========================================================================
    // 1. ALERTAS DE SEGURIDAD
    // =========================================================================

    pub async fn create_alert(
        &self,
        feature_id: Option<i64>,
        threat_id: Option<i32>,
        status_id: Option<i32>,
        anomaly_score: f64,
    ) -> Result<i64, Error> {
        let row = sqlx::query(
            "INSERT INTO security_alerts (feature_id, threat_id, status_id, anomaly_score)
             VALUES ($1, $2, $3, $4)
             RETURNING alert_id"
        )
        .bind(feature_id)
        .bind(threat_id)
        .bind(status_id)
        .bind(anomaly_score)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("alert_id"))
    }

    pub async fn update_alert_status(&self, alert_id: i64, new_status_id: i32) -> Result<(), Error> {
        sqlx::query(
            "UPDATE security_alerts
             SET status_id = $1
             WHERE alert_id = $2"
        )
        .bind(new_status_id)
        .bind(alert_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_recent_alerts(&self, limit: i64) -> Result<Vec<SecurityAlert>, Error> {
        sqlx::query_as::<_, SecurityAlert>(
            "SELECT alert_id, feature_id, threat_id, status_id, anomaly_score, detected_at
             FROM security_alerts
             ORDER BY detected_at DESC
             LIMIT $1"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    // =========================================================================
    // 2. ACCIONES DE MITIGACIÓN
    // =========================================================================

    pub async fn create_mitigation_action(
        &self,
        action_name: &str,
        layer_target: &str,
        description: Option<&str>,
    ) -> Result<i32, Error> {
        let row = sqlx::query(
            "INSERT INTO mitigation_actions (action_name, layer_target, description)
             VALUES ($1, $2, $3)
             ON CONFLICT (action_name) DO UPDATE SET
                 layer_target = EXCLUDED.layer_target,
                 description = EXCLUDED.description
             RETURNING action_id"
        )
        .bind(action_name)
        .bind(layer_target)
        .bind(description)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("action_id"))
    }

    pub async fn get_mitigation_actions(&self) -> Result<Vec<MitigationAction>, Error> {
        sqlx::query_as::<_, MitigationAction>(
            "SELECT action_id, action_name, layer_target, description
             FROM mitigation_actions
             ORDER BY action_id ASC"
        )
        .fetch_all(&self.pool)
        .await
    }

    // =========================================================================
    // 3. COLA DE EJECUCIÓN (ORQUESTACIÓN)
    // =========================================================================

    pub async fn queue_task(
        &self,
        alert_id: Option<i64>,
        action_id: Option<i32>,
        node_id: Option<i32>,
        status_id: Option<i32>,
    ) -> Result<i64, Error> {
        let row = sqlx::query(
            "INSERT INTO execution_queue (alert_id, action_id, node_id, status_id)
             VALUES ($1, $2, $3, $4)
             RETURNING task_id"
        )
        .bind(alert_id)
        .bind(action_id)
        .bind(node_id)
        .bind(status_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("task_id"))
    }

    pub async fn update_task_status(&self, task_id: i64, status_id: i32) -> Result<(), Error> {
        sqlx::query(
            "UPDATE execution_queue
             SET status_id = $1,
                 executed_at = CASE WHEN $1 = 2 THEN CURRENT_TIMESTAMP ELSE executed_at END
             WHERE task_id = $2"
        )
        .bind(status_id)
        .bind(task_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // =========================================================================
    // 4. ROLLBACK SNAPSHOTS
    // =========================================================================

    pub async fn create_rollback_snapshot(
        &self,
        task_id: i64,
        pre_state_json: Value,
        rollback_payload: Value,
    ) -> Result<i64, Error> {
        let row = sqlx::query(
            "INSERT INTO rollback_snapshots (task_id, pre_state_json, rollback_payload)
             VALUES ($1, $2, $3)
             ON CONFLICT (task_id) DO UPDATE SET
                 pre_state_json = EXCLUDED.pre_state_json,
                 rollback_payload = EXCLUDED.rollback_payload
             RETURNING snapshot_id"
        )
        .bind(task_id)
        .bind(pre_state_json)
        .bind(rollback_payload)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("snapshot_id"))
    }

    // =========================================================================
    // 5. AUDITORÍA DEL SISTEMA
    // =========================================================================

    pub async fn log_audit(
        &self,
        user_id: Option<i32>,
        action: &str,
        target_table: &str,
        target_id: Option<i64>,
    ) -> Result<i64, Error> {
        let row = sqlx::query(
            "INSERT INTO system_audit_logs (user_id, action, target_table, target_id)
             VALUES ($1, $2, $3, $4)
             RETURNING sys_audit_id"
        )
        .bind(user_id)
        .bind(action)
        .bind(target_table)
        .bind(target_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("sys_audit_id"))
    }
}
