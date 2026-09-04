# CONTEXTO DE PROYECTO: RUST NETWORK SOC (TESIS DE GRADO)

## 1. Identidad y Objetivo
El usuario es un Ingeniero de Sistemas desarrollando su proyecto de grado. 
El objetivo es construir el backend de un Centro de Operaciones de Seguridad (SOC) automatizado, enfocado en la detección de exfiltración de datos mediante Machine Learning (LightGBM), con una arquitectura agnóstica al proveedor (Vendor-Agnostic) aplicable a ISPs.
*   **Repositorio GitHub:** `https://github.com/JorgeMedranoGit/rust_network_soc.git`

## 2. Stack Tecnológico
*   **Sistema Operativo:** BigLinux (basado en Arch).
*   **Emulación de Red:** GNS3 (usando ubridge). Nodos: Attacker (Kali), Victims (Alpine), Firewall/Gateway (pfSense/FortiGate VM).
*   **Contenedorización:** Docker y Docker Compose (`network_mode: "host"` activado).
*   **Base de Datos:** PostgreSQL 15 (esquema normalizado BCNF, uso intensivo de JSONB).
*   **Backend:** Rust (compilador estable moderno `slim-bookworm`).
    *   *Crates clave:* `tokio` (asíncrono), `sqlx` (ORM BD), `axum` (API web), `pcap` y `etherparse` (Sniffing de red con soporte multi-linktype SLL/Ethernet), `serde_json`, `dotenvy`.

## 3. Arquitectura de Base de Datos (Diseño Optimizado)
El esquema DDL está dividido en dos mundos para evitar la saturación de disco:
*   **Mundo Estadístico / Masivo (`network_traffic_metrics`):** Almacena resúmenes por intervalos de tiempo (paquetes totales, bytes) por cada nodo. Resuelve el cálculo de porcentajes históricos de ataques sin saturar la BD con millones de paquetes limpios.
*   **Mundo Forense / Alertas (`network_logs`, `feature_store`, `security_alerts`):** Exclusivo para almacenar únicamente los ataques aislados o anomalías detectadas por LightGBM.
*   **Catálogos y RBAC:** Tablas estáticas para tipos de dispositivos, protocolos, roles y usuarios del sistema.

## 4. Arquitectura de Software (Rust)
Diseño en 3 capas encapsuladas mediante `mod.rs`:
*   `domain/`: Modelos de datos sincronizados con el DDL de PostgreSQL (`DeviceType`, `NetworkTrafficMetric`, `NetworkLog`, `FeatureStore`, `SecurityAlert`, `ExecutionQueue`, `NetworkEvent`, etc.) y motor de evaluación de anomalías (`threat_detector.rs`).
*   `infrastructure/`: Repositorios CRUD separados por responsabilidad (`repo_catalogs.rs`, `repo_inventory.rs`, `repo_telemetry.rs`, `repo_orchestration.rs`), conexión a BD (`db_connection.rs`) y adaptador de red asíncrono (`network_adapter.rs`).
*   `presentation/`: Servidor API REST con Axum (`api_routes.rs`) y canales de transmisión de eventos en tiempo real (`telemetry_stream.rs`).

## 5. Estado Actual del Proyecto
*   **Logros:**
    *   Arquitectura de 3 capas 100% implementada, modular y compilando limpiamente con 0 errores y 0 warnings.
    *   Sincronización exacta entre el esquema SQL (`bd/init.sql`) y los repositorios de Rust.
    *   Sembrado automático de catálogos base en el arranque del sistema.
    *   Servidor Web Axum operativo en el puerto 3000 con endpoints de salud, estado, inventario, catálogos y telemetría.
    *   Captura asíncrona de red (Sniffer) funcionando bajo el Patrón Productor-Consumidor (`pcap` + `etherparse` + canales `tokio::sync::mpsc`) con soporte para LinkTypes Ethernet y Linux Cooked SLL/SLL2.
    *   Contenedores Docker estabilizados con `network_mode: "host"`, resolviendo el bucle de reinicios.
*   **Fase Siguiente (Sprint ML & Mitigación):**
    *   Integración del modelo Machine Learning (LightGBM / Polars) sobre la telemetría agregada y extracción de vectores de características en `feature_store`.
    *   Implementación del Patrón Adapter (Traits en Rust) para la mitigación agnóstica de dispositivos de red (pfSense, FortiGate, Routers).

## 6. Instrucción para el Asistente AI
Asume el rol de Arquitecto de Software Senior y experto en Rust/Ciberseguridad. Responde con código limpio, modular y técnicamente riguroso, manteniendo un enfoque apto para una tesis universitaria de ingeniería.
