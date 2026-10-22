# Diccionario de Datos Formal de la Base de Datos (PostgreSQL 15)
**Proyecto:** Sistema Web Aplicando LightGBM para Detección de Exfiltración de Datos  
**Caso de Estudio:** Tigo Business Bolivia (AS27839)  
**Autor:** Jorge Esteban Medrano Chacolla | **Forma Normal:** Tercera Forma Normal (BCNF) | **Total Tablas:** 21  

---

## 1. Estructura y Diccionario Detallado por Sprint

### Sprint 1: Gobernanza de Seguridad Zero-Trust, Autenticación y RBAC

#### Tabla: `roles` (BCNF (Boyce-Codd))
**Propósito:** Catálogo de perfiles y roles del sistema para control de acceso RBAC.

| Columna | Tipo de Dato | Longitud | Llave | Nulidad | Defecto | Descripción / Regla de Negocio |
| :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| **`role_id`** | `SERIAL` | 4 bytes | `PK` | NOT NULL | `Autoincremental` | Identificador único secuencial del rol en el sistema. |
| **`role_name`** | `VARCHAR(50)` | 50 caracteres | `UNIQUE` | NOT NULL | `Ninguno` | Nombre único del rol asignado (ej. 'SOC_L1', 'SOC_L2', 'SOC_ADMIN', 'AUDITOR'). |
| **`description`** | `TEXT` | Variable | `Ninguna` | NULL | `Ninguno` | Descripción funcional de los privilegios y responsabilidades del rol. |

---

#### Tabla: `permissions` (BCNF (Boyce-Codd))
**Propósito:** Catálogo de privilegios y permisos atómicos para control granular de endpoints y funciones críticas.

| Columna | Tipo de Dato | Longitud | Llave | Nulidad | Defecto | Descripción / Regla de Negocio |
| :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| **`permission_id`** | `SERIAL` | 4 bytes | `PK` | NOT NULL | `Autoincremental` | Identificador único secuencial del permiso atómico. |
| **`permission_name`** | `VARCHAR(50)` | 50 caracteres | `UNIQUE` | NOT NULL | `Ninguno` | Nombre técnico del permiso (ej. 'USER_MANAGE', 'MITIGATE_EXECUTE', 'ROLLBACK_EXECUTE'). |
| **`description`** | `TEXT` | Variable | `Ninguna` | NULL | `Ninguno` | Alcance y restricciones asociadas al permiso técnico. |

---

#### Tabla: `role_permissions` (BCNF (Boyce-Codd))
**Propósito:** Tabla asociativa para modelar la relación muchos a muchos (N:M) entre roles y permisos.

| Columna | Tipo de Dato | Longitud | Llave | Nulidad | Defecto | Descripción / Regla de Negocio |
| :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| **`role_id`** | `INT` | 4 bytes | `PK, FK` | NOT NULL | `Ninguno` | Referencia al rol en roles(role_id) con eliminación en cascada (ON DELETE CASCADE). |
| **`permission_id`** | `INT` | 4 bytes | `PK, FK` | NOT NULL | `Ninguno` | Referencia al permiso en permissions(permission_id) con eliminación en cascada. |

---

#### Tabla: `sys_users` (BCNF (Boyce-Codd))
**Propósito:** Registro de operadores y analistas autorizados en el SOC con credenciales Argon2id.

| Columna | Tipo de Dato | Longitud | Llave | Nulidad | Defecto | Descripción / Regla de Negocio |
| :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| **`user_id`** | `SERIAL` | 4 bytes | `PK` | NOT NULL | `Autoincremental` | Identificador único secuencial del usuario del sistema. |
| **`username`** | `VARCHAR(50)` | 50 caracteres | `UNIQUE` | NOT NULL | `Ninguno` | Nombre de usuario único para inicio de sesión en el SOC. |
| **`password_hash`** | `VARCHAR(255)` | 255 caracteres | `Ninguna` | NOT NULL | `Ninguno` | Hash criptográfico de la contraseña generado mediante algoritmo Argon2id. |
| **`role_id`** | `INT` | 4 bytes | `FK` | NULL | `Ninguno` | Referencia al rol RBAC en roles(role_id) con restricción restrictiva (ON DELETE RESTRICT). |
| **`created_at`** | `TIMESTAMPTZ` | 8 bytes | `Ninguna` | NOT NULL | `CURRENT_TIMESTAMP` | Fecha y hora con zona horaria de creación del usuario en el sistema. |

