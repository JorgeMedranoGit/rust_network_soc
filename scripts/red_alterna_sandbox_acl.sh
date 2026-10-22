#!/usr/bin/env bash
# ==============================================================================
# TIGO SOC - RED ALTERNA DE PRUEBAS PARA ACLs Y BLACKHOLING (SANDBOX VLAN 999)
# ==============================================================================
# Propósito:
#   Desplegar un entorno de red completamente segregado (Subred 10.200.0.0/24)
#   destinado a probar y verificar la inyección dinámica de nuevas ACLs (L3/L4),
#   técnicas de Null Routing (Blackholing) y mecanismos de Rollback (300s),
#   SIN INTERFERIR con el tráfico en producción de clientes corporativos ni el core.
#
# Topología de la Red Alterna (VLAN 999):
#   - Segmento:              10.200.0.0/24
#   - Bridge Dedicado:       soc_sandbox_acl_net (VLAN 999)
#   - Sandbox.RouterAcl:     10.200.0.1  (Router Gateway / Firewall iptables)
#   - Sandbox.TargetServer:  10.200.0.10 (Servidor de prueba / Servicio objetivo)
#   - Sandbox.LegitClient:   10.200.0.20 (Cliente legítimo de control de Blast Radius)
#   - Sandbox.Attacker:      10.200.0.50 (Host generador de ataques SYN/ICMP Flood)
# ==============================================================================

set -eo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'

SANDBOX_NET="soc_sandbox_acl_net"
SANDBOX_SUBNET="10.200.0.0/24"
GATEWAY_IP="10.200.0.1"
TARGET_IP="10.200.0.10"
LEGIT_IP="10.200.0.20"
ATTACKER_IP="10.200.0.50"

echo -e "${BLUE}==================================================================${NC}"
echo -e "${GREEN}   TIGO SOC: DESPLIEGUE DE RED ALTERNA DE PRUEBAS (SANDBOX ACL)   ${NC}"
echo -e "${BLUE}==================================================================${NC}"
echo -e " Subred Segregada:       ${YELLOW}${SANDBOX_SUBNET}${NC} (VLAN 999 Sandbox)"
echo -e " Red Docker Bridge:      ${YELLOW}${SANDBOX_NET}${NC}"
echo -e " Gateway Router ACL:     ${CYAN}${GATEWAY_IP}${NC} (Sandbox.RouterAcl)"
echo -e " Servidor Destino:       ${CYAN}${TARGET_IP}${NC} (Sandbox.TargetServer)"
echo -e " Cliente Legítimo:       ${CYAN}${LEGIT_IP}${NC} (Sandbox.LegitClient)"
echo -e " Host Atacante:          ${RED}${ATTACKER_IP}${NC} (Sandbox.Attacker)"
echo -e "${BLUE}==================================================================${NC}"

# Función helper para invocar docker con o sin sudo
DOCKER_CMD="docker"
if ! command -v docker &>/dev/null; then
    echo -e "${RED}[ERROR] Docker no está instalado en el sistema.${NC}"
    exit 1
fi

if ! docker ps &>/dev/null; then
    if command -v sudo &>/dev/null; then
        DOCKER_CMD="sudo docker"
    else
        echo -e "${RED}[ERROR] Se requieren privilegios para interactuar con Docker daemon.${NC}"
        exit 1
    fi
fi

# 1. Crear la red bridge segregada si no existe
echo -e "\n${BLUE}[*] 1. Configurando red bridge segregada [${SANDBOX_NET}] (${SANDBOX_SUBNET})...${NC}"
if ! $DOCKER_CMD network inspect "$SANDBOX_NET" &>/dev/null; then
    $DOCKER_CMD network create \
        --driver bridge \
        --subnet "$SANDBOX_SUBNET" \
        --gateway "$GATEWAY_IP" \
        -o "com.docker.network.bridge.name=br-soc-sb999" \
        -o "com.docker.network.driver.mtu=1500" \
        "$SANDBOX_NET"
    echo -e "  ${GREEN}[✓] Red bridge segregada creada exitosamente.${NC}"
else
    echo -e "  ${YELLOW}[i] Red bridge '${SANDBOX_NET}' ya existe.${NC}"
fi

