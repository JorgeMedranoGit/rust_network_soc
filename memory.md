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
    *   Pipeline de Ingesta de Alto Rendimiento (preparado para ráfagas masivas / 1M pps):
        *   **Cero Heap Allocations**: `NetworkEvent` reside 100% en el stack (`std::net::IpAddr`, enum `L4Protocol`, `u8` bitflags, implementando `Copy`).
        *   **I/O de Base de Datos 100% Desacoplado**: Ingesta principal en sub-microsegundos con `tokio::select!`. Todo el I/O hacia PostgreSQL (`insert_log` y `record_metric`) delegado a tareas secundarias en segundo plano mediante `Arc<TelemetryRepository>`.
        *   **Agregación Estadística Atómica**: Acumulación en RAM agrupada por `node_id` y vaciado atómico en cada ventana temporal mediante `std::mem::take`, previniendo fugas de memoria (`memory leaks`).
        *   **Caché de Topología en RAM ($O(1)$)**: Identificación inmediata de nodos de GNS3 (`AttackerKali`, `VictimeAlpine`, `pfSense_Gateway`, etc.) sin consultar la base de datos durante la ingesta.
        *   **Worker de Inferencia Aislado**: Canal desacoplado para la evaluación de anomalías e inferencia con LightGBM sin ralentizar la recepción de paquetes.
    *   Sincronización exacta entre el esquema SQL (`bd/init.sql`) y los repositorios de Rust.
    *   Sembrado automático de catálogos base e inventario de nodos en el arranque del sistema.
    *   Servidor Web Axum operativo en el puerto 3000 con endpoints de salud, estado, inventario, catálogos y telemetría.
    *   Captura asíncrona de red (Sniffer) funcionando bajo el Patrón Productor-Consumidor (`pcap` + `etherparse` + canales `tokio::sync::mpsc`) con soporte para LinkTypes Ethernet y Linux Cooked SLL/SLL2.
    *   Contenedores Docker estabilizados con `network_mode: "host"`, resolviendo el bucle de reinicios.
    *   **Motor de Feature Engineering Columnar con Polars:** Extracción de 23 características de tráfico (Rate, IAT, Variance, Header_Length, TTL, flags TCP, protocolos L4/L7, estadísticas agregadas) ejecutadas en microsegundos sobre micro-ventanas continuas.
    *   **Motor de Inferencia y Entrenamiento LightGBM Integrado:** 
        *   Inferencia real-time aislada con latencia sub-milisegundo (~25-50 µs) evaluada sobre tráfico vivo y ráfagas.
        *   Clasificación automática de amenazas (`DATA_EXFILTRATION`, `DDOS_SYN_FLOOD`, `PORT_SCAN`, `UNAUTHORIZED_ACCESS`), niveles de severidad (`CRITICAL`, `HIGH`, `MEDIUM`) y políticas Carrier-Grade.
        *   Persistencia forense desacoplada en PostgreSQL (`network_logs`, `feature_store`, `security_alerts`).
        *   Soporte para entrenamiento K-Fold cruzado con Polars + LightGBM (`cargo run -- train`).
        *   Exposición de métricas de rendimiento y estadísticas en endpoints REST `/api/v1/ml/stats` y `/api/v1/ml/model-info`.
*   **Fase Siguiente (Mitigación & Orquestación):**
    *   Implementación del Patrón Adapter (Traits en Rust) para la mitigación agnóstica de dispositivos de red (pfSense, FortiGate, Routers).

## 6. Instrucción para el Asistente AI
Asume el rol de Arquitecto de Software Senior y experto en Rust/Ciberseguridad. Responde con código limpio, modular y técnicamente riguroso, manteniendo un enfoque apto para una tesis universitaria de ingeniería.
*   **Estilo de Código Estricto:** 
    *   Para comentarios separadores de bloques usar siempre el formato: `// * * * NOMBRE DE SECCIÓN * * *`.
    *   Para salidas en consola estándar usar prefijos estructurados: `|- INFO -|`, `|- DB -|`, `|- PACKET -|`, `|- ALERTA -|`, `|- FATAL -|`.
    *   Si una IP no se encuentra en la caché de topología durante la ingesta de red, asignar SIEMPRE el `node_id = 6` por defecto, nunca usar `NULL` ni omitirlo.

## 7. Flujo de Trabajo Git y Gestión de Versiones (Obligatorio)
Para fines de auditoría, trazabilidad y documentación técnica de la tesis:
1. **Política de Ramas (Branching)**: NUNCA realizar `push` directo a la rama `main`.
2. **Ciclo de Desarrollo**:
   - Crear una rama secundaria con nomenclatura clara para cada cambio o sprint (ej. `feature/nombre-funcionalidad`, `fix/descripcion-arreglo`, `docs/tema`).
   - Desarrollar e implementar los cambios dentro de dicha rama.
   - Ejecutar verificaciones estrictas (`cargo check`, compilación en Docker, pruebas de endpoints/red).
   - Subir la rama secundaria a GitHub: `git push -u origin <nombre-rama>`.
3. **Fusión (Merge)**:
   - Solo cuando todas las pruebas pasen satisfactoriamente, integrar los cambios a `main` mediante `git merge <nombre-rama>`.
   - Subir la rama `main` actualizada al remoto: `git push origin main`.