---

#### Tabla: `system_parameters` (BCNF (Boyce-Codd))
**Propósito:** Parámetros globales de configuración del sistema y políticas de gobernanza operativa.

| Columna | Tipo de Dato | Longitud | Llave | Nulidad | Defecto | Descripción / Regla de Negocio |
| :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| **`param_id`** | `SERIAL` | 4 bytes | `PK` | NOT NULL | `Autoincremental` | Identificador único secuencial del parámetro del sistema. |
| **`param_key`** | `VARCHAR(100)` | 100 caracteres | `UNIQUE` | NOT NULL | `Ninguno` | Clave única del parámetro (ej. 'critical_reauth_timeout_secs', 'password_rotation_days'). |
| **`param_value`** | `TEXT` | Variable | `Ninguna` | NOT NULL | `Ninguno` | Valor textual o JSON del parámetro configurable. |
| **`description`** | `TEXT` | Variable | `Ninguna` | NULL | `Ninguno` | Explicación técnica del propósito y efecto del parámetro. |
| **`last_updated_by`** | `INT` | 4 bytes | `FK` | NULL | `Ninguno` | Referencia al usuario en sys_users(user_id) que actualizó el parámetro (ON DELETE SET NULL). |
| **`updated_at`** | `TIMESTAMPTZ` | 8 bytes | `Ninguna` | NOT NULL | `CURRENT_TIMESTAMP` | Sello de tiempo de la última modificación del parámetro. |

---

#### Tabla: `user_sessions` (BCNF (Boyce-Codd))
**Propósito:** Gestión activa de sesiones concurrentes, tokens opacos y control de re-autenticación.

| Columna | Tipo de Dato | Longitud | Llave | Nulidad | Defecto | Descripción / Regla de Negocio |
| :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| **`session_id`** | `VARCHAR(64)` | 64 caracteres | `PK` | NOT NULL | `Ninguno` | Token de sesión aleatorio seguro de 256 bits codificado en hexadecimal. |
| **`user_id`** | `INT` | 4 bytes | `FK` | NOT NULL | `Ninguno` | Referencia al usuario propietario de la sesión en sys_users(user_id) (ON DELETE CASCADE). |
| **`created_at`** | `TIMESTAMPTZ` | 8 bytes | `Ninguna` | NOT NULL | `CURRENT_TIMESTAMP` | Fecha y hora de inicio de la sesión. |
| **`expires_at`** | `TIMESTAMPTZ` | 8 bytes | `Ninguna` | NOT NULL | `Ninguno` | Fecha y hora de expiración automática de la sesión. |
| **`last_activity`** | `TIMESTAMPTZ` | 8 bytes | `Ninguna` | NOT NULL | `CURRENT_TIMESTAMP` | Sello de tiempo de la última interacción del usuario. |
| **`last_reauth_at`** | `TIMESTAMPTZ` | 8 bytes | `Ninguna` | NULL | `Ninguno` | Sello de tiempo de la última re-autenticación 2FA para operaciones críticas. |
| **`is_revoked`** | `BOOLEAN` | 1 byte | `Ninguna` | NOT NULL | `FALSE` | Bandera de invalidación manual o forzada de la sesión. |
| **`ip_address`** | `VARCHAR(45)` | 45 caracteres | `Ninguna` | NULL | `Ninguno` | Dirección IP IPv4 o IPv6 desde la cual se originó la sesión. |
| **`user_agent`** | `TEXT` | Variable | `Ninguna` | NULL | `Ninguno` | Cadena descriptiva del navegador o cliente web conectado. |

---

### Sprint 2: Motor Analítico de Machine Learning LightGBM y CRISP-DM

