// * * * INICIALIZACIÓN DEL POOL DE CONEXIONES POSTGRESQL * * *
use sqlx::{PgPool, Pool, Postgres};
use std::env;

pub type DbPool = Pool<Postgres>;

pub async fn init_pool() -> Result<DbPool, sqlx::Error> {
    let _ = dotenvy::dotenv();
    
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/tigo_soc".to_string());

    println!("|- INSTRUCCION -| Conectando a la base de datos PostgreSQL...");
    PgPool::connect(&database_url).await
}
