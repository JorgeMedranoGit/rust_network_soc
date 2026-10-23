#!/bin/bash
set -e

# ==============================================================================
# TIGO SOC - CONSTRUCCIÓN DE LAS DOS REDES GNS3 DEL LABORATORIO
# ==============================================================================
# Red 1 (PRINCIPAL / PRODUCCIÓN SIMULADA): 'Tigo_SOC_ISP_Expanded'
#   - Subred 192.168.100.0/24 (atacante) + 192.168.1.0/24 (víctimas corporativas)
#   - Aquí se lanzan TODOS los ataques y se captura TODO el tráfico que alimenta
#     al pipeline del SOC (telemetría -> ML LightGBM -> alertas en PostgreSQL).
#
# Red 2 (SANDBOX ACL, más pequeña): 'Tigo_SOC_Sandbox_ACL'
#   - Subred segregada 10.200.0.0/24 (VLAN 999), solo 4 nodos:
#     RouterAcl (gateway/inyección ACL), TargetServer, LegitClient, Attacker.
#   - Destinada a PROBAR las ACLs antes de aplicarlas en la red principal.
#   - Una vez APROBADA una ACL en sandbox, esta se guarda en la base de datos
#     (tabla approved_acl_rules) para futuras detecciones automáticas.
#   - Desplegada por scripts/red_alterna_sandbox_acl.sh; este script la
#     referencia y verifica su existencia.
# ==============================================================================

API_URL="http://127.0.0.1:3080/v2"
PROJECT_NAME="Tigo_SOC_ISP_Expanded"
SANDBOX_PROJECT_NAME="Tigo_SOC_Sandbox_ACL"

# Comprobar si GNS3 Server está escuchando en puerto 3080
GNS3_ONLINE=false
if curl -s --connect-timeout 2 "$API_URL/version" &>/dev/null || curl -s --connect-timeout 2 "$API_URL/projects" &>/dev/null; then
    GNS3_ONLINE=true
fi