#### Tabla: `threat_types` (BCNF (Boyce-Codd))
**Propósito:** Catálogo de vectores de ciberamenaza clasificados por el modelo LightGBM.

| Columna | Tipo de Dato | Longitud | Llave | Nulidad | Defecto | Descripción / Regla de Negocio |
| :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| **`threat_id`** | `SERIAL` | 4 bytes | `PK` | NOT NULL | `Autoincremental` | Identificador único secuencial del tipo de amenaza. |
| **`threat_name`** | `VARCHAR(50)` | 50 caracteres | `UNIQUE` | NOT NULL | `Ninguno` | Nombre del vector de ataque (ej. 'DNS_TUNNELING', 'HTTP_POST_EXFIL', 'SLOW_AND_LOW'). |
| **`severity_level`** | `INT` | 4 bytes | `CHECK` | NOT NULL | `Ninguno` | Nivel de severidad técnica del incidente entre 1 (Baja) y 5 (Crítica). |

---

#### Tabla: `feature_store` (BCNF (Boyce-Codd))
**Propósito:** Almacén estructurado de vectores de 23 características procesadas por Polars SIMD para inferencia y reentrenamiento.

| Columna | Tipo de Dato | Longitud | Llave | Nulidad | Defecto | Descripción / Regla de Negocio |
| :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| **`feature_id`** | `BIGSERIAL` | 8 bytes | `PK` | NOT NULL | `Autoincremental` | Identificador único secuencial del vector de características. |
| **`log_id`** | `BIGINT` | 8 bytes | `UNIQUE, FK` | NULL | `Ninguno` | Referencia unívoca al log de tráfico original en network_logs(log_id) (ON DELETE CASCADE). |
| **`feature_vector`** | `JSONB` | Variable | `Ninguna` | NOT NULL | `Ninguno` | Objeto JSON binario con las 23 características de red normalizadas. |
| **`processed_at`** | `TIMESTAMPTZ` | 8 bytes | `Ninguna` | NOT NULL | `CURRENT_TIMESTAMP` | Sello de tiempo de extracción y vectorización en memoria. |

---

#### Tabla: `security_alerts` (BCNF (Boyce-Codd))
**Propósito:** Alertas generadas en tiempo sub-milisegundo ante detección de exfiltración de datos.

| Columna | Tipo de Dato | Longitud | Llave | Nulidad | Defecto | Descripción / Regla de Negocio |
| :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| **`alert_id`** | `BIGSERIAL` | 8 bytes | `PK` | NOT NULL | `Autoincremental` | Identificador único secuencial de la alerta de ciberseguridad. |
| **`feature_id`** | `BIGINT` | 8 bytes | `UNIQUE, FK` | NULL | `Ninguno` | Referencia al vector causante en feature_store(feature_id) (ON DELETE CASCADE). |
| **`threat_id`** | `INT` | 4 bytes | `FK` | NULL | `Ninguno` | Referencia al tipo de amenaza en threat_types(threat_id) (ON DELETE RESTRICT). |
| **`status_id`** | `INT` | 4 bytes | `FK` | NULL | `Ninguno` | Referencia al estado de atención en alert_statuses(status_id) (ON DELETE RESTRICT). |
| **`anomaly_score`** | `FLOAT` | 8 bytes | `CHECK` | NOT NULL | `Ninguno` | Probabilidad de exfiltración generada por LightGBM (rango continuo 0.0 a 1.0). |
| **`detected_at`** | `TIMESTAMPTZ` | 8 bytes | `Ninguna` | NOT NULL | `CURRENT_TIMESTAMP` | Sello de tiempo preciso de la detección en la sonda de red. |

---

### Sprint 3: Telemetría de Red de Alta Velocidad, Descubrimiento y Topología AS27839

#### Tabla: `device_types` (BCNF (Boyce-Codd))
**Propósito:** Catálogo de tipos de dispositivos de telecomunicaciones en la red carrier.

