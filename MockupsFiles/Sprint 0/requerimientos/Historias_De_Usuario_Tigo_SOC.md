# Product Backlog - Historias de Usuario (HU)
**Proyecto:** Sistema Web Aplicando LightGBM para Detección de Exfiltración de Datos  
**Caso:** Tigo Business Bolivia (AS27839)  
**Autor:** Jorge Esteban Medrano Chacolla | **Metodología:** Scrum One-Person  

---

## Resumen Ejecutivo de Estimaciones

| ID | Título de la Historia de Usuario | Sprint | RF / RNF Asociado | Puntos de Función | Prioridad |
| :---: | :--- | :---: | :---: | :---: | :---: |
| **HU-01** | Autenticación Multifactorial (2FA TOTP) y Hashing Criptográfico Argon2id | Sprint 1 | RF1, RNF5 | 42 | Must Have |
| **HU-02** | Control de Acceso Basado en Roles (RBAC) y Autorización Granular | Sprint 1 | RF2 | 38 | Must Have |
| **HU-03** | Rotación Obligatoria de Contraseñas y Verificación de Entropía Criptográfica | Sprint 1 | RF3 | 35 | Must Have |
| **HU-04** | Gestión Activa, Renovación y Revocación de Sesiones Concurrentes | Sprint 1 | RF1, RF19 | 36 | Must Have |
| **HU-05** | Pipeline de Extracción Columnar de 23 Características de Red con Polars SIMD | Sprint 2 | RF5, RNF2 | 50 | Must Have |
| **HU-06** | Inferencia Probabilística de Exfiltración de Datos Sub-milisegundo con LightGBM C-API | Sprint 2 | RF6, RNF1, RNF4 | 55 | Must Have |
| **HU-07** | Detección Especializada de Canales Encubiertos DNS Tunneling y Fugas Lentas (Slow & Low) | Sprint 2 | RF7 | 48 | Must Have |
| **HU-08** | Validación Cruzada K-Fold y Explicabilidad de Modelos con SHAP y Feature Importance | Sprint 2 | RF12, RNF4 | 44 | Must Have |
| **HU-09** | Ingesta Asíncrona Continua de Tráfico de Red con AF_PACKET / eBPF | Sprint 3 | RF4, RNF3 | 52 | Must Have |
| **HU-10** | Descubrimiento Pasivo de Activos y Resolución de Fabricante OUI en Inventario | Sprint 3 | RF11 | 39 | Must Have |
| **HU-11** | Agregación Temporal de Métricas de Red y Persistencia Desacoplada | Sprint 3 | RF8, RNF12 | 41 | Must Have |
| **HU-12** | Streaming de Telemetría en Tiempo Real a 60 FPS mediante Canal WebSocket | Sprint 3 | RF10, RNF9 | 46 | Must Have |
| **HU-13** | Análisis de Radio de Impacto (Blast Radius) y Re-autenticación Crítica con Ventana de 300s | Sprint 4 | RF16, RNF7 | 47 | Must Have |
| **HU-14** | Contención en Capa 2 mediante Aislamiento de Puertos y VLAN de Cuarentena vía RADIUS CoA | Sprint 4 | RF13 | 51 | Must Have |
| **HU-15** | Mitigación en Capas 3 y 4 mediante Inyección de Rutas BGP Blackhole (RTBH) y ACLs | Sprint 4 | RF14 | 53 | Must Have |
| **HU-16** | Mitigación en Capa 7 de Aplicación mediante Redirección Preventiva por DNS Sinkholing | Sprint 4 | RF15 | 46 | Must Have |
| **HU-17** | Mecanismo de Reversión Automática (Rollback) con Instantáneas JSON y Temporizador de 300s | Sprint 4 | RF17 | 49 | Must Have |
| **HU-18** | Entorno Sandbox Aislado (VLAN 999) para Validación Previa de Reglas Sintéticas | Sprint 4 | RF18, RNF8 | 45 | Must Have |
| **HU-19** | Cuadro de Mando Ejecutivo Interactivo con KPIs de Detección y Conmutación Mocha/Latte | Sprint 5 | RF20, RNF9 | 43 | Must Have |
| **HU-20** | Despacho de Alertas y Notificaciones Push en Tiempo Real mediante Firebase Cloud Messaging | Sprint 5 | RF9 | 45 | Must Have |
| **HU-21** | Pistas de Auditoría Inmutables del Sistema y Exportación de Expedientes Forenses | Sprint 5 | RF19, RNF6 | 44 | Must Have |
| **HU-22** | Consola Web Reactiva Multiplataforma (Desktop / Tablet / Móvil PWA) en Rust WebAssembly | Sprint 5 | RF20, RNF10 | 48 | Must Have |
| **TOTAL** | **22 Historias de Usuario completadas** | - | - | **997** | - |

---

### Historia de Usuario (HU-01)

| Elemento | Contenido |
| :--- | :--- |
| **Título** | Autenticación Multifactorial (2FA TOTP) y Hashing Criptográfico Argon2id |
| **Descripción** | Como analista del SOC, quiero autenticarme de forma segura mediante credenciales protegidas con Argon2id y un segundo factor de autenticación basado en tiempo (TOTP RFC 6238), para garantizar que sólo personal autorizado acceda a la consola operativa de Tigo Business. |
| **Criterios de Aceptación** |   * Validar credenciales de usuario mediante función de derivación de claves Argon2id (mínimo 64MB RAM, 3 iteraciones, paralelismo 4).
  * Generar y verificar códigos TOTP de 6 dígitos con ventana de deriva temporal de ±30 segundos.
  * Emitir tokens de sesión criptográficos opacos de 64 caracteres en hexadecimal tras validación exitosa.
  * Bloquear temporalmente la cuenta tras 5 intentos fallidos consecutivos mediante rate limiting. |
