// * * * INICIALIZACIÓN DEL POOL DE CONEXIONES POSTGRESQL * * *
use sqlx::{Pool, Postgres};
use std::env;

pub type DbPool = Pool<Postgres>;

pub async fn init_pool() -> Result<DbPool, sqlx::Error> {
    let _ = dotenvy::dotenv();
    
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/tigo_soc".to_string());

    println!("|- INSTRUCCION -| Conectando a la base de datos PostgreSQL...");
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(50)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&database_url)
        .await
}