| Columna | Tipo de Dato | Longitud | Llave | Nulidad | Defecto | Descripción / Regla de Negocio |
| :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| **`type_id`** | `SERIAL` | 4 bytes | `PK` | NOT NULL | `Autoincremental` | Identificador único secuencial del tipo de dispositivo. |
| **`type_name`** | `VARCHAR(50)` | 50 caracteres | `UNIQUE` | NOT NULL | `Ninguno` | Nombre del tipo de equipo (ej. 'BGP_ROUTER', 'CORE_SWITCH', 'FIREWALL', 'SERVER', 'HOST'). |

---

#### Tabla: `mgmt_protocols` (BCNF (Boyce-Codd))
**Propósito:** Catálogo de protocolos de red para gestión y mitigación automatizada.

| Columna | Tipo de Dato | Longitud | Llave | Nulidad | Defecto | Descripción / Regla de Negocio |
| :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| **`protocol_id`** | `SERIAL` | 4 bytes | `PK` | NOT NULL | `Autoincremental` | Identificador único secuencial del protocolo de gestión. |
| **`protocol_name`** | `VARCHAR(20)` | 20 caracteres | `UNIQUE` | NOT NULL | `Ninguno` | Nombre del protocolo de enlace (ej. 'SNMP_V3', 'SSH', 'RESTCONF', 'RADIUS'). |

---

#### Tabla: `network_nodes` (BCNF (Boyce-Codd))
**Propósito:** Inventario de activos de red descubiertos pasivamente en la topología AS27839 de Tigo Business.

| Columna | Tipo de Dato | Longitud | Llave | Nulidad | Defecto | Descripción / Regla de Negocio |
| :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| **`node_id`** | `SERIAL` | 4 bytes | `PK` | NOT NULL | `Autoincremental` | Identificador único secuencial del activo de red. |
| **`hostname`** | `VARCHAR(100)` | 100 caracteres | `UNIQUE` | NOT NULL | `Ninguno` | Nombre o identificador FQDN del dispositivo en la topología. |
| **`ip_address`** | `VARCHAR(45)` | 45 caracteres | `UNIQUE` | NOT NULL | `Ninguno` | Dirección IP IPv4 o IPv6 asignada al nodo de red. |
| **`type_id`** | `INT` | 4 bytes | `FK` | NULL | `Ninguno` | Referencia al tipo de equipo en device_types(type_id) (ON DELETE RESTRICT). |
| **`protocol_id`** | `INT` | 4 bytes | `FK` | NULL | `Ninguno` | Referencia al protocolo de gestión en mgmt_protocols(protocol_id) (ON DELETE RESTRICT). |
| **`mac_address`** | `VARCHAR(50)` | 50 caracteres | `Ninguna` | NULL | `Ninguno` | Dirección física MAC de la interfaz de red capturada. |
| **`vendor`** | `VARCHAR(50)` | 50 caracteres | `Ninguna` | NULL | `Ninguno` | Fabricante del equipo resuelto mediante consulta OUI (ej. Cisco, MikroTik, Fortinet). |

---

#### Tabla: `network_traffic_metrics` (BCNF (Boyce-Codd))
**Propósito:** Métricas agregadas periódicas por ventana de 1 segundo para visualización masiva sin saturar disco.

| Columna | Tipo de Dato | Longitud | Llave | Nulidad | Defecto | Descripción / Regla de Negocio |
| :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| **`metric_id`** | `BIGSERIAL` | 8 bytes | `PK` | NOT NULL | `Autoincremental` | Identificador único secuencial de la métrica de tráfico agregada. |
| **`node_id`** | `INT` | 4 bytes | `FK, UNIQUE` | NOT NULL | `Ninguno` | Referencia al nodo en network_nodes(node_id) (ON DELETE CASCADE). |
| **`window_start`** | `TIMESTAMPTZ` | 8 bytes | `UNIQUE` | NOT NULL | `Ninguno` | Inicio del intervalo de tiempo de agregación (clave de unicidad compuesta). |
| **`window_end`** | `TIMESTAMPTZ` | 8 bytes | `Ninguna` | NOT NULL | `Ninguno` | Fin del intervalo de tiempo de agregación. |
| **`total_packets`** | `BIGINT` | 8 bytes | `Ninguna` | NOT NULL | `0` | Cantidad total de paquetes capturados en la ventana de 1 segundo. |
| **`total_bytes`** | `BIGINT` | 8 bytes | `Ninguna` | NOT NULL | `0` | Volumen total en bytes transferidos en la ventana de 1 segundo. |
| **`created_at`** | `TIMESTAMPTZ` | 8 bytes | `Ninguna` | NOT NULL | `CURRENT_TIMESTAMP` | Sello de tiempo de persistencia en base de datos. |