| **Notas Técnicas** |   * Integración del crate argon2 en Rust con configuración defensiva recomendada por OWASP.
  * Persistencia de sesiones en tabla user_sessions con llaves foráneas a sys_users.
  * Generación y validación de HMAC-SHA1 con crate totp-rs.
  * Endpoints REST POST /api/v1/auth/login y /api/v1/auth/verify-totp en Axum. |
| **Tareas Relacionadas** |   * Diseñar migración SQL para tablas sys_users y user_sessions.
  * Desarrollar módulo de seguridad Argon2SecurityEngine en Rust.
  * Implementar vista de Login con soporte 2FA en componente Leptos WASM.
  * Escribir pruebas unitarias de hashing y resistencia a ataques de diccionario. |
| **Valor de Negocio** | Mitiga el riesgo de compromiso de credenciales y accesos no autorizados al SOC de Tigo Business, garantizando el cumplimiento estricto del estándar ISO/IEC 27001 y políticas Zero-Trust. |
| **Puntos de Función (estimados)** | 42 |

---

### Historia de Usuario (HU-02)

| Elemento | Contenido |
| :--- | :--- |
| **Título** | Control de Acceso Basado en Roles (RBAC) y Autorización Granular |
| **Descripción** | Como administrador de seguridad, quiero asignar roles diferenciados (SOC_L1, SOC_L2, SOC_ADMIN, AUDITOR) con permisos granulares a los usuarios, para restringir la ejecución de acciones críticas según el principio de mínimo privilegio. |
| **Criterios de Aceptación** |   * Definir permisos atómicos en base de datos (USER_MANAGE, MITIGATE_EXECUTE, ROLLBACK_EXECUTE, READ_ONLY).
  * Validar autorizaciones en middleware de Axum interceptando cada petición HTTP y WebSocket.
  * Denegar el acceso con código HTTP 403 Forbidden a usuarios que intenten ejecutar mitigaciones sin el rol L2/Admin.
  * Permitir a los auditores el acceso exclusivo de solo lectura a métricas e informes. |
| **Notas Técnicas** |   * Tablas relacionales roles, permissions y role_permissions en PostgreSQL BCNF.
  * Middleware de extracción de claims y roles en Rust axum::middleware::from_fn.
  * Transmisión de token seguro en cabecera Authorization: Bearer <token>. |
| **Tareas Relacionadas** |   * Poblar catálogos iniciales de roles y permisos en bd/init.sql.
  * Implementar middleware de validación RBAC en Axum.
  * Adaptar componentes visuales en Leptos para ocultar botones críticos a roles no autorizados.
  * Pruebas de regresión de seguridad para validar denegación de privilegios indebidos. |
| **Valor de Negocio** | Asegura el principio de mínimo privilegio y segregación de funciones, previniendo errores operativos catastróficos o sabotajes internos en la red corporativa de Tigo. |
| **Puntos de Función (estimados)** | 38 |

---

### Historia de Usuario (HU-03)

| Elemento | Contenido |
| :--- | :--- |
| **Título** | Rotación Obligatoria de Contraseñas y Verificación de Entropía Criptográfica |
| **Descripción** | Como analista del SOC, quiero ser notificado y forzado a rotar periódicamente mi contraseña cumpliendo umbrales mínimos de entropía, para prevenir ataques de fuerza bruta y garantizar la longevidad criptográfica del sistema. |
| **Criterios de Aceptación** |   * Calcular dinámicamente la entropía en bits de la contraseña propuesta (mínimo 60 bits).
  * Forzar expiración automática de credenciales cada 90 días según parámetro configurable en system_parameters.
  * Rechazar contraseñas previamente utilizadas en los últimos 5 ciclos de rotación.
  * Mostrar indicador visual reactivo de fortaleza de contraseña en la interfaz de usuario. |
| **Notas Técnicas** |   * Algoritmo de cálculo de entropía de Shannon implementado en Rust.
  * Consulta y actualización de tabla system_parameters (key = 'password_rotation_days').
  * Registro de fecha de última actualización en sys_users.updated_at. |
| **Tareas Relacionadas** |   * Implementar lógica de cálculo de entropía en security.rs.
  * Crear endpoint POST /api/v1/auth/change-password con validación de historial.
  * Diseñar modal interactivo de rotación en Leptos WASM con barra de progreso reactiva. |
| **Valor de Negocio** | Cumple con los controles A.9.4.3 de la norma ISO/IEC 27001 para la gestión de contraseñas de analistas de seguridad, blindando la infraestructura contra ataques de credenciales estáticas. |
| **Puntos de Función (estimados)** | 35 |

---

### Historia de Usuario (HU-04)

| Elemento | Contenido |
| :--- | :--- |
| **Título** | Gestión Activa, Renovación y Revocación de Sesiones Concurrentes |
| **Descripción** | Como administrador de seguridad, quiero supervisar las sesiones activas en tiempo real y poder revocar accesos sospechosos o concurrentes inmediatamente, para neutralizar secuestros de sesión en la consola. |
| **Criterios de Aceptación** |   * Registrar dirección IP de origen, User-Agent, fecha de inicio y última actividad en cada sesión.
  * Proveer botón de revocación instantánea que invalide el token tanto en PostgreSQL como en caché de memoria.
  * Terminar automáticamente sesiones inactivas que superen los 15 minutos (900 segundos).
  * Impedir más de dos sesiones concurrentes por cuenta de usuario simultáneamente. |
