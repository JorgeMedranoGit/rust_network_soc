// * * * REPOSITORIO DE GESTIÓN DE USUARIOS, SESIONES SEGURAS Y RBAC * * *
#![allow(dead_code)]
use chrono::{Duration, Utc};
use sqlx::{Error, PgPool};
use crate::domain::models::{SysUser, UserSession, UserWithRole};

#[derive(Clone)]
pub struct UsersRepository {
    pub pool: PgPool,
}

impl UsersRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // * * * 1. CRUD DE USUARIOS DEL SISTEMA * * *

    pub async fn create_user(&self, username: &str, password_hash: &str, role_id: i32) -> Result<SysUser, Error> {
        sqlx::query_as::<_, SysUser>(
            r#"
            INSERT INTO sys_users (username, password_hash, role_id, created_at)
            VALUES ($1, $2, $3, CURRENT_TIMESTAMP)
            RETURNING user_id, username, password_hash, role_id, created_at
            "#,
        )
        .bind(username)
        .bind(password_hash)
        .bind(role_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_users_with_roles(&self) -> Result<Vec<UserWithRole>, Error> {
        sqlx::query_as::<_, UserWithRole>(
            r#"
            SELECT 
                u.user_id,
                u.username,
                u.role_id,
                r.role_name,
                r.description AS role_description,
                u.created_at
            FROM sys_users u
            LEFT JOIN roles r ON u.role_id = r.role_id
            ORDER BY u.user_id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_user_by_id(&self, user_id: i32) -> Result<Option<UserWithRole>, Error> {
        sqlx::query_as::<_, UserWithRole>(
            r#"
            SELECT 
                u.user_id,
                u.username,
                u.role_id,
                r.role_name,
                r.description AS role_description,
                u.created_at
            FROM sys_users u
            LEFT JOIN roles r ON u.role_id = r.role_id
            WHERE u.user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<SysUser>, Error> {
        sqlx::query_as::<_, SysUser>(
            r#"
            SELECT user_id, username, password_hash, role_id, created_at
            FROM sys_users
            WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn update_user_role(&self, user_id: i32, role_id: i32) -> Result<(), Error> {
        sqlx::query(
            "UPDATE sys_users SET role_id = $1 WHERE user_id = $2"
        )
        .bind(role_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_user(&self, user_id: i32) -> Result<(), Error> {
        sqlx::query("DELETE FROM sys_users WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // * * * 2. GESTIÓN DE SESIONES EN SERVIDOR (TOKENS OPACOS) * * *

    pub async fn create_session(
        &self,
        session_id: &str,
        user_id: i32,
        ttl_secs: i64,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<UserSession, Error> {
        let expires_at = Utc::now() + Duration::seconds(ttl_secs);

        sqlx::query_as::<_, UserSession>(
            r#"
            INSERT INTO user_sessions (session_id, user_id, created_at, expires_at, last_activity, last_reauth_at, is_revoked, ip_address, user_agent)
            VALUES ($1, $2, CURRENT_TIMESTAMP, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, FALSE, $4, $5)
            RETURNING session_id, user_id, created_at, expires_at, last_activity, last_reauth_at, is_revoked, ip_address, user_agent
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .bind(expires_at)
        .bind(ip_address)
        .bind(user_agent)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_valid_session(&self, session_id: &str) -> Result<Option<UserSession>, Error> {
        sqlx::query_as::<_, UserSession>(
            r#"
            SELECT session_id, user_id, created_at, expires_at, last_activity, last_reauth_at, is_revoked, ip_address, user_agent
            FROM user_sessions
            WHERE session_id = $1 AND is_revoked = FALSE AND expires_at > CURRENT_TIMESTAMP
            "#,
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn touch_session(&self, session_id: &str) -> Result<(), Error> {
        sqlx::query(
            "UPDATE user_sessions SET last_activity = CURRENT_TIMESTAMP WHERE session_id = $1"
        )
        .bind(session_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_reauth_timestamp(&self, session_id: &str) -> Result<(), Error> {
        sqlx::query(
            "UPDATE user_sessions SET last_reauth_at = CURRENT_TIMESTAMP, last_activity = CURRENT_TIMESTAMP WHERE session_id = $1"
        )
        .bind(session_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn revoke_session(&self, session_id: &str) -> Result<(), Error> {
        sqlx::query(
            "UPDATE user_sessions SET is_revoked = TRUE WHERE session_id = $1"
        )
        .bind(session_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // * * * 3. CONFIGURACIÓN DE TIMEOUT PARA ACCIONES CRÍTICAS * * *

    pub async fn get_critical_timeout_secs(&self) -> Result<i64, Error> {
        let row = sqlx::query_scalar::<_, String>(
            "SELECT param_value FROM system_parameters WHERE param_key = 'CRITICAL_ACTION_TIMEOUT_SECS'"
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|v| v.parse::<i64>().ok()).unwrap_or(300))
    }

    pub async fn set_critical_timeout_secs(&self, seconds: i64, updated_by_user_id: Option<i32>) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO system_parameters (param_key, param_value, description, last_updated_by, updated_at)
            VALUES (
                'CRITICAL_ACTION_TIMEOUT_SECS',
                $1,
                'Tiempo de inactividad máximo en segundos para re-autenticación por contraseña en mitigación y rollback',
                $2,
                CURRENT_TIMESTAMP
            )
            ON CONFLICT (param_key) DO UPDATE
            SET param_value = EXCLUDED.param_value,
                last_updated_by = EXCLUDED.last_updated_by,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(seconds.to_string())
        .bind(updated_by_user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Valida si la sesión actual tiene una re-autenticación vigente dentro del umbral configurable.
    /// Retorna `true` si la re-autenticación es válida; `false` si ha expirado y requiere contraseña.
    pub async fn is_reauth_valid(&self, session_id: &str) -> Result<bool, Error> {
        let session = match self.get_valid_session(session_id).await? {
            Some(s) => s,
            None => return Ok(false),
        };

        let last_reauth = match session.last_reauth_at {
            Some(t) => t,
            None => return Ok(false),
        };

        let timeout_secs = self.get_critical_timeout_secs().await?;
        let elapsed = (Utc::now() - last_reauth).num_seconds();

        Ok(elapsed < timeout_secs)
    }
}