---

#### Tabla: `network_logs` (BCNF (Boyce-Codd))
**Propósito:** Registros forenses detallados de tramas de red aisladas exclusivamente para eventos anómalos.

| Columna | Tipo de Dato | Longitud | Llave | Nulidad | Defecto | Descripción / Regla de Negocio |
| :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| **`log_id`** | `BIGSERIAL` | 8 bytes | `PK` | NOT NULL | `Autoincremental` | Identificador único secuencial del registro forense. |
| **`node_id`** | `INT` | 4 bytes | `FK` | NOT NULL | `Ninguno` | Referencia al nodo de red en network_nodes(node_id) (ON DELETE CASCADE). |
| **`source_ip`** | `VARCHAR(45)` | 45 caracteres | `Ninguna` | NOT NULL | `Ninguno` | Dirección IP de origen del flujo de tráfico inspeccionado. |
| **`destination_ip`** | `VARCHAR(45)` | 45 caracteres | `Ninguna` | NOT NULL | `Ninguno` | Dirección IP de destino sospechosa de exfiltración. |
| **`protocol`** | `VARCHAR(10)` | 10 caracteres | `Ninguna` | NOT NULL | `Ninguno` | Protocolo de red inspeccionado (ej. 'TCP', 'UDP', 'ICMP', 'DNS'). |
| **`packet_size`** | `INT` | 4 bytes | `Ninguna` | NOT NULL | `Ninguno` | Tamaño total en bytes de la trama capturada. |
| **`flags`** | `VARCHAR(20)` | 20 caracteres | `Ninguna` | NULL | `Ninguno` | Banderas de control TCP asociadas a la sesión (ej. 'SYN-ACK', 'RST'). |
| **`timestamp`** | `TIMESTAMPTZ` | 8 bytes | `Ninguna` | NOT NULL | `CURRENT_TIMESTAMP` | Sello de tiempo de precisión microsegundo de la captura por la sonda eBPF. |

---

### Sprint 4: Orquestación de Mitigación Multivector y Resiliencia con Rollback Inmediato

#### Tabla: `alert_statuses` (BCNF (Boyce-Codd))
**Propósito:** Catálogo de estados del ciclo de vida de una alerta de seguridad.

| Columna | Tipo de Dato | Longitud | Llave | Nulidad | Defecto | Descripción / Regla de Negocio |
| :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| **`status_id`** | `SERIAL` | 4 bytes | `PK` | NOT NULL | `Autoincremental` | Identificador único secuencial del estado de la alerta. |
| **`status_name`** | `VARCHAR(20)` | 20 caracteres | `UNIQUE` | NOT NULL | `Ninguno` | Nombre del estado operativo (ej. 'NEW', 'IN_ANALYSIS', 'MITIGATING', 'RESOLVED', 'FALSE_POSITIVE'). |

---

#### Tabla: `task_statuses` (BCNF (Boyce-Codd))
**Propósito:** Catálogo de estados de las órdenes de mitigación encoladas en el despachador.

| Columna | Tipo de Dato | Longitud | Llave | Nulidad | Defecto | Descripción / Regla de Negocio |
| :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| **`status_id`** | `SERIAL` | 4 bytes | `PK` | NOT NULL | `Autoincremental` | Identificador único secuencial del estado de la tarea. |
| **`status_name`** | `VARCHAR(20)` | 20 caracteres | `UNIQUE` | NOT NULL | `Ninguno` | Nombre del estado de ejecución (ej. 'QUEUED', 'IN_PROGRESS', 'COMPLETED', 'FAILED', 'ROLLED_BACK'). |