| **Notas Técnicas** |   * Consulta y actualización atómica en tabla user_sessions (is_revoked = true).
  * Canal de invalidación en memoria mediante Tokio RwLock.
  * Desconexión inmediata de conexiones WebSockets asociadas al token revocado. |
| **Tareas Relacionadas** |   * Crear repositorio repo_users.rs con métodos de consulta y revocación.
  * Diseñar vista de gestión de sesiones en componente UserManagement.rs.
  * Pruebas de integración simulando ataques de reuso de tokens revocados. |
| **Valor de Negocio** | Evita la persistencia de sesiones comprometidas y garantiza que las estaciones desatendidas en el SOC se cierren automáticamente, preservando la confidencialidad de la operación. |
| **Puntos de Función (estimados)** | 36 |

---

### Historia de Usuario (HU-05)

| Elemento | Contenido |
| :--- | :--- |
| **Título** | Pipeline de Extracción Columnar de 23 Características de Red con Polars SIMD |
| **Descripción** | Como motor de inferencia, quiero procesar lotes de eventos de red mediante estructuras columnares Polars aprovechando instrucciones SIMD, para calcular 23 características estadísticas y de flujo en menos de 500 microsegundos en memoria RAM. |
| **Criterios de Aceptación** |   * Extraer 23 métricas estadísticas (entropía de bytes, desviación de paquetes, tasas SYN/ACK, varianza de TTL, frecuencias DNS).
  * Ejecutar la extracción columnar en un tiempo promedio inferior a 500 µs por lote de 1,000 paquetes.
  * Procesar datos sin copias innecesarias en memoria (zero-copy) ni bloqueos de concurrencia.
  * Serializar los vectores resultantes al formato tabular para consumo directo del booster LightGBM. |
| **Notas Técnicas** |   * Uso de la biblioteca polars en Rust compilada con soporte SIMD AVX2.
  * Implementación del módulo PolarsFeatureEngine en tigo_soc_backend/src/domain/feature_engine.rs.
  * Mapeo a estructura ExtractedFeatures con conversión a slices &[f32]. |
| **Tareas Relacionadas** |   * Definir esquema de las 23 características optimizadas para detección de exfiltración.
  * Implementar cálculo de entropía de información de Shannon vectorizado.
  * Desarrollar pruebas unitarias de rendimiento y benchmarking con criterion. |
| **Valor de Negocio** | Permite procesar el tráfico carrier masivo de Tigo sin saturar la CPU del servidor, logrando una inspección continua y profunda sin demoras de búfer. |
| **Puntos de Función (estimados)** | 50 |

---

### Historia de Usuario (HU-06)

| Elemento | Contenido |
| :--- | :--- |
| **Título** | Inferencia Probabilística de Exfiltración de Datos Sub-milisegundo con LightGBM C-API |
| **Descripción** | Como analista del SOC, quiero que el motor analítico evalúe en tiempo real la probabilidad de fuga de datos en tiempo sub-milisegundo (< 1.0 ms), para identificar incidentes de exfiltración de forma casi instantánea antes de que se completen. |
| **Criterios de Aceptación** |   * Invocar la librería compartida lib_lightgbm.so mediante enlaces FFI nativos C-API libres del GIL de Python.
  * Alcanzar una latencia de inferencia inferior a 0.1 ms (meta alcanzada: ~0.08 ms).
  * Emitir una puntuación de anomalía (anomaly_score) normalizada entre 0.0 y 1.0.
  * Mantener un F1-Score >= 99.0% y tasa de falsos positivos (FPR) < 0.5% sobre tráfico validado. |
| **Notas Técnicas** |   * Wrapper BoosterWrapper y ThreatDetector en Rust con tipos Send + Sync.
  * Modelo compilado con árboles GOSS (Gradient-based One-Side Sampling) y EFB (Exclusive Feature Bundling).
  * Almacenamiento de alertas en tabla security_alerts de PostgreSQL. |
| **Tareas Relacionadas** |   * Configurar enlaces nativos FFI hacia lib_lightgbm en tigo_soc_backend.
  * Entrenar y serializar el modelo final ModelTrainedLightGBM.
  * Implementar pruebas de estrés de inferencia a 100,000 predicciones por segundo. |
| **Valor de Negocio** | Constituye el núcleo de la tesis de grado, reduciendo el tiempo medio de detección (MTTD) de horas a fracciones de milisegundo, salvaguardando la confidencialidad de datos corporativos de Tigo. |
| **Puntos de Función (estimados)** | 55 |

---

### Historia de Usuario (HU-07)

| Elemento | Contenido |
| :--- | :--- |
| **Título** | Detección Especializada de Canales Encubiertos DNS Tunneling y Fugas Lentas (Slow & Low) |
| **Descripción** | Como analista de ciberseguridad, quiero detectar canales encubiertos sofisticados (DNS Tunneling mediante dnscat2/iodined y tráfico exfiltrado a baja velocidad), para neutralizar técnicas avanzadas que evaden firewalls perimetrales tradicionales. |
| **Criterios de Aceptación** |   * Analizar la longitud de subdominios, ratio de entropía en nombres FQDN y porcentaje de registros TXT/NULL.
  * Detectar flujos de exfiltración sigilosa (Slow & Low) mediante correlación de ventanas temporales deslizantes.
  * Clasificar el tipo de amenaza en tabla threat_types (DNS_TUNNELING, HTTP_POST_EXFIL, ICMP_TUNNEL).
  * Generar alerta forense indexando el vector de características en feature_store. |
| **Notas Técnicas** |   * Analizador de protocolos DNS implementado en network_adapter.rs.
  * Inserción relacional en network_logs y feature_store con tipo JSONB en PostgreSQL.
  * Umbrales adaptativos de entropía basados en el modelo entrenado. |