if [ "$GNS3_ONLINE" = true ]; then
    echo -e "\e[1;34m[INFO] Inicializando construcción de topología ISP en Bash (API GNS3)...\e[0m"

    # 1. Limpiar el proyecto si ya existe
    EXISTING_PROJECT_ID=$(curl -s "$API_URL/projects" | jq -r --arg n "$PROJECT_NAME" '.[] | select(.name==$n) | .project_id' 2>/dev/null || true)

    if [ -n "$EXISTING_PROJECT_ID" ] && [ "$EXISTING_PROJECT_ID" != "null" ]; then
        echo -e "\e[1;33m[INFO] El proyecto '$PROJECT_NAME' ya existe. Eliminando infraestructura anterior...\e[0m"
        curl -s -X DELETE "$API_URL/projects/$EXISTING_PROJECT_ID" > /dev/null || true
        sleep 3
    fi

    # 2. Crear el proyecto nuevo
    PROJECT_RES=$(curl -s -X POST "$API_URL/projects" \
        -H "Content-Type: application/json" \
        -d "{\"name\": \"$PROJECT_NAME\"}")
    PROJECT_ID=$(echo "$PROJECT_RES" | jq -r '.project_id')

    if [ "$PROJECT_ID" == "null" ] || [ -z "$PROJECT_ID" ]; then
        echo -e "\e[1;31m[ERROR] No se pudo crear el proyecto en GNS3. Respuesta: $PROJECT_RES\e[0m"
        exit 1
    fi
    echo -e "\e[1;32m[+] Proyecto creado con ID: $PROJECT_ID\e[0m"

    # 3. Función para instanciar Docker
    create_docker() {
        local name=$1
        local image=$2
        local payload=$(jq -n --arg name "$name" --arg image "$image" \
            '{name: $name, node_type: "docker", compute_id: "local", console_type: "telnet", properties: {image: $image, adapters: 1}}')
        
        local res=$(curl -s -X POST "$API_URL/projects/$PROJECT_ID/nodes" \
            -H "Content-Type: application/json" -d "$payload")
        echo "$res" | jq -r '.node_id'
    }

    echo -e "\e[1;34m[INFO] Creando 10 dispositivos finales en GNS3...\e[0m"
    ATTACKER_ID=$(create_docker "AttackerKali" "kali-soc-tools")

    declare -a VICTIM_IDS
    for i in {1..9}; do
        VICTIM_IDS[$i]=$(create_docker "VictimeAlpine$i" "alpine:latest")
    done

    # 4. Crear el Switch / Router Core
    SWITCH_PAYLOAD='{
        "name": "ISP_Core_Router",
        "node_type": "ethernet_switch",
        "compute_id": "local",
        "properties": {
            "ports_mapping": [
                {"name": "Ethernet0", "port_number": 0, "type": "access", "vlan": 1},
                {"name": "Ethernet1", "port_number": 1, "type": "access", "vlan": 1},
                {"name": "Ethernet2", "port_number": 2, "type": "access", "vlan": 1},
                {"name": "Ethernet3", "port_number": 3, "type": "access", "vlan": 1},
                {"name": "Ethernet4", "port_number": 4, "type": "access", "vlan": 1},
                {"name": "Ethernet5", "port_number": 5, "type": "access", "vlan": 1},
                {"name": "Ethernet6", "port_number": 6, "type": "access", "vlan": 1},
                {"name": "Ethernet7", "port_number": 7, "type": "access", "vlan": 1},
                {"name": "Ethernet8", "port_number": 8, "type": "access", "vlan": 1},
                {"name": "Ethernet9", "port_number": 9, "type": "access", "vlan": 1}
            ]
        }
    }'
    SWITCH_RES=$(curl -s -X POST "$API_URL/projects/$PROJECT_ID/nodes" \
        -H "Content-Type: application/json" -d "$SWITCH_PAYLOAD")
    SWITCH_ID=$(echo "$SWITCH_RES" | jq -r '.node_id')

    # 3.b Red GNS3 SANDBOX ACL (más pequeña, segregada de la principal).
    #     Si no existe el proyecto sandbox en GNS3, se despliega automáticamente
    #     vía scripts/red_alterna_sandbox_acl.sh (equivalente Docker VLAN 999).
    echo -e "\e[1;34m[INFO] Verificando segunda red GNS3: Sandbox ACL (prueba previa de ACLs)...\e[0m"
    SANDBOX_EXISTING_ID=$(curl -s "$API_URL/projects" | jq -r --arg n "$SANDBOX_PROJECT_NAME" '.[] | select(.name==$n) | .project_id' 2>/dev/null || true)
    if [ -z "$SANDBOX_EXISTING_ID" ] || [ "$SANDBOX_EXISTING_ID" == "null" ]; then
        SB_PROJECT_RES=$(curl -s -X POST "$API_URL/projects" \
            -H "Content-Type: application/json" \
            -d "{\"name\": \"$SANDBOX_PROJECT_NAME\", \"summary\": \"Red GNS3 pequena para probar ACLs antes de aplicarlas en la red principal\"}" || true)
        SB_PROJECT_ID=$(echo "$SB_PROJECT_RES" | jq -r '.project_id' 2>/dev/null || true)
        if [ -n "$SB_PROJECT_ID" ] && [ "$SB_PROJECT_ID" != "null" ]; then
            echo -e "\e[1;34m[INFO] Creando 4 nodos minimos en la red Sandbox ACL...\e[0m"
            SB_ROUTER_RES=$(curl -s -X POST "$API_URL/projects/$SB_PROJECT_ID/nodes" \
                -H "Content-Type: application/json" \
                -d '{"name": "Sandbox.RouterAcl", "node_type": "vpcs", "compute_id": "local", "properties": {"adapters": 1}}' || true)
            SB_ROUTER_ID=$(echo "$SB_ROUTER_RES" | jq -r '.node_id')
            SB_TARGET_ID=$(create_docker "Sandbox.TargetServer" "alpine:latest")
            SB_LEGIT_ID=$(create_docker "Sandbox.LegitClient" "alpine:latest")
            SB_ATTACKER_ID=$(create_docker "Sandbox.Attacker" "alpine:latest")
            echo -e "\e[1;32m[+] Proyecto Sandbox ACL creado con ID: $SB_PROJECT_ID (4 nodos)\e[0m"
        else
            echo -e "\e[1;33m[!] No se pudo crear el proyecto sandbox en GNS3. Se usará la red Docker alternativa.\e[0m"
        fi
    else
        echo -e "\e[1;32m[i] El proyecto Sandbox ACL ya existe en GNS3 (ID: $SANDBOX_EXISTING_ID).\e[0m"
    fi

    echo -e "\e[1;34m[INFO] Nodos instanciados. Estableciendo enlaces...\e[0m"

    # 5. Función enlaces
    create_link() {
        local n1=$1
        local p1=$2
        local n2=$3
        local p2=$4
        local payload=$(jq -n --arg n1 "$n1" --argjson p1 "$p1" --arg n2 "$n2" --argjson p2 "$p2" \
            '{nodes: [{node_id: $n1, adapter_number: 0, port_number: $p1}, {node_id: $n2, adapter_number: 0, port_number: $p2}]}')
        
        curl -s -X POST "$API_URL/projects/$PROJECT_ID/links" \
            -H "Content-Type: application/json" -d "$payload" > /dev/null
    }

    create_link "$ATTACKER_ID" 0 "$SWITCH_ID" 0
    for i in {1..9}; do
        create_link "$SWITCH_ID" "$i" "${VICTIM_IDS[$i]}" 0
    done

    # Enlaces de la red Sandbox ACL (más pequeña): RouterAcl en el centro,
    # conectado a TargetServer(0), LegitClient(1) y Attacker(2).
    if [ -n "$SB_ROUTER_ID" ] && [ "$SB_ROUTER_ID" != "null" ]; then
        SB_SWITCH_RES=$(curl -s -X POST "$API_URL/projects/$SB_PROJECT_ID/nodes" \
            -H "Content-Type: application/json" \
            -d '{"name": "Sandbox_Switch", "node_type": "ethernet_switch", "compute_id": "local", "properties": {"ports_mapping": [{"name": "Ethernet0", "port_number": 0, "type": "access", "vlan": 999}, {"name": "Ethernet1", "port_number": 1, "type": "access", "vlan": 999}, {"name": "Ethernet2", "port_number": 2, "type": "access", "vlan": 999}, {"name": "Ethernet3", "port_number": 3, "type": "access", "vlan": 999}]}}' || true)
        SB_SWITCH_ID=$(echo "$SB_SWITCH_RES" | jq -r '.node_id')
        if [ -n "$SB_SWITCH_ID" ] && [ "$SB_SWITCH_ID" != "null" ]; then
            create_link "$SB_ROUTER_ID" 0 "$SB_SWITCH_ID" 0
            create_link "$SB_TARGET_ID" 0 "$SB_SWITCH_ID" 1
            create_link "$SB_LEGIT_ID" 0 "$SB_SWITCH_ID" 2
            create_link "$SB_ATTACKER_ID" 0 "$SB_SWITCH_ID" 3
            echo -e "\e[1;32m[+] Red Sandbox ACL cableada (4 enlaces, VLAN 999).\e[0m"
        fi
    fi

    echo -e "\e[1;34m[INFO] Cableado finalizado. Iniciando nodos en paralelo...\e[0m"
    curl -s -X POST "$API_URL/projects/$PROJECT_ID/nodes/start" \
        -H "Content-Type: application/json" -d "{}" > /dev/null
    if [ -n "$SB_PROJECT_ID" ] && [ "$SB_PROJECT_ID" != "null" ]; then
        curl -s -X POST "$API_URL/projects/$SB_PROJECT_ID/nodes/start" \
            -H "Content-Type: application/json" -d "{}" > /dev/null || true
    fi

    echo -e "\e[1;32m[+] Topología ISP levantada exitosamente por script Bash (GNS3 API).\e[0m"
    echo -e "\e[1;32m[+] Red 1 (PRINCIPAL): $PROJECT_NAME -> ataques + captura de tráfico.\e[0m"
    echo -e "\e[1;32m[+] Red 2 (SANDBOX ACL, más pequeña): $SANDBOX_PROJECT_NAME -> validación previa de ACLs.\e[0m"