---

#### Tabla: `mitigation_actions` (BCNF (Boyce-Codd))
**Propósito:** Catálogo agnóstico de acciones de contención clasificadas por capa del modelo OSI.

| Columna | Tipo de Dato | Longitud | Llave | Nulidad | Defecto | Descripción / Regla de Negocio |
| :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| **`action_id`** | `SERIAL` | 4 bytes | `PK` | NOT NULL | `Autoincremental` | Identificador único secuencial de la acción de mitigación. |
| **`action_name`** | `VARCHAR(100)` | 100 caracteres | `UNIQUE` | NOT NULL | `Ninguno` | Nombre descriptivo de la acción (ej. 'RADIUS_COA_DISCONNECT', 'BGP_RTBH_NULL0', 'DNS_SINKHOLE'). |
| **`layer_target`** | `VARCHAR(10)` | 10 caracteres | `CHECK` | NOT NULL | `Ninguno` | Capa objetivo del modelo OSI sujeta a contención ('L2', 'L3', 'L4', 'L7'). |
| **`description`** | `TEXT` | Variable | `Ninguna` | NULL | `Ninguno` | Descripción técnica de los mecanismos de red invocados. |

---

#### Tabla: `execution_queue` (BCNF (Boyce-Codd))
**Propósito:** Cola de tareas de mitigación asíncronas despachadas hacia conmutadores y enrutadores.

| Columna | Tipo de Dato | Longitud | Llave | Nulidad | Defecto | Descripción / Regla de Negocio |
| :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| **`task_id`** | `BIGSERIAL` | 8 bytes | `PK` | NOT NULL | `Autoincremental` | Identificador único secuencial de la tarea en cola. |
| **`alert_id`** | `BIGINT` | 8 bytes | `FK` | NOT NULL | `Ninguno` | Referencia a la alerta generadora en security_alerts(alert_id) (ON DELETE CASCADE). |
| **`action_id`** | `INT` | 4 bytes | `FK` | NOT NULL | `Ninguno` | Referencia a la acción de mitigación en mitigation_actions(action_id) (ON DELETE RESTRICT). |
| **`node_id`** | `INT` | 4 bytes | `FK` | NOT NULL | `Ninguno` | Referencia al nodo objetivo en network_nodes(node_id) (ON DELETE RESTRICT). |
| **`status_id`** | `INT` | 4 bytes | `FK` | NOT NULL | `Ninguno` | Referencia al estado de la tarea en task_statuses(status_id) (ON DELETE RESTRICT). |
| **`queued_at`** | `TIMESTAMPTZ` | 8 bytes | `Ninguna` | NOT NULL | `CURRENT_TIMESTAMP` | Momento exacto en que la orden fue encolada en el orquestador. |
| **`executed_at`** | `TIMESTAMPTZ` | 8 bytes | `Ninguna` | NULL | `Ninguno` | Momento exacto en que el equipo de red confirmó la ejecución. |

---

#### Tabla: `rollback_snapshots` (BCNF (Boyce-Codd))
**Propósito:** Instantáneas JSON de pre-estado y cargas útiles de reversión para resiliencia ante falsos positivos.

| Columna | Tipo de Dato | Longitud | Llave | Nulidad | Defecto | Descripción / Regla de Negocio |
| :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| **`snapshot_id`** | `BIGSERIAL` | 8 bytes | `PK` | NOT NULL | `Autoincremental` | Identificador único secuencial de la instantánea de reversión. |
| **`task_id`** | `BIGINT` | 8 bytes | `UNIQUE, FK` | NOT NULL | `Ninguno` | Referencia unívoca a la tarea ejecutada en execution_queue(task_id) (ON DELETE CASCADE). |
| **`pre_state_json`** | `JSONB` | Variable | `Ninguna` | NOT NULL | `Ninguno` | Estado completo de configuración del equipo de red previo a la mitigación. |
| **`rollback_payload`** | `JSONB` | Variable | `Ninguna` | NOT NULL | `Ninguno` | Comandos estructurados necesarios para restaurar la configuración original. |
| **`created_at`** | `TIMESTAMPTZ` | 8 bytes | `Ninguna` | NOT NULL | `CURRENT_TIMESTAMP` | Sello de tiempo de generación previa de la instantánea. |

