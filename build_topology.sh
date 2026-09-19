#!/bin/bash
set -e

API_URL="http://127.0.0.1:3080/v2"

echo -e "\e[1;34m[INFO] Inicializando construcción de topología ISP en Bash (API GNS3)...\e[0m"

# 1. Crear el proyecto
PROJECT_RES=$(curl -s -X POST "$API_URL/projects" \
    -H "Content-Type: application/json" \
    -d '{"name": "Tigo_SOC_ISP_Expanded"}')
PROJECT_ID=$(echo "$PROJECT_RES" | jq -r '.project_id')

if [ "$PROJECT_ID" == "null" ] || [ -z "$PROJECT_ID" ]; then
    echo -e "\e[1;31m[ERROR] No se pudo crear el proyecto en GNS3. Respuesta: $PROJECT_RES\e[0m"
    exit 1
fi
echo -e "\e[1;32m[+] Proyecto creado con ID: $PROJECT_ID\e[0m"

# 2. Función para instanciar Docker
create_docker() {
    local name=$1
    local image=$2
    local payload=$(jq -n --arg name "$name" --arg image "$image" \
        '{name: $name, node_type: "docker", compute_id: "local", properties: {image: $image, adapters: 1}}')
    
    local res=$(curl -s -X POST "$API_URL/projects/$PROJECT_ID/nodes" \
        -H "Content-Type: application/json" -d "$payload")
    echo "$res" | jq -r '.node_id'
}

echo -e "\e[1;34m[INFO] Creando 10 dispositivos finales...\e[0m"
ATTACKER_ID=$(create_docker "AttackerKali" "kalilinux/kali-rolling")

declare -a VICTIM_IDS
for i in {1..9}; do
    VICTIM_IDS[$i]=$(create_docker "VictimeAlpine$i" "alpine:latest")
done

# 3. Crear el Router Core del ISP (Switch de 16 puertos)
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

echo -e "\e[1;34m[INFO] Nodos instanciados. Estableciendo enlaces...\e[0m"

# 4. Función para crear enlaces
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

# Conectar Kali al puerto 0
create_link "$ATTACKER_ID" 0 "$SWITCH_ID" 0

# Conectar las 9 Víctimas a los puertos 1-9
for i in {1..9}; do
    create_link "$SWITCH_ID" "$i" "${VICTIM_IDS[$i]}" 0
done

echo -e "\e[1;34m[INFO] Cableado finalizado. Iniciando nodos en paralelo...\e[0m"
curl -s -X POST "$API_URL/projects/$PROJECT_ID/nodes/start" > /dev/null

echo -e "\e[1;32m[+] Topología ISP levantada exitosamente por script Bash.\e[0m"