| **Tareas Relacionadas** |   * Generar datasets sintéticos de tráfico con dnscat2 e iodined en laboratorio GNS3.
  * Calibrar pesos de características específicas para consultas DNS anómalas.
  * Validar matriz de confusión para subtipos de ataque en pruebas controladas. |
| **Valor de Negocio** | Cubre el principal vector ciego de exfiltración en redes corporativas (el protocolo DNS comúnmente no inspeccionado), evitando fugas de información confidencial sin elevar falsas alarmas. |
| **Puntos de Función (estimados)** | 48 |

---

### Historia de Usuario (HU-08)

| Elemento | Contenido |
| :--- | :--- |
| **Título** | Validación Cruzada K-Fold y Explicabilidad de Modelos con SHAP y Feature Importance |
| **Descripción** | Como operador del SOC, quiero visualizar la justificación explicable (XAI) de cada alerta de exfiltración mediante valores de importancia de características (Gain / SHAP), para comprender la causa raíz de la anomalía y validar el modelo estadísticamente. |
| **Criterios de Aceptación** |   * Presentar las 5 características determinantes que dispararon el score de anomalía (ej. longitud de consulta DNS, entropía).
  * Reportar resultados de validación cruzada estratificada K-Fold (k=5) con métricas de Precisión, Recall y AUC-ROC.
  * Proveer desglose de matriz de confusión en vivo (Verdaderos Positivos, Falsos Positivos, etc.).
  * Asegurar que la explicación se compute sin penalizar la velocidad de la ruta crítica de inferencia. |
| **Notas Técnicas** |   * Módulo LightGBMTrainer en Rust para cálculo de estadísticas de ganancia (Gain) y división (Split).
  * Serialización de métricas en DTOs consumibles vía REST GET /api/v1/ml/metrics.
  * Renderizado de gráficos de barras SVG en componente Leptos WASM monitoring_chart.rs. |
| **Tareas Relacionadas** |   * Implementar cálculo de pesos de árboles y ganancias por nodo.
  * Crear endpoint REST para entrega de métricas de desempeño y curvas ROC.
  * Diseñar panel visual de Inteligencia Artificial Explicable (XAI) en el frontend. |
| **Valor de Negocio** | Elimina el problema de la 'caja negra' en IA, brindando confianza analítica a los operadores humanos y auditores para fundamentar acciones de aislamiento sobre clientes carrier. |
| **Puntos de Función (estimados)** | 44 |

---

### Historia de Usuario (HU-09)

| Elemento | Contenido |
| :--- | :--- |
| **Título** | Ingesta Asíncrona Continua de Tráfico de Red con AF_PACKET / eBPF |
| **Descripción** | Como sonda de captura de red, quiero capturar tramas Ethernet promiscuas a nivel de kernel mediante sockets raw AF_PACKET y filtros eBPF sin copia de memoria, para procesar hasta 15,000 pps sin pérdida de paquetes en interfaces de Tigo Business. |
| **Criterios de Aceptación** |   * Capturar tráfico en interfaces físicas y virtuales (TAP/SPAN) de la red AS27839.
  * Implementar búfer de anillo circular (ring buffer) de tamaño configurable en memoria RAM.
  * Descartar paquetes corruptos o truncados a nivel de kernel mediante filtros eBPF.
  * Transferir tramas decodificadas a canales mpsc de Tokio sin bloquear el hilo principal. |
| **Notas Técnicas** |   * Módulo NetworkAdapter en tigo_soc_backend/src/infrastructure/network_adapter.rs.
  * Sockets Linux AF_PACKET en modo PACKET_MMAP para cero copias de memoria.
  * Control de concurrencia mediante primitivas atómicas Arc<AtomicBool> y canales asíncronos. |
| **Tareas Relacionadas** |   * Configurar script de inicio y privilegios CAP_NET_RAW / CAP_NET_ADMIN en Linux.
  * Diseñar analizador de tramas Ethernet, cabeceras IPv4/IPv6, TCP y UDP.
  * Ejecutar pruebas de saturación con iperf3 y generadores de paquetes sintéticos. |
| **Valor de Negocio** | Garantiza la visibilidad total del tráfico en el borde de la red carrier de Tigo sin degradar el ancho de banda de los enlaces de producción ni requerir hardware propietario costoso. |
| **Puntos de Función (estimados)** | 52 |

---

### Historia de Usuario (HU-10)

| Elemento | Contenido |
| :--- | :--- |
| **Título** | Descubrimiento Pasivo de Activos y Resolución de Fabricante OUI en Inventario |
| **Descripción** | Como operador de red, quiero que el sistema descubra pasivamente nuevos hosts en la subred y resuelva el fabricante de la interfaz mediante prefijos OUI, para mantener el inventario de nodos de red actualizado automáticamente. |
| **Criterios de Aceptación** |   * Interceptar paquetes ARP y de difusión local para extraer pares MAC-IP en tiempo real.
  * Resolver fabricante del hardware (Cisco, MikroTik, Fortinet, Huawei) mediante base de datos OUI local.
  * Registrar nodos no identificados en la tabla network_nodes con estado inicial pendiente.
  * Asignar tipos de dispositivo (device_types) y protocolos de gestión (mgmt_protocols) configurables. |
| **Notas Técnicas** |   * Motor AssetDiscoveryEngine en tigo_soc_backend/src/infrastructure/asset_discovery.rs.
  * Almacenamiento en tabla network_nodes en PostgreSQL BCNF.
  * Caché interna de prefijos OUI en HashMap en memoria para resolución en O(1). |