---

#### Tabla: `mitigation_audit` (BCNF (Boyce-Codd))
**Propósito:** Registro inmutable de trazabilidad de respuestas ante incidentes ejecutadas o revertidas.

| Columna | Tipo de Dato | Longitud | Llave | Nulidad | Defecto | Descripción / Regla de Negocio |
| :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| **`audit_id`** | `BIGSERIAL` | 8 bytes | `PK` | NOT NULL | `Autoincremental` | Identificador único secuencial del registro de auditoría de mitigación. |
| **`task_id`** | `BIGINT` | 8 bytes | `UNIQUE, FK` | NULL | `Ninguno` | Referencia a la tarea en execution_queue(task_id) (ON DELETE SET NULL). |
| **`executed_by_user`** | `INT` | 4 bytes | `FK` | NULL | `Ninguno` | Operador que autorizó la acción en sys_users(user_id) (NULL si fue autónoma). |
| **`executed_at`** | `TIMESTAMPTZ` | 8 bytes | `Ninguna` | NOT NULL | `CURRENT_TIMESTAMP` | Fecha y hora exacta de ejecución o reversión. |
| **`result_status`** | `VARCHAR(50)` | 50 caracteres | `Ninguna` | NOT NULL | `Ninguno` | Estatus del resultado ('APPLIED_SUCCESS', 'FAILED', 'ROLLED_BACK_BY_USER'). |

---

### Sprint 5: Consola Web Unificada Leptos WASM y Auditoría Integral

#### Tabla: `system_audit_logs` (BCNF (Boyce-Codd))
**Propósito:** Pistas de auditoría general inmutables del sistema para cumplimiento normativo e investigaciones forenses.

| Columna | Tipo de Dato | Longitud | Llave | Nulidad | Defecto | Descripción / Regla de Negocio |
| :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| **`sys_audit_id`** | `BIGSERIAL` | 8 bytes | `PK` | NOT NULL | `Autoincremental` | Identificador único secuencial de la pista de auditoría general. |
| **`user_id`** | `INT` | 4 bytes | `FK` | NULL | `Ninguno` | Referencia al usuario ejecutor en sys_users(user_id) (ON DELETE SET NULL). |
| **`action`** | `VARCHAR(100)` | 100 caracteres | `Ninguna` | NOT NULL | `Ninguno` | Acción registrada (ej. 'USER_LOGIN', 'CHANGE_PASSWORD', 'PARAMETER_UPDATE', 'EXPORT_REPORT'). |
| **`target_table`** | `VARCHAR(50)` | 50 caracteres | `Ninguna` | NOT NULL | `Ninguno` | Nombre de la entidad o tabla afectada por la transacción. |
| **`target_id`** | `BIGINT` | 8 bytes | `Ninguna` | NULL | `Ninguno` | Identificador numérico del registro afectado. |
| **`timestamp`** | `TIMESTAMPTZ` | 8 bytes | `Ninguna` | NOT NULL | `CURRENT_TIMESTAMP` | Sello de tiempo inmutable con zona horaria de la transacción. |

---

## 2. Resumen Catálogo Consolidado de Tablas

