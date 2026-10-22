#!/bin/bash
set -e

API_URL="http://127.0.0.1:3080/v2"
PROJECT_NAME="Tigo_SOC_ISP_Expanded"

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

    echo -e "\e[1;34m[INFO] Cableado finalizado. Iniciando nodos en paralelo...\e[0m"
    curl -s -X POST "$API_URL/projects/$PROJECT_ID/nodes/start" \
        -H "Content-Type: application/json" -d "{}" > /dev/null

    echo -e "\e[1;32m[+] Topología ISP levantada exitosamente por script Bash (GNS3 API).\e[0m"
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
fi
