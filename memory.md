# CONTEXTO DE PROYECTO: RUST NETWORK SOC (TESIS DE GRADO)

## 1. Identidad y Objetivo
El usuario es un Ingeniero de Sistemas desarrollando su proyecto de grado. 
El objetivo es construir el backend de un Centro de Operaciones de Seguridad (SOC) automatizado, enfocado en la detección de exfiltración de datos mediante Machine Learning (LightGBM), con una arquitectura agnóstica al proveedor (Vendor-Agnostic) aplicable a ISPs.
*   **Repositorio GitHub:** `rust-network-soc`

## 2. Stack Tecnológico
*   **Sistema Operativo:** BigLinux (basado en Arch).
*   **Emulación de Red:** GNS3 (usando ubridge). Nodos: Attacker (Kali), Victims (Alpine), Firewall/Gateway (pfSense/FortiGate VM).
*   **Contenedorización:** Docker y Docker Compose.
*   **Base de Datos:** PostgreSQL 15 (esquema normalizado BCNF, uso intensivo de JSONB).
*   **Backend:** Rust (compilador estable moderno `slim-bookworm`).
    *   *Crates clave:* `tokio` (asíncrono), `sqlx` (ORM BD), `axum` (API web), `pcap` y `etherparse` (Sniffing de red), `serde_json`.

## 3. Arquitectura de Base de Datos (Diseño Optimizado)
El esquema DDL está dividido en dos mundos para evitar la saturación de disco:
*   **Mundo Estadístico / Masivo (`network_traffic_metrics`):** Almacena resúmenes por intervalos de tiempo (paquetes totales, bytes) por cada nodo. Resuelve el cálculo de porcentajes históricos de ataques sin saturar la BD con millones de paquetes limpios.
*   **Mundo Forense / Alertas (`network_logs`, `feature_store`, `security_alerts`):** Exclusivo para almacenar únicamente los ataques aislados o anomalías detectadas por LightGBM.
*   **Catálogos y RBAC:** Tablas estáticas para tipos de dispositivos, protocolos, roles y usuarios del sistema.

## 4. Arquitectura de Software (Rust)
Diseño en 3 capas encapsuladas mediante `mod.rs`:
*   `domain/`: Modelos de datos.
*   `infrastructure/`: Repositorios CRUD separados por responsabilidad (`repo_catalogs.rs`, `repo_inventory.rs`, `repo_telemetry.rs`, `repo_orchestration.rs`) y conexión a BD (`db_connection.rs`).
*   `presentation/`: Rutas API (axum).

## 5. Estado Actual del Proyecto
*   **Logros:** El entorno Docker compila perfectamente. Rust se conecta a PostgreSQL vía `sqlx` y opera con los repositorios modulares. Se ha estructurado el DDL final con la tabla de métricas agregadas.
*   **Sprint Actual (Captura de Tráfico):** Implementar la captura de red asíncrona (Sniffer) mediante el Patrón Productor-Consumidor con canales `tokio::sync::mpsc`.
    *   *Productor:* Hilo bloqueante con `pcap` escuchando la interfaz de GNS3 y parseando con `etherparse`.
    *   *Consumidor:* Tarea asíncrona procesando el flujo.
    *   *Bloqueo pendiente:* Configurar `network_mode: "host"` en el `docker-compose.yml` para permitir la escucha de red desde el contenedor.
*   **Fase Futura:** Implementar el Patrón Adapter (Traits en Rust) para la mitigación agnóstica de dispositivos.

## 6. Instrucción para el Asistente AI
Asume el rol de Arquitecto de Software Senior y experto en Rust/Ciberseguridad. Responde con código limpio, modular y técnicamente riguroso, manteniendo un enfoque apto para una tesis universitaria de ingeniería.