| **Tareas Relacionadas** |   * Cargar tabla de prefijos IEEE OUI en archivo de catálogo local.
  * Desarrollar lógica de inserción condicional en base de datos (ON CONFLICT DO NOTHING).
  * Exponer endpoints REST para consulta y edición de nodos inventariados. |
| **Valor de Negocio** | Elimina el inventario manual obsoleto en hojas de cálculo, permitiendo detectar de inmediato la conexión de dispositivos no autorizados (Rogue Devices) en el perímetro carrier de Tigo. |
| **Puntos de Función (estimados)** | 39 |

---

### Historia de Usuario (HU-11)

| Elemento | Contenido |
| :--- | :--- |
| **Título** | Agregación Temporal de Métricas de Red y Persistencia Desacoplada |
| **Descripción** | Como analista del SOC, quiero que las métricas de tráfico continuo se agreguen en ventanas periódicas de 1 segundo en base de datos, separando las estadísticas generales de los registros forenses detallados, para evitar la saturación de almacenamiento en disco. |
| **Criterios de Aceptación** |   * Consolidar totales de paquetes y bytes por nodo en intervalos de ventana temporal (window_start, window_end).
  * Persistir en la tabla network_traffic_metrics usando sentencias INSERT por lotes (batching).
  * Aislar el almacenamiento de eventos forenses completos en network_logs únicamente cuando exista anomalía.
  * Proveer consultas históricas de throughput en Mbps y paquetes por segundo (pps) optimizadas por índices. |
| **Notas Técnicas** |   * Repositorio TelemetryRepository en tigo_soc_backend/src/infrastructure/repo_telemetry.rs.
  * Restricción de unicidad relacional UNIQUE(node_id, window_start) para prevenir duplicados.
  * Ejecución de sentencias preparadas con el pool sqlx::PgPool. |
| **Tareas Relacionadas** |   * Crear índices B-Tree en network_traffic_metrics por nodo y marca de tiempo.
  * Implementar tarea de recolección periódica en segundo plano con Tokio interval.
  * Diseñar consultas agregadas para cálculo de tendencias en el frontend. |
| **Valor de Negocio** | Optimiza la huella de almacenamiento en base de datos en más del 80%, manteniendo la capacidad forense detallada únicamente cuando se requiere para responder a incidentes. |
| **Puntos de Función (estimados)** | 41 |

---

### Historia de Usuario (HU-12)

| Elemento | Contenido |
| :--- | :--- |
| **Título** | Streaming de Telemetría en Tiempo Real a 60 FPS mediante Canal WebSocket |
| **Descripción** | Como analista del SOC, quiero recibir un flujo continuo de telemetría y eventos de tráfico a 60 cuadros por segundo mediante WebSockets, para monitorear la salud de la red y las anomalías en vivo sin recargar el navegador. |
| **Criterios de Aceptación** |   * Establecer canal de comunicación bidireccional seguro WSS en /api/v1/telemetry/stream.
  * Difundir métricas agregadas y alertas a todos los clientes web conectados con latencia de red < 50 ms.
  * Manejar reconexión automática en el cliente frontend si ocurre una desconexión transitoria.
  * Limitar la carga de serialización mediante payloads compactos JSON o binarios. |
| **Notas Técnicas** |   * Manejador TelemetryStreamHandler con canal tokio::sync::broadcast de alta concurrencia.
  * Módulo de presentación en tigo_soc_backend/src/presentation/telemetry_stream.rs.
  * Cliente WebSocket reactivo en Rust WASM utilizando el crate gloo-net. |
| **Tareas Relacionadas** |   * Implementar endpoint de upgrade HTTP a WebSocket en Axum.
  * Desarrollar bucle de broadcast con control de backpressure.
  * Integrar suscripción reactiva en el componente app.rs de Leptos. |
| **Valor de Negocio** | Proporciona una experiencia de monitoreo situacional instantánea para los operadores de primera línea de Tigo, permitiendo detectar ataques en el mismo segundo en que inician. |
| **Puntos de Función (estimados)** | 46 |

---

### Historia de Usuario (HU-13)

| Elemento | Contenido |
| :--- | :--- |
| **Título** | Análisis de Radio de Impacto (Blast Radius) y Re-autenticación Crítica con Ventana de 300s |
| **Descripción** | Como operador de seguridad, quiero que el sistema analice el radio de impacto de cualquier mitigación antes de aplicarla y solicite re-autenticación por contraseña si involucra equipos críticos, para evitar cortes involuntarios de servicios esenciales. |
| **Criterios de Aceptación** |   * Evaluar si la IP destino corresponde a un Gateway predeterminado, servidor DNS primario o enlace troncal BGP.
  * Categorizar mitigaciones entre Singulares (bajo riesgo, automáticas) y Masivas (alto riesgo, aprobación obligatoria).
  * Requerir ingreso de contraseña o 2FA si la sesión del analista superó los 300 segundos sin re-autenticación.
  * Bloquear la ejecución si el informe de Blast Radius detecta afectación sobre servicios troncales sin override explícito. |
| **Notas Técnicas** |   * Algoritmo de evaluación en tigo_soc_backend/src/mitigation/mod.rs.
  * Parámetro global critical_reauth_timeout_secs = 300 en system_parameters.
  * Estructura BlastRadiusReport retornada al frontend para confirmación del analista. |
| **Tareas Relacionadas** |   * Implementar validador de direcciones IP críticas en la infraestructura de Tigo.
  * Diseñar modal de confirmación y re-autenticación en componente BlastRadiusControl.rs.
  * Pruebas de seguridad simulando intentos de mitigación sobre routers core. |