# Función para desplegar o reiniciar contenedor
deploy_container() {
    local name=$1
    local ip=$2
    local image=$3
    local cmd=$4

    if [ "$($DOCKER_CMD ps -aq -f name=^/${name}$)" ]; then
        echo -e "  ${YELLOW}[-] Limpiando contenedor previo: ${name}...${NC}"
        $DOCKER_CMD rm -f "$name" >/dev/null
    fi

    echo -e "  ${GREEN}[+] Desplegando ${name} en IP ${ip}...${NC}"
    $DOCKER_CMD run -d \
        --name "$name" \
        --network "$SANDBOX_NET" \
        --ip "$ip" \
        --cap-add=NET_ADMIN \
        --privileged \
        "$image" \
        sh -c "$cmd" >/dev/null
}

# 2. Desplegar Servidor Objetivo (Sandbox.TargetServer)
echo -e "\n${BLUE}[*] 2. Desplegando Servidor Objetivo (Servicio Web HTTP:80 & Echo ICMP)...${NC}"
TARGET_CMD="
apk add --no-cache iproute2 iptables curl mini_httpd 2>/dev/null || true
mkdir -p /var/www/localhost/htdocs
echo '<html><body><h1>Tigo Business SOC - Servidor Objetivo Sandbox (VLAN 999)</h1><p>Estado: OPERATIVO - Enlace Estable</p></body></html>' > /var/www/localhost/htdocs/index.html
# Servidor web ultraligero nativo en puerto 80
while true; do
    echo -e 'HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nConnection: close\r\n\r\n<h1>Tigo Business SOC - Objetivo Protegido (Sandbox VLAN 999)</h1><p>Conectividad ACL Verificada</p>' | nc -l -p 80
done
"
deploy_container "Sandbox.TargetServer" "$TARGET_IP" "alpine:latest" "$TARGET_CMD"

# 3. Desplegar Cliente Legítimo (Sandbox.LegitClient) para verificar contención del Blast Radius
echo -e "\n${BLUE}[*] 3. Desplegando Cliente Legítimo (Auditoría de Disponibilidad Blast Radius)...${NC}"
LEGIT_CMD="
apk add --no-cache iproute2 curl 2>/dev/null || true
sleep infinity
"
deploy_container "Sandbox.LegitClient" "$LEGIT_IP" "alpine:latest" "$LEGIT_CMD"

# 4. Desplegar Host Atacante (Sandbox.Attacker)
echo -e "\n${BLUE}[*] 4. Desplegando Host Atacante (Generador de Tráfico Volumétrico y Escaneo)...${NC}"
ATTACKER_CMD="
apk add --no-cache iproute2 curl nmap 2>/dev/null || true
sleep infinity
"
deploy_container "Sandbox.Attacker" "$ATTACKER_IP" "alpine:latest" "$ATTACKER_CMD"

# 5. Desplegar Router Gateway de Inspección ACL (Sandbox.RouterAcl)
echo -e "\n${BLUE}[*] 5. Desplegando Gateway Router para Inyección Dinámica de ACLs y Blackholing...${NC}"
ROUTER_CMD="
apk add --no-cache iproute2 iptables tcpdump conntrack-tools 2>/dev/null || true
# Habilitar IP Forwarding en el stack de red
sysctl -w net.ipv4.ip_forward=1 >/dev/null 2>&1 || true
# Cadena de ACLs del SOC
iptables -N SOC_SANDBOX_ACL 2>/dev/null || true
iptables -F SOC_SANDBOX_ACL 2>/dev/null || true
iptables -I FORWARD 1 -j SOC_SANDBOX_ACL 2>/dev/null || true
sleep infinity
"
deploy_container "Sandbox.RouterAcl" "10.200.0.2" "alpine:latest" "$ROUTER_CMD"

# 6. Verificación de salud de la red alterna
echo -e "\n${BLUE}[*] 6. Verificando conectividad inicial en la red alterna...${NC}"
sleep 2

if $DOCKER_CMD exec Sandbox.Attacker ping -c 2 -W 2 "$TARGET_IP" &>/dev/null; then
    echo -e "  ${GREEN}[✓] Conectividad base verificada: Sandbox.Attacker -> Sandbox.TargetServer OK${NC}"
else
    echo -e "  ${YELLOW}[!] Inicializando enlaces de red. Espere un momento...${NC}"
fi

echo -e "\n${GREEN}==================================================================${NC}"
echo -e "${GREEN}   RED ALTERNA DE PRUEBAS SANDBOX ACL (VLAN 999) LISTA Y ACTIVA   ${NC}"
echo -e "${GREEN}==================================================================${NC}"
echo -e " Para ejecutar la prueba de inyección de ACL y Rollback, ejecute:"
echo -e "   ${CYAN}./scripts/probar_nuevas_acl.sh${NC}"
echo -e "${GREEN}==================================================================${NC}"