else
    echo -e "\e[1;33m[INFO] Servidor GNS3 no disponible en puerto 3080.\e[0m"
    echo -e "\e[1;34m[INFO] Desplegando topología ISP equivalente directamente en Docker (Multi-Distro / Azure Cloud)...\e[0m"
    
    # Crear red bridge dedicada si no existe
    sudo docker network inspect soc_isp_net &>/dev/null || sudo docker network create soc_isp_net || true

    # Crear / reiniciar nodo AttackerKali
    echo -e "\e[1;34m[INFO] Desplegando GNS3.AttackerKali...\e[0m"
    if [ "$(sudo docker ps -aq -f name=^/GNS3.AttackerKali$)" ]; then
        sudo docker rm -f GNS3.AttackerKali >/dev/null
    fi
    sudo docker run -d --name GNS3.AttackerKali --network soc_isp_net --cap-add=NET_ADMIN --privileged kali-soc-tools sleep infinity

    # Crear / reiniciar nodos VictimeAlpine1 .. VictimeAlpine9
    echo -e "\e[1;34m[INFO] Desplegando 9 víctimas Alpine (GNS3.VictimeAlpine1..9)...\e[0m"
    for i in {1..9}; do
        if [ "$(sudo docker ps -aq -f name=^/GNS3.VictimeAlpine${i}$)" ]; then
            sudo docker rm -f "GNS3.VictimeAlpine${i}" >/dev/null
        fi
        sudo docker run -d --name "GNS3.VictimeAlpine${i}" --network soc_isp_net --cap-add=NET_ADMIN --privileged alpine:latest sleep infinity
    done

    echo -e "\e[1;32m[+] Topología ISP levantada exitosamente en Docker (10 nodos activos con prefijo GNS3.*).\e[0m"

    # Red 2 (SANDBOX ACL, más pequeña): desplegar la red Docker segregada
    # soc_sandbox_acl_net (VLAN 999 / 10.200.0.0/24) para probar ACLs antes
    # de aplicarlas en la red principal. Las ACLs aprobadas aquí se guardan
    # en PostgreSQL (approved_acl_rules) por scripts/probar_nuevas_acl.sh.
    if [ -x "$PWD/scripts/red_alterna_sandbox_acl.sh" ]; then
        echo -e "\e[1;34m[INFO] Desplegando segunda red: SANDBOX ACL (soc_sandbox_acl_net, 10.200.0.0/24)...\e[0m"
        chmod +x "$PWD/scripts/red_alterna_sandbox_acl.sh"
        "$PWD/scripts/red_alterna_sandbox_acl.sh" || echo -e "\e[1;33m[AVISO] No se pudo desplegar la red sandbox automáticamente.\e[0m"
    fi

    echo -e "\e[1;32m[+] RED 1 (PRINCIPAL): soc_isp_net  -> ataques y captura de tráfico.\e[0m"
    echo -e "\e[1;32m[+] RED 2 (SANDBOX ACL, más pequeña): soc_sandbox_acl_net -> validación previa de ACLs + aprobación persistida en BD.\e[0m"
fi