| **Valor de Negocio** | Protege la continuidad operativa de los servicios de telecomunicaciones de Tigo Business, garantizando que una acción de contención nunca cause un apagón en la infraestructura crítica. |
| **Puntos de Función (estimados)** | 47 |

---

### Historia de Usuario (HU-14)

| Elemento | Contenido |
| :--- | :--- |
| **Título** | Contención en Capa 2 mediante Aislamiento de Puertos y VLAN de Cuarentena vía RADIUS CoA |
| **Descripción** | Como analista del SOC, quiero desconectar o forzar la reasignación de un dispositivo infectado a una VLAN de cuarentena aislada mediante paquetes RADIUS CoA, para cortar la comunicación lateral del atacante en la red de acceso local. |
| **Criterios de Aceptación** |   * Transmitir paquetes RADIUS Change of Authorization (CoA RFC 3576) o Disconnect-Request hacia el NAS (Switch/WLC).
  * Reasignar al host atacante a la VLAN 999 (Sandbox/Cuarentena) preservando la trazabilidad de su dirección MAC.
  * Recibir confirmación CoA-ACK del conmutador de red en menos de 2 segundos.
  * Registrar el resultado en la tabla de auditoría mitigation_audit. |
| **Notas Técnicas** |   * Cliente RadiusCoaClient en tigo_soc_backend/src/mitigation/radius_coa.rs.
  * Soporte de atributos RFC 2865 y RFC 3576 sobre socket UDP hacia puerto 3799.
  * Manejo de secretos compartidos configurados por equipo en network_nodes. |
| **Tareas Relacionadas** |   * Implementar codificación y firma criptográfica de paquetes RADIUS CoA en Rust.
  * Configurar escenario de pruebas con Cisco ISE / FreeRADIUS en entorno de laboratorio.
  * Diseñar botón de aislamiento rápido en la tabla de alertas del frontend. |
| **Valor de Negocio** | Permite contener ataques de ransomware y exfiltración interna en la misma boca de red o punto de acceso WiFi del usuario comprometido sin afectar al resto del segmento empresarial. |
| **Puntos de Función (estimados)** | 51 |

---

### Historia de Usuario (HU-15)

| Elemento | Contenido |
| :--- | :--- |
| **Título** | Mitigación en Capas 3 y 4 mediante Inyección de Rutas BGP Blackhole (RTBH) y ACLs |
| **Descripción** | Como ingeniero de seguridad, quiero inyectar rutas BGP hacia la interfaz Null0 o aplicar reglas ACL en routers de borde, para descartar inmediatamente el tráfico hacia servidores de comando y control o destinos maliciosos en Internet. |
| **Criterios de Aceptación** |   * Generar y enviar comandos de enrutamiento estático o BGP Remotely Triggered Black Hole hacia routers de borde (Cisco/MikroTik).
  * Inyectar rutas /32 apuntando a Null0 para neutralizar la IP de exfiltración en toda la red autónoma AS27839.
  * Validar la aplicación de la regla mediante lectura de respuesta SSH/RESTCONF del router.
  * Generar la instantánea de reversión correspondiente antes de la inyección. |
| **Notas Técnicas** |   * Cliente AclBlackholeClient en tigo_soc_backend/src/mitigation/acl_blackhole.rs.
  * Conectores específicos para Cisco IOS (cisco.rs) y RouterOS MikroTik (mikrotik.rs).
  * Comunicación segura por SSH con el crate ssh2 o RESTCONF HTTPS. |
| **Tareas Relacionadas** |   * Desarrollar pool de conexiones SSH persistentes hacia equipos de borde.
  * Implementar pruebas de inyección y remoción de rutas nulas en topología virtualizada.
  * Integrar registros de cola de ejecución en tabla execution_queue. |
| **Valor de Negocio** | Aísla amenazas externas a escala de proveedor carrier (ISP/Telco), mitigando la fuga masiva de datos hacia destinos no autorizados en milisegundos a nivel de red global. |
| **Puntos de Función (estimados)** | 53 |

---

### Historia de Usuario (HU-16)

| Elemento | Contenido |
| :--- | :--- |
| **Título** | Mitigación en Capa 7 de Aplicación mediante Redirección Preventiva por DNS Sinkholing |
| **Descripción** | Como analista del SOC, quiero redirigir las consultas dirigidas a dominios maliciosos hacia un servidor sumidero (Sinkhole) controlado, para neutralizar canales encubiertos y capturar telemetría del malware sin alertar al atacante. |
| **Criterios de Aceptación** |   * Inyectar respuestas DNS manipuladas asociando el dominio malicioso a la IP del Sinkhole interno.
  * Actualizar la zona DNS de los servidores de nombres corporativos mediante API o script dinámico.
  * Capturar las peticiones HTTP/TCP residuales dirigidas al Sinkhole para enriquecer el expediente forense.
  * Permitir la restauración inmediata del registro DNS original al solicitarse el rollback. |
| **Notas Técnicas** |   * Cliente DnsSinkholeClient en tigo_soc_backend/src/mitigation/dns_sinkhole.rs.
  * Integración con servidores DNS BIND9 / CoreDNS o APIs de firewalls perimetrales.
  * Almacenamiento del estado original de resolución en rollback_snapshots. |
| **Tareas Relacionadas** |   * Crear conector de actualización dinámica de registros DNS en Rust.
  * Configurar servidor sumidero receptor en laboratorio GNS3.
  * Validar interceptación de tráfico de prueba con herramientas de tunneling. |
