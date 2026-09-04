-- =========================================================================
-- 1. CATÁLOGOS BASE
-- =========================================================================

CREATE TABLE device_types (
    type_id SERIAL PRIMARY KEY,
    type_name VARCHAR(50) UNIQUE NOT NULL
);

CREATE TABLE mgmt_protocols (
    protocol_id SERIAL PRIMARY KEY,
    protocol_name VARCHAR(20) UNIQUE NOT NULL
);

CREATE TABLE threat_types (
    threat_id SERIAL PRIMARY KEY,
    threat_name VARCHAR(50) UNIQUE NOT NULL,
    severity_level INT NOT NULL CHECK (severity_level BETWEEN 1 AND 5)
);

CREATE TABLE alert_statuses (
    status_id SERIAL PRIMARY KEY,
    status_name VARCHAR(20) UNIQUE NOT NULL
);

CREATE TABLE task_statuses (
    status_id SERIAL PRIMARY KEY,
    status_name VARCHAR(20) UNIQUE NOT NULL 
);

-- =========================================================================
-- 2. RBAC Y SEGURIDAD DEL SISTEMA
-- =========================================================================

CREATE TABLE roles (
    role_id SERIAL PRIMARY KEY,
    role_name VARCHAR(50) UNIQUE NOT NULL,
    description TEXT
);

CREATE TABLE permissions (
    permission_id SERIAL PRIMARY KEY,
    permission_name VARCHAR(50) UNIQUE NOT NULL,
    description TEXT
);

CREATE TABLE role_permissions (
    role_id INT REFERENCES roles(role_id) ON DELETE CASCADE,
    permission_id INT REFERENCES permissions(permission_id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

CREATE TABLE sys_users (
    user_id SERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role_id INT REFERENCES roles(role_id) ON DELETE RESTRICT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE system_parameters (
    param_id SERIAL PRIMARY KEY,
    param_key VARCHAR(100) UNIQUE NOT NULL,
    param_value TEXT NOT NULL,
    description TEXT,
    last_updated_by INT REFERENCES sys_users(user_id) ON DELETE SET NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- =========================================================================
-- 3. INVENTARIO Y TELEMETRÍA (DIVIDIDA: AGREGADA VS FORENSE)
-- =========================================================================

CREATE TABLE network_nodes (
    node_id SERIAL PRIMARY KEY,
    hostname VARCHAR(100) UNIQUE NOT NULL,
    ip_address VARCHAR(45) UNIQUE NOT NULL,
    type_id INT REFERENCES device_types(type_id) ON DELETE RESTRICT,
    protocol_id INT REFERENCES mgmt_protocols(protocol_id) ON DELETE RESTRICT
);

-- NUEVA TABLA: Almacena métricas agregadas por intervalos (ej. cada minuto/10 mins)
-- Resuelve el cálculo de porcentajes sin saturar la base con tráfico normal.
CREATE TABLE network_traffic_metrics (
    metric_id BIGSERIAL PRIMARY KEY,
    node_id INT REFERENCES network_nodes(node_id) ON DELETE CASCADE,
    window_start TIMESTAMP WITH TIME ZONE NOT NULL,
    window_end TIMESTAMP WITH TIME ZONE NOT NULL,
    total_packets BIGINT NOT NULL DEFAULT 0,
    total_bytes BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (node_id, window_start)
);

-- FORENSE: Solo guarda paquetes maliciosos o sospechosos (ataques aislados)
CREATE TABLE network_logs (
    log_id BIGSERIAL PRIMARY KEY,
    node_id INT REFERENCES network_nodes(node_id) ON DELETE CASCADE,
    source_ip VARCHAR(45) NOT NULL,
    destination_ip VARCHAR(45) NOT NULL,
    protocol VARCHAR(10) NOT NULL,
    packet_size INT NOT NULL,
    flags VARCHAR(20),
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE feature_store (
    feature_id BIGSERIAL PRIMARY KEY,
    log_id BIGINT UNIQUE REFERENCES network_logs(log_id) ON DELETE CASCADE,
    feature_vector JSONB NOT NULL,
    processed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- =========================================================================
-- 4. ORQUESTACIÓN Y MITIGACIÓN
-- =========================================================================

CREATE TABLE mitigation_actions (
    action_id SERIAL PRIMARY KEY,
    action_name VARCHAR(100) UNIQUE NOT NULL,
    layer_target VARCHAR(10) NOT NULL CHECK (layer_target IN ('L2', 'L3', 'L4', 'L7')),
    description TEXT
);

CREATE TABLE security_alerts (
    alert_id BIGSERIAL PRIMARY KEY,
    feature_id BIGINT UNIQUE REFERENCES feature_store(feature_id) ON DELETE CASCADE,
    threat_id INT REFERENCES threat_types(threat_id) ON DELETE RESTRICT,
    status_id INT REFERENCES alert_statuses(status_id) ON DELETE RESTRICT,
    anomaly_score FLOAT NOT NULL CHECK (anomaly_score >= 0.0 AND anomaly_score <= 1.0),
    detected_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE execution_queue (
    task_id BIGSERIAL PRIMARY KEY,
    alert_id BIGINT REFERENCES security_alerts(alert_id) ON DELETE CASCADE,
    action_id INT REFERENCES mitigation_actions(action_id) ON DELETE RESTRICT,
    node_id INT REFERENCES network_nodes(node_id) ON DELETE RESTRICT,
    status_id INT REFERENCES task_statuses(status_id) ON DELETE RESTRICT,
    queued_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    executed_at TIMESTAMP WITH TIME ZONE
);

CREATE TABLE rollback_snapshots (
    snapshot_id BIGSERIAL PRIMARY KEY,
    task_id BIGINT UNIQUE REFERENCES execution_queue(task_id) ON DELETE CASCADE,
    pre_state_json JSONB NOT NULL,
    rollback_payload JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE mitigation_audit (
    audit_id BIGSERIAL PRIMARY KEY,
    task_id BIGINT UNIQUE REFERENCES execution_queue(task_id) ON DELETE SET NULL,
    executed_by_user INT REFERENCES sys_users(user_id) ON DELETE SET NULL, 
    executed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    result_status VARCHAR(50) NOT NULL
);

-- =========================================================================
-- 5. AUDITORÍA GENERAL
-- =========================================================================

CREATE TABLE system_audit_logs (
    sys_audit_id BIGSERIAL PRIMARY KEY,
    user_id INT REFERENCES sys_users(user_id) ON DELETE SET NULL,
    action VARCHAR(100) NOT NULL,
    target_table VARCHAR(50) NOT NULL,
    target_id BIGINT,
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);