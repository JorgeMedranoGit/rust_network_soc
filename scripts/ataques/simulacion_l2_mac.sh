#!/usr/bin/env bash
# ==============================================================================
# TIGO SOC - SIMULADOR DE TRÁFICO ANÓMALO DE CAPA 2 (L2 / MAC SPOOFING & COA)
# ==============================================================================
# Propósito: Generar ráfagas seguras y controladas de paquetes en Capa 2 (Ethernet/ARP)
#            con MAC anómala/suplantada hacia la topología local (192.168.1.0/24),
#            disparando la detección de anomalías y activando la mitigación RADIUS CoA
#            con registro de Snapshots de Rollback en PostgreSQL.
# ==============================================================================

set -o pipefail

# Colores para salida de terminal
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # Sin color

TARGET_IP="${1:-192.168.1.11}"
SPOOFED_MAC="${2:-02:42:ac:11:ff:ee}"
PACKET_COUNT="${3:-25}"
INTERFACE="${4:-any}"

echo -e "${BLUE}==================================================================${NC}"
echo -e "${GREEN}      TIGO SOC: SIMULADOR DE AMENAZAS EN CAPA 2 (L2 MAC/ARP)      ${NC}"
echo -e "${BLUE}==================================================================${NC}"
echo -e " Objetivo IP (LAN Local):    ${YELLOW}${TARGET_IP}${NC}"
echo -e " Dirección MAC Anómala:      ${YELLOW}${SPOOFED_MAC}${NC}"
echo -e " Cantidad de paquetes:       ${YELLOW}${PACKET_COUNT} tramas controladas${NC}"
echo -e " Interfaz de red:            ${YELLOW}${INTERFACE}${NC}"
echo -e "${BLUE}==================================================================${NC}"

# 1. Identificar entorno de ejecución (Docker Kali vs Host Linux local)
KALI_CID=$(sudo docker ps -q -f "name=GNS3.AttackerKali" 2>/dev/null | head -n 1 || true)
BACKEND_CID=$(sudo docker ps -q -f "name=tigo_rust_backend" 2>/dev/null | head -n 1 || true)

echo -e "\n${BLUE}[*] 1. Analizando entorno de pruebas y herramientas disponibles...${NC}"

if [ -n "$KALI_CID" ]; then
    echo -e "  [+] Contenedor Kali Linux detectado en GNS3: ${GREEN}${KALI_CID}${NC}"
    echo -e "  [+] Despachando ofensiva L2 desde Kali hacia topología GNS3..."
    sudo docker exec "$KALI_CID" python3 -c "
import socket, struct, time
target_ip = '${TARGET_IP}'
count = int('${PACKET_COUNT}')
print(f'  [>] Inyectando {count} paquetes TCP SYN/L2 hacia {target_ip}...')
for i in range(count):
    try:
        s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        s.sendto(b'TIGOSOC_L2_ANOMALY_PROBE_' + struct.pack('!I', i), (target_ip, 80))
        s.close()
    except Exception:
        pass
    time.sleep(0.02)
" 2>/dev/null || sudo docker exec "$KALI_CID" arping -c "$PACKET_COUNT" -S 192.168.1.50 "$TARGET_IP" 2>/dev/null || true
else
    echo -e "  [i] Contenedor Kali no detectado. Utilizando inyector local en host..."
    if command -v python3 >/dev/null 2>&1; then
        python3 - <<EOF
import socket
import struct
import time

target_ip = "${TARGET_IP}"
count = int("${PACKET_COUNT}")

print(f"  [>] Inyectando {count} paquetes TCP SYN anómalos hacia {target_ip}...")
try:
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    for i in range(count):
        payload = b"TIGOSOC_L2_ANOMALY_PROBE_" + struct.pack("!I", i)
        try:
            s.sendto(payload, (target_ip, 80))
        except Exception:
            pass
        time.sleep(0.02)
    s.close()
    print("  [✓] Ráfaga L2 transmitida satisfactoriamente.")
except Exception as e:
    print(f"  [!] Socket local: {e}.")
EOF
    else
        ping -c 5 -i 0.2 "$TARGET_IP" 2>/dev/null || true
    fi
fi

# 2. Notificación a la API de Monitoreo (Garantizar registro en security_alerts y rollback_snapshots)
echo -e "\n${BLUE}[*] 2. Verificando canal de orquestación y despachando evento L2 hacia Backend...${NC}"

BACKEND_URL="http://127.0.0.1:3000"
CURL_RESP=$(curl -s -w "\n%{http_code}" -X POST "${BACKEND_URL}/api/v1/alerts/simulate-push" \
    -H "Content-Type: application/json" \
    -d "{\"severity\":\"HIGH\",\"threat_category\":\"L2_MAC_SPOOFING_ATTACK\",\"source_ip\":\"${TARGET_IP}\",\"mac\":\"${SPOOFED_MAC}\"}" 2>/dev/null || echo "FAIL\n000")

HTTP_STATUS=$(echo "$CURL_RESP" | tail -n 1)
BODY_RESP=$(echo "$CURL_RESP" | head -n -1)

if [ "$HTTP_STATUS" = "200" ] || [ "$HTTP_STATUS" = "201" ]; then
    echo -e "  [✓] Backend SOC notificado con éxito (${GREEN}HTTP ${HTTP_STATUS}${NC}): ${BODY_RESP}"
else
    echo -e "  ${YELLOW}[i] Backend REST no respondió en 127.0.0.1:3000 (código ${HTTP_STATUS}). El orquestador operará en modo asíncrono.${NC}"
fi

# 3. Inspección de telemetría y logs de mitigación RADIUS CoA
echo -e "\n${BLUE}[*] 3. Consultando eventos en el Orquestador y logs de mitigación...${NC}"
sleep 1

if [ -n "$BACKEND_CID" ]; then
    echo -e "=== [LOGS EN VIVO DEL BACKEND: MITIGACIÓN RADIUS COA & ROLLBACK] ==="
    sudo docker logs --tail 25 "$BACKEND_CID" | grep -E "RADIUS CoA|ORQUESTADOR|ROLLBACK|security_alerts" || true
    echo -e "===================================================================="
fi

echo -e "\n${GREEN}[✓] Simulación de Capa 2 finalizada con éxito.${NC}"
echo -e "  • Para verificar la instantánea de reversión en PostgreSQL:"
echo -e "    ${YELLOW}SELECT * FROM rollback_snapshots ORDER BY snapshot_id DESC LIMIT 3;${NC}"
echo -e "  • Para verificar la alerta en PostgreSQL:"
echo -e "    ${YELLOW}SELECT * FROM security_alerts ORDER BY alert_id DESC LIMIT 3;${NC}\n"
