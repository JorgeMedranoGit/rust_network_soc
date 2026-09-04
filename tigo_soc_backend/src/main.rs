mod presentation;
mod domain;
mod infrastructure;
use infrastructure::db_connection;
use domain::models::{DeviceType, AlertStatus};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[INFO] Iniciando Tigo SOC Backend");

    let db_pool = match db_connection::init_pool().await {
        Ok(pool) => {
            println!("Conectado a la Base de Datos.");
            pool
        }
        Err(e) => {
            println!("Falló la conexión: {}", e);
            return Err(e.into());
        }
    };


    let new_device = DeviceType {
        type_id: None,
        type_name: "ROUTER_CORE_L3".to_string(),
    };

    let device_result = sqlx::query!(
        "INSERT INTO device_types (type_name) VALUES ($1) ON CONFLICT DO NOTHING",
        new_device.type_name
    )
    .execute(&db_pool)
    .await;

    match device_result {
        Ok(_) => println!("Catálogo DeviceType procesado correctamente."),
        Err(e) => println!("Fallo en DeviceType: {}", e),
    }

    let new_status = AlertStatus {
        status_id: None,
        status_name: "PENDING_ANALYSIS".to_string(),
    };

    let status_result = sqlx::query!(
        "INSERT INTO alert_statuses (status_name) VALUES ($1) ON CONFLICT DO NOTHING",
        new_status.status_name
    )
    .execute(&db_pool)
    .await;

    match status_result {
        Ok(_) => println!("Catálogo AlertStatus procesado correctamente."),
        Err(e) => println!("Fallo en AlertStatus: {}", e),
    }
    
    Ok(())
}