| **Valor de Negocio** | Corta la exfiltración a nivel de aplicación e inhabilita las comunicaciones de mando y control (C2), obteniendo evidencia forense de qué máquinas internas están infectadas sin causar caídas de red. |
| **Puntos de Función (estimados)** | 46 |

---

### Historia de Usuario (HU-17)

| Elemento | Contenido |
| :--- | :--- |
| **Título** | Mecanismo de Reversión Automática (Rollback) con Instantáneas JSON y Temporizador de 300s |
| **Descripción** | Como analista del SOC, quiero disponer de un botón de reversión inmediata (Rollback) con temporizador visual de 300 segundos respaldado por instantáneas de configuración pre-estado, para restaurar el servicio en segundos si la mitigación resultó ser un falso positivo. |
| **Criterios de Aceptación** |   * Capturar el estado exacto del dispositivo (pre_state_json) antes de ejecutar cualquier acción de contención.
  * Generar automáticamente la carga útil de reversión (rollback_payload) capaz de deshacer la regla aplicada.
  * Mostrar un temporizador regresivo de 300 segundos en la barra superior de la consola tras cada mitigación.
  * Al pulsar 'Revertir' o al expirar sin confirmación ante fallo de conectividad, ejecutar la orden inversa inmediatamente. |
| **Notas Técnicas** |   * Entidad y tabla rollback_snapshots en PostgreSQL BCNF vinculada a execution_queue.
  * Lógica de reversión orquestada en MitigationOrchestrator::execute_rollback.
  * Actualización del estado en tabla mitigation_audit con resultado 'ROLLED_BACK_BY_USER'. |
| **Tareas Relacionadas** |   * Diseñar generador de comandos inversos para RADIUS, BGP y DNS.
  * Implementar temporizador reactivo con animación fluida en Leptos WASM.
  * Pruebas de resiliencia ejecutando mitigaciones y rollbacks sucesivos en menos de 5 segundos. |
| **Valor de Negocio** | Elimina el temor a los falsos positivos en entornos de producción carrier, dotando al equipo del SOC de una red de seguridad infalible para actuar con velocidad sin comprometer los SLAs de Tigo. |
| **Puntos de Función (estimados)** | 49 |

---

### Historia de Usuario (HU-18)

| Elemento | Contenido |
| :--- | :--- |
| **Título** | Entorno Sandbox Aislado (VLAN 999) para Validación Previa de Reglas Sintéticas |
| **Descripción** | Como especialista en seguridad, quiero un entorno de laboratorio aislado (VLAN 999: 10.200.0.0/24) con inyección de tráfico sintético controlado, para ensayar y certificar las reglas de mitigación antes de aplicarlas en routers y switches de producción. |
| **Criterios de Aceptación** |   * Aislar estrictamente el segmento de red de la VLAN 999 impidiendo el enrutamiento hacia la LAN corporativa.
  * Permitir el lanzamiento de ataques simulados (DNS Tunneling, HTTP POST flood) desde contenedores de prueba (Kali Linux).
  * Evaluar la eficacia de las respuestas automatizadas sin riesgo para usuarios reales.
  * Reportar métricas de contención en tiempo real en la consola de administración. |
| **Notas Técnicas** |   * Scripts de aprovisionamiento de topología de red build_topology.sh y start_network.sh.
  * Emulación de dispositivos en entorno GNS3 / Linux Bridges / OpenWrt.
  * Generador de tráfico sintético calibrado con perfiles de carga de Tigo Business. |
| **Tareas Relacionadas** |   * Automatizar creación de interfaces virtuales e iptables de aislamiento en Linux.
  * Escribir script de ataque sintético con herramientas de ciberseguridad (lanzar_ataque.sh).
  * Documentar procedimiento de pruebas pre-producción en manual técnico. |
| **Valor de Negocio** | Garantiza que toda funcionalidad automatizada del SOC esté validada rigurosamente en un entorno seguro e idéntico a producción, reduciendo el riesgo operacional a cero. |
| **Puntos de Función (estimados)** | 45 |

---

### Historia de Usuario (HU-19)

| Elemento | Contenido |
| :--- | :--- |
| **Título** | Cuadro de Mando Ejecutivo Interactivo con KPIs de Detección y Conmutación Mocha/Latte |
| **Descripción** | Como usuario de la consola SOC, quiero un cuadro de mando ejecutivo interactivo con indicadores clave (MTTD, MTTR, latencia < 1 ms, volumen de tráfico) y la posibilidad de alternar entre los temas Catppuccin Mocha y Latte, para una supervisión ergonómica en salas de monitoreo o entornos iluminados. |
| **Criterios de Aceptación** |   * Presentar tarjetas KPI reactivas actualizadas en tiempo real sin recarga de página.
  * Ofrecer botón flotante para conmutar dinámicamente entre Catppuccin Mocha (modo oscuro) y Latte (modo claro).
  * Cumplir con estándares de contraste WCAG 2.1 AA en tipografías y bordes en ambos temas.
  * Cargar y renderizar la vista inicial en menos de 100 ms tras descarga del binario WebAssembly. |
| **Notas Técnicas** |   * Implementación en Rust Leptos con componentes SystemStatus.rs y app.rs.
  * Manejo reactivo de estado con create_signal y clases CSS dinámicas de Catppuccin.
  * Compilación de frontend con trunk build --release generando binario optimizado WASM. |
| **Tareas Relacionadas** |   * Diseñar hojas de estilo CSS con variables de color de Catppuccin Mocha y Latte.
  * Implementar cálculo reactivo de promedios de MTTD y throughput de red.
  * Realizar pruebas de rendimiento de pintado DOM y accesibilidad visual. |
