// * * * REPOSITORIO DE ACLs APROBADAS EN SANDBOX (RED GNS3 SECUNDARIA VLAN 999) * * *
// Flujo de dos redes: la red GNS3 PRINCIPAL captura ataques y trafico reales;
// la red SANDBOX (mas pequena, 10.200.0.0/24) prueba cada ACL antes de usarla
// en la principal. Una ACL APROBADA se persiste aqui y alimenta futuras
// detecciones automaticas (coincidencia por threat_name / source_ip).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{Error, PgPool};

/// Fila de la tabla approved_acl_rules (spec bd/init.sql seccion 4.b)
#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ApprovedAclRule {
    pub acl_rule_id: i64,
    pub rule_name: String,
    pub layer_target: String,
    pub vendor: String,
    pub source_ip: Option<String>,
    pub dest_ip: Option<String>,
    pub protocol: Option<String>,
    pub port_range: Option<String>,
    pub action: String,
    pub applied_command: String,
    pub rollback_command: String,
    pub sandbox_verified: bool,
    pub sandbox_snapshot: Option<Value>,
    pub threat_name: Option<String>,
    pub hit_count: i64,
    pub created_by: Option<i32>,
    pub approved_at: Option<DateTime<Utc>>,
}

/// Payload de aprobacion enviado por la lamina Sandbox ACL tras validar en VLAN 999
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApproveAclDto {
    pub rule_name: String,
    #[serde(default = "default_layer")]
    pub layer_target: String,
    #[serde(default = "default_vendor")]
    pub vendor: String,
    pub source_ip: Option<String>,
    pub dest_ip: Option<String>,
    pub protocol: Option<String>,
    pub port_range: Option<String>,
    pub action: String,
    pub applied_command: String,
    pub rollback_command: String,
    pub sandbox_snapshot: Option<Value>,
    pub threat_name: Option<String>,
    pub created_by: Option<i32>,
}

fn default_layer() -> String {
    "L3".to_string()
}

fn default_vendor() -> String {
    "AclBlackhole".to_string()
}

const RETURN_COLS: &str = "acl_rule_id, rule_name, layer_target, vendor, host(source_ip) AS source_ip, host(dest_ip) AS dest_ip, protocol, port_range, action, applied_command, rollback_command, sandbox_verified, sandbox_snapshot, threat_name, hit_count, created_by, approved_at";

#[derive(Clone)]
pub struct AclRepository {
    pub pool: PgPool,
}

impl AclRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Persiste una ACL aprobada en la sandbox. Idempotente por rule_name:
    /// si ya existe, refresca el snapshot y marca sandbox_verified = TRUE.
    pub async fn approve_rule(&self, dto: &ApproveAclDto) -> Result<ApprovedAclRule, Error> {
        let sql = format!(
            r#"
            INSERT INTO approved_acl_rules (
                rule_name, layer_target, vendor, source_ip, dest_ip, protocol,
                port_range, action, applied_command, rollback_command,
                sandbox_verified, sandbox_snapshot, threat_name, created_by
            )
            VALUES ($1, $2, $3, $4::cidr, $5::cidr, $6, $7, $8, $9, $10, TRUE, $11, $12, $13)
            ON CONFLICT (rule_name) DO UPDATE SET
                sandbox_verified = TRUE,
                sandbox_snapshot = COALESCE(EXCLUDED.sandbox_snapshot, approved_acl_rules.sandbox_snapshot),
                applied_command = EXCLUDED.applied_command,
                rollback_command = EXCLUDED.rollback_command,
                approved_at = CURRENT_TIMESTAMP
            RETURNING {}
            "#,
            RETURN_COLS
        );
        let row = sqlx::query_as::<_, ApprovedAclRule>(&sql)
            .bind(&dto.rule_name)
            .bind(&dto.layer_target)
            .bind(&dto.vendor)
            .bind(dto.source_ip.as_deref().map(|s| format!("{}/32", s)))
            .bind(dto.dest_ip.as_deref().map(|s| format!("{}/32", s)))
            .bind(&dto.protocol)
            .bind(&dto.port_range)
            .bind(&dto.action)
            .bind(&dto.applied_command)
            .bind(&dto.rollback_command)
            .bind(dto.sandbox_snapshot.clone())
            .bind(&dto.threat_name)
            .bind(dto.created_by)
            .fetch_one(&self.pool)
            .await?;
        Ok(row)
    }

    /// Lista las ACLs aprobadas (para la tabla de la lamina Sandbox ACL)
    pub async fn list_approved(&self) -> Result<Vec<ApprovedAclRule>, Error> {
        let sql = format!(
            "SELECT {} FROM approved_acl_rules ORDER BY approved_at DESC LIMIT 100",
            RETURN_COLS
        );
        let rows = sqlx::query_as::<_, ApprovedAclRule>(&sql)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    /// Coincidencia para FUTURAS DETECCIONES: busca reglas aprobadas cuyo
    /// threat_name coincide con la amenaza detectada o cuyo source_ip bate con
    /// el atacante observado. Incrementa hit_count al re-aplicarse.
    pub async fn match_for_detection(
        &self,
        threat_name: &str,
        attacker_ip: &str,
    ) -> Result<Vec<ApprovedAclRule>, Error> {
        let sql = format!(
            r#"UPDATE approved_acl_rules
                  SET hit_count = hit_count + 1
                WHERE sandbox_verified = TRUE
                  AND (threat_name = $1 OR host(source_ip) = $2)
              RETURNING {}"#,
            RETURN_COLS
        );
        let rows = sqlx::query_as::<_, ApprovedAclRule>(&sql)
            .bind(threat_name)
            .bind(attacker_ip)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }
}