| No. | Tabla | Sprint | Campos | Clave Primaria | Forma Normal | Propósito en el Sistema |
| :---: | :--- | :---: | :---: | :---: | :---: | :--- |
| 1 | **`roles`** | Sprint 1 | 3 | `role_id` | BCNF (Boyce-Codd) | Catálogo de perfiles y roles del sistema para control de acceso RBAC. |
| 2 | **`permissions`** | Sprint 1 | 3 | `permission_id` | BCNF (Boyce-Codd) | Catálogo de privilegios y permisos atómicos para control granular de endpoints y funciones críticas. |
| 3 | **`role_permissions`** | Sprint 1 | 2 | `role_id, permission_id` | BCNF (Boyce-Codd) | Tabla asociativa para modelar la relación muchos a muchos (N:M) entre roles y permisos. |
| 4 | **`sys_users`** | Sprint 1 | 5 | `user_id` | BCNF (Boyce-Codd) | Registro de operadores y analistas autorizados en el SOC con credenciales Argon2id. |
| 5 | **`system_parameters`** | Sprint 1 | 6 | `param_id` | BCNF (Boyce-Codd) | Parámetros globales de configuración del sistema y políticas de gobernanza operativa. |
| 6 | **`user_sessions`** | Sprint 1 | 9 | `session_id` | BCNF (Boyce-Codd) | Gestión activa de sesiones concurrentes, tokens opacos y control de re-autenticación. |
| 7 | **`threat_types`** | Sprint 2 | 3 | `threat_id` | BCNF (Boyce-Codd) | Catálogo de vectores de ciberamenaza clasificados por el modelo LightGBM. |
| 8 | **`feature_store`** | Sprint 2 | 4 | `feature_id` | BCNF (Boyce-Codd) | Almacén estructurado de vectores de 23 características procesadas por Polars SIMD para inferencia y reentrenamiento. |
| 9 | **`security_alerts`** | Sprint 2 | 6 | `alert_id` | BCNF (Boyce-Codd) | Alertas generadas en tiempo sub-milisegundo ante detección de exfiltración de datos. |
| 10 | **`device_types`** | Sprint 3 | 2 | `type_id` | BCNF (Boyce-Codd) | Catálogo de tipos de dispositivos de telecomunicaciones en la red carrier. |
| 11 | **`mgmt_protocols`** | Sprint 3 | 2 | `protocol_id` | BCNF (Boyce-Codd) | Catálogo de protocolos de red para gestión y mitigación automatizada. |
| 12 | **`network_nodes`** | Sprint 3 | 7 | `node_id` | BCNF (Boyce-Codd) | Inventario de activos de red descubiertos pasivamente en la topología AS27839 de Tigo Business. |
| 13 | **`network_traffic_metrics`** | Sprint 3 | 7 | `metric_id` | BCNF (Boyce-Codd) | Métricas agregadas periódicas por ventana de 1 segundo para visualización masiva sin saturar disco. |
| 14 | **`network_logs`** | Sprint 3 | 8 | `log_id` | BCNF (Boyce-Codd) | Registros forenses detallados de tramas de red aisladas exclusivamente para eventos anómalos. |
| 15 | **`alert_statuses`** | Sprint 4 | 2 | `status_id` | BCNF (Boyce-Codd) | Catálogo de estados del ciclo de vida de una alerta de seguridad. |
| 16 | **`task_statuses`** | Sprint 4 | 2 | `status_id` | BCNF (Boyce-Codd) | Catálogo de estados de las órdenes de mitigación encoladas en el despachador. |
| 17 | **`mitigation_actions`** | Sprint 4 | 4 | `action_id` | BCNF (Boyce-Codd) | Catálogo agnóstico de acciones de contención clasificadas por capa del modelo OSI. |
| 18 | **`execution_queue`** | Sprint 4 | 7 | `task_id` | BCNF (Boyce-Codd) | Cola de tareas de mitigación asíncronas despachadas hacia conmutadores y enrutadores. |
| 19 | **`rollback_snapshots`** | Sprint 4 | 5 | `snapshot_id` | BCNF (Boyce-Codd) | Instantáneas JSON de pre-estado y cargas útiles de reversión para resiliencia ante falsos positivos. |
| 20 | **`mitigation_audit`** | Sprint 4 | 5 | `audit_id` | BCNF (Boyce-Codd) | Registro inmutable de trazabilidad de respuestas ante incidentes ejecutadas o revertidas. |
| 21 | **`system_audit_logs`** | Sprint 5 | 6 | `sys_audit_id` | BCNF (Boyce-Codd) | Pistas de auditoría general inmutables del sistema para cumplimiento normativo e investigaciones forenses. |
| **TOTAL** | **21 Tablas Relacionales** | - | **98 campos** | - | **BCNF** | **Persistencia Integral del Sistema SOC** |