| **Valor de Negocio** | Optimiza la concentración de los analistas en turnos prolongados de monitoreo 24/7, mejorando la ergonomía visual y facilitando la toma de decisiones ejecutivas en incidentes graves. |
| **Puntos de Función (estimados)** | 43 |

---

### Historia de Usuario (HU-20)

| Elemento | Contenido |
| :--- | :--- |
| **Título** | Despacho de Alertas y Notificaciones Push en Tiempo Real mediante Firebase Cloud Messaging |
| **Descripción** | Como analista de guardia del SOC, quiero recibir notificaciones push inmediatas en mi dispositivo móvil cuando se detecte una exfiltración crítica, para iniciar la investigación y respuesta incluso fuera de mi puesto físico. |
| **Criterios de Aceptación** |   * Enviar notificación push inmediata (latencia < 3 segundos) ante alertas con anomaly_score >= 0.85.
  * Incluir en la notificación el nombre del nodo comprometido, vector de exfiltración y score de criticidad.
  * Integrar credenciales seguras mediante cuenta de servicio oficial de Google Firebase (OAuth 2.0).
  * Registrar el estatus de entrega de la notificación en los registros del backend. |
| **Notas Técnicas** |   * Cliente FcmClient en tigo_soc_backend/src/infrastructure/fcm_client.rs.
  * Autenticación mediante Service Account JSON (service-account.json) y tokens JWT efímeros.
  * Conexión HTTP/2 asíncrona hacia los endpoints oficiales de FCM v1 API. |
| **Tareas Relacionadas** |   * Configurar proyecto en consola de Firebase y generar claves de servicio.
  * Implementar generador de JWT firmado con RSA SHA-256 para tokens de acceso de Google.
  * Diseñar manejador de recepción de notificaciones en aplicación cliente móvil PWA. |
| **Valor de Negocio** | Reduce el tiempo de respuesta ante incidentes fuera de horario de oficina (MTTR), asegurando que los especialistas de guardia intervengan de inmediato ante amenazas de alta severidad. |
| **Puntos de Función (estimados)** | 45 |

---

### Historia de Usuario (HU-21)

| Elemento | Contenido |
| :--- | :--- |
| **Título** | Pistas de Auditoría Inmutables del Sistema y Exportación de Expedientes Forenses |
| **Descripción** | Como auditor de seguridad, quiero consultar un registro inmutable de todas las acciones ejecutadas en el sistema (inicios de sesión, cambios de políticas, mitigaciones) y exportar reportes forenses, para respaldar auditorías normativas y judiciales. |
| **Criterios de Aceptación** |   * Registrar automáticamente cada acción en la tabla system_audit_logs con sello de tiempo, usuario y tabla afectada.
  * Proteger la integridad del registro impidiendo modificaciones o borrados (ON DELETE RESTRICT / SET NULL).
  * Proveer funcionalidad de filtrado multicriterio (por usuario, fecha, severidad y tipo de evento).
  * Permitir la exportación de expedientes forenses completos en formatos estructurados CSV y PDF. |
| **Notas Técnicas** |   * Tabla system_audit_logs en PostgreSQL BCNF.
  * Generador de reportes en backend Axum con serialización estructurada.
  * Sellado de eventos con función resumen hash SHA-256 para verificación de no manipulación. |
| **Tareas Relacionadas** |   * Configurar disparadores o interceptores de auditoría en repositorio SQLx.
  * Implementar vista de tabla de auditoría con paginación reactiva en el frontend.
  * Pruebas de consistencia e integridad de pistas de auditoría ante transacciones fallidas. |
| **Valor de Negocio** | Garantiza la trazabilidad legal y el cumplimiento de las normativas de la ATT y estándares internacionales de auditoría informática, blindando judicialmente a Tigo Business ante litigios. |
| **Puntos de Función (estimados)** | 44 |

---

### Historia de Usuario (HU-22)

| Elemento | Contenido |
| :--- | :--- |
| **Título** | Consola Web Reactiva Multiplataforma (Desktop / Tablet / Móvil PWA) en Rust WebAssembly |
| **Descripción** | Como operador del SOC, quiero interactuar con una interfaz fluida y adaptativa que funcione de manera óptima en computadoras de escritorio (1440x900), tablets (1024x768) y teléfonos móviles (390x844), para operar eficientemente desde cualquier dispositivo corporativo. |
| **Criterios de Aceptación** |   * Adaptar la disposición de menús, gráficos y tablas de acuerdo con la resolución de pantalla del dispositivo.
  * Compilar la interfaz íntegramente a WebAssembly (Leptos) sin depender de frameworks pesados de Node.js en ejecución.
  * Proveer capacidades PWA (Progressive Web App) con manifest y soporte para instalación en pantalla de inicio.
  * Mantener la interacción reactiva fluida sin bloqueos de interfaz durante la recepción de telemetría masiva. |
| **Notas Técnicas** |   * Arquitectura SPA reactiva en tigo_soc_frontend/src con componentes modulares.
  * Flexbox y Grid CSS adaptativo con media queries ergonómicas.
  * Alojamiento estático servido eficientemente mediante Nginx Reverse Proxy. |
| **Tareas Relacionadas** |   * Optimizar renderizado de componentes para visualización en pantallas táctiles móviles.
  * Configurar manifiesto PWA e iconos corporativos de Tigo Business.
  * Realizar pruebas de compatibilidad en Google Chrome, Mozilla Firefox, Safari y Microsoft Edge. |
| **Valor de Negocio** | Otorga máxima versatilidad y movilidad al equipo de seguridad, permitiendo la supervisión y contención de amenazas tanto en el búnker central del SOC como en desplazamientos de campo. |
| **Puntos de Función (estimados)** | 48 |

---
