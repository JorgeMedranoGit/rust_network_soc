-- * * * ESQUEMA DE BASE DE DATOS TIGOSOC EN TERCERA FORMA NORMAL (BCNF) * * *

-- * * * 1. CATÁLOGOS BASE DEL SISTEMA * * *

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

-- * * * 2. RBAC Y SEGURIDAD DEL SISTEMA * * *

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

-- * * * SESIONES SEGURAS EN SERVIDOR (TOKENS OPACOS CON TIEMPO DE ACTIVIDAD Y RE-AUTENTICACIÓN) * * *
CREATE TABLE user_sessions (
    session_id VARCHAR(64) PRIMARY KEY,
    user_id INT NOT NULL REFERENCES sys_users(user_id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    last_activity TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    last_reauth_at TIMESTAMP WITH TIME ZONE,
    is_revoked BOOLEAN DEFAULT FALSE,
    ip_address VARCHAR(45),
    user_agent TEXT
);


-- * * * 3. INVENTARIO Y TELEMETRÍA (DIVIDIDA: AGREGADA VS FORENSE) * * *

CREATE TABLE network_nodes (
    node_id SERIAL PRIMARY KEY,
    hostname VARCHAR(100) UNIQUE NOT NULL,
    ip_address VARCHAR(45) UNIQUE NOT NULL,
    type_id INT REFERENCES device_types(type_id) ON DELETE RESTRICT,
    protocol_id INT REFERENCES mgmt_protocols(protocol_id) ON DELETE RESTRICT,
    mac_address VARCHAR(50),
    vendor VARCHAR(50)
);

-- * * * MÉTRICAS AGREGADAS PERIÓDICAS PARA VOLÚMENES MASIVOS SIN SATURAR DISCO * * *
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

-- * * * REGISTROS FORENSES AISLADOS EXCLUSIVOS PARA ANOMALÍAS DE SEGURIDAD * * *
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

-- * * * 4. ORQUESTACIÓN Y MITIGACIÓN AGNOSTICA * * *

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

-- * * * 5. AUDITORÍA GENERAL DEL SISTEMA * * *

CREATE TABLE system_audit_logs (
    sys_audit_id BIGSERIAL PRIMARY KEY,
    user_id INT REFERENCES sys_users(user_id) ON DELETE SET NULL,
    action VARCHAR(100) NOT NULL,
    target_table VARCHAR(50) NOT NULL,
    target_id BIGINT,
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- * * * 6. DATOS SEMILLA (SEED DATA) INICIALES DEL SISTEMA * * *

-- Catálogos de tipos de dispositivos
INSERT INTO device_types (type_name) VALUES
    ('ROUTER_CORE_L3'),
    ('FIREWALL_PERIMETER'),
    ('SWITCH_DISTRIBUTION'),
    ('WORKSTATION_VICTIM'),
    ('ATTACKER_KALI')
ON CONFLICT (type_name) DO NOTHING;

-- Catálogos de protocolos de administración
INSERT INTO mgmt_protocols (protocol_name) VALUES
    ('SSH'),
    ('NETCONF'),
    ('RESTCONF'),
    ('SNMP')
ON CONFLICT (protocol_name) DO NOTHING;

-- Estados de alertas forenses
INSERT INTO alert_statuses (status_name) VALUES
    ('PENDING_ANALYSIS'),
    ('CONFIRMED_THREAT'),
    ('FALSE_POSITIVE'),
    ('MITIGATED')
ON CONFLICT (status_name) DO NOTHING;

-- Estados de tareas de mitigación
INSERT INTO task_statuses (status_name) VALUES
    ('QUEUED'),
    ('EXECUTING'),
    ('COMPLETED'),
    ('FAILED'),
    ('ROLLED_BACK'),
    ('PENDING_MANUAL')
ON CONFLICT (status_name) DO NOTHING;

-- Catálogo de amenazas y severidades
INSERT INTO threat_types (threat_name, severity_level) VALUES
    ('DATA_EXFILTRATION', 5),
    ('DNS_C2_EXFILTRATION', 5),
    ('PORT_SCAN', 2),
    ('DDOS_SYN_FLOOD', 4),
    ('UNAUTHORIZED_ACCESS', 3),
    ('ICMP_FLOOD', 3),
    ('MAC_SPOOFING', 4)
ON CONFLICT (threat_name) DO NOTHING;

-- Acciones de mitigación multi-vendor
INSERT INTO mitigation_actions (action_name, layer_target, description) VALUES
    ('ISOLATE_L2_RADIUS_COA', 'L2', 'Aislamiento de host o cambio a VLAN de cuarentena mediante RADIUS CoA UDP 3799'),
    ('ACL_BLACKHOLE_L3_L4', 'L3', 'Descarte perimetral inmediato de IP y puertos vía iptables Null Route'),
    ('DNS_SINKHOLE_L7', 'L7', 'Redirección controlada de peticiones DNS hacia IP de Sinkhole seguro')
ON CONFLICT (action_name) DO NOTHING;

-- Roles del sistema (RBAC Zero-Trust)
INSERT INTO roles (role_name, description) VALUES
    ('SOC_ADMIN', 'Administrador general del SOC con privilegios totales'),
    ('SOC_OPERATOR', 'Operador SOC - Aprobación de mitigaciones masivas y rollback'),
    ('SOC_AUDITOR', 'Auditor de Seguridad - Solo lectura e inspección forense'),
    ('SOC_ANALYST_L1', 'Analista L1 - Triaje inicial y monitoreo en streaming'),
    ('SOC_ANALYST_L2', 'Analista L2 - Respuesta a incidentes y análisis profundo')
ON CONFLICT (role_name) DO NOTHING;

-- Permisos granulares
INSERT INTO permissions (permission_name, description) VALUES
    ('USER_MANAGE', 'Creación y administración de usuarios RBAC'),
    ('MITIGATE_EXECUTE', 'Aprobación y despacho de directivas de mitigación'),
    ('ROLLBACK_EXECUTE', 'Ejecución de directivas inversas de reversión'),
    ('READ_ONLY', 'Lectura e inspección de alertas, telemetría y reportes')
ON CONFLICT (permission_name) DO NOTHING;

-- Asignación de permisos por rol
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.role_id, p.permission_id
FROM roles r, permissions p
WHERE r.role_name = 'SOC_ADMIN'
ON CONFLICT (role_id, permission_id) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.role_id, p.permission_id
FROM roles r, permissions p
WHERE r.role_name = 'SOC_OPERATOR' AND p.permission_name IN ('MITIGATE_EXECUTE', 'ROLLBACK_EXECUTE', 'READ_ONLY')
ON CONFLICT (role_id, permission_id) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.role_id, p.permission_id
FROM roles r, permissions p
WHERE r.role_name IN ('SOC_AUDITOR', 'SOC_ANALYST_L1', 'SOC_ANALYST_L2') AND p.permission_name = 'READ_ONLY'
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- Parámetros del sistema (Timeout crítico de 300 segundos = 5 minutos)
INSERT INTO system_parameters (param_key, param_value, description) VALUES
    ('critical_action_timeout_secs', '300', 'Ventana máxima de inactividad antes de exigir re-autenticación por contraseña para acciones críticas')
ON CONFLICT (param_key) DO NOTHING;

-- Usuarios iniciales del sistema (Contraseña por defecto: admin)
INSERT INTO sys_users (username, password_hash, role_id) VALUES
    ('admin_soc', 'admin', (SELECT role_id FROM roles WHERE role_name = 'SOC_ADMIN')),
    ('operador_red', 'admin', (SELECT role_id FROM roles WHERE role_name = 'SOC_OPERATOR')),
    ('auditor_seguridad', 'admin', (SELECT role_id FROM roles WHERE role_name = 'SOC_AUDITOR')),
    ('analista_triaje', 'admin', (SELECT role_id FROM roles WHERE role_name = 'SOC_ANALYST_L1'))
ON CONFLICT (username) DO NOTHING;

-- Nodos de la topología de red GNS3 / Tigo Business Bolivia
INSERT INTO network_nodes (hostname, ip_address, type_id, protocol_id, mac_address, vendor) VALUES
    ('AttackerKali', '192.168.100.50', (SELECT type_id FROM device_types WHERE type_name = 'ATTACKER_KALI'), (SELECT protocol_id FROM mgmt_protocols WHERE protocol_name = 'SSH'), '00:00:0c:aa:bb:cc', 'Cisco ISE'),
    ('VictimeAlpine1', '192.168.1.11', (SELECT type_id FROM device_types WHERE type_name = 'WORKSTATION_VICTIM'), (SELECT protocol_id FROM mgmt_protocols WHERE protocol_name = 'SSH'), '02:42:ac:11:00:0b', 'GNS3 Alpine'),
    ('VictimeAlpine2', '192.168.1.12', (SELECT type_id FROM device_types WHERE type_name = 'WORKSTATION_VICTIM'), (SELECT protocol_id FROM mgmt_protocols WHERE protocol_name = 'SSH'), '02:42:ac:11:00:0c', 'GNS3 Alpine'),
    ('VictimeAlpine3', '192.168.1.13', (SELECT type_id FROM device_types WHERE type_name = 'WORKSTATION_VICTIM'), (SELECT protocol_id FROM mgmt_protocols WHERE protocol_name = 'SSH'), '02:42:ac:11:00:0d', 'GNS3 Alpine'),
    ('VictimeAlpine4', '192.168.1.14', (SELECT type_id FROM device_types WHERE type_name = 'WORKSTATION_VICTIM'), (SELECT protocol_id FROM mgmt_protocols WHERE protocol_name = 'SSH'), '02:42:ac:11:00:0e', 'GNS3 Alpine'),
    ('VictimeAlpine5', '192.168.1.15', (SELECT type_id FROM device_types WHERE type_name = 'WORKSTATION_VICTIM'), (SELECT protocol_id FROM mgmt_protocols WHERE protocol_name = 'SSH'), '02:42:ac:11:00:0f', 'GNS3 Alpine'),
    ('VictimeAlpine6', '192.168.1.16', (SELECT type_id FROM device_types WHERE type_name = 'WORKSTATION_VICTIM'), (SELECT protocol_id FROM mgmt_protocols WHERE protocol_name = 'SSH'), '02:42:ac:11:00:10', 'GNS3 Alpine'),
    ('VictimeAlpine7', '192.168.1.17', (SELECT type_id FROM device_types WHERE type_name = 'WORKSTATION_VICTIM'), (SELECT protocol_id FROM mgmt_protocols WHERE protocol_name = 'SSH'), '02:42:ac:11:00:11', 'GNS3 Alpine'),
    ('VictimeAlpine8', '192.168.1.18', (SELECT type_id FROM device_types WHERE type_name = 'WORKSTATION_VICTIM'), (SELECT protocol_id FROM mgmt_protocols WHERE protocol_name = 'SSH'), '02:42:ac:11:00:12', 'GNS3 Alpine'),
    ('VictimeAlpine9', '192.168.1.19', (SELECT type_id FROM device_types WHERE type_name = 'WORKSTATION_VICTIM'), (SELECT protocol_id FROM mgmt_protocols WHERE protocol_name = 'SSH'), '02:42:ac:11:00:13', 'GNS3 Alpine'),
    ('pfSense_Gateway', '192.168.1.1', (SELECT type_id FROM device_types WHERE type_name = 'FIREWALL_PERIMETER'), (SELECT protocol_id FROM mgmt_protocols WHERE protocol_name = 'SSH'), '00:18:0a:33:44:55', 'OpenWrt'),
    ('Router_Core_L3', '192.168.1.254', (SELECT type_id FROM device_types WHERE type_name = 'ROUTER_CORE_L3'), (SELECT protocol_id FROM mgmt_protocols WHERE protocol_name = 'SSH'), '00:0c:42:66:77:88', 'MikroTik RouterOS')
ON CONFLICT (hostname) DO NOTHING;
