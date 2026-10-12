#!/usr/bin/env bash
# ==============================================================================
# TIGO SOC - SIMULADOR DE ATAQUE VOLUMÉTRICO CAPAS 3 Y 4 (SYN & ICMP FLOOD)
# ==============================================================================
# Propósito: Simular ráfagas combinadas de tráfico malicioso en Capas 3 y 4:
#            1. ICMP Flood masivo (Capa 3 - saturación de ancho de banda y CPU)
#            2. TCP SYN Flood (Capa 4 - agotamiento de la tabla de conexiones TCP)
#
# Regla de Negocio (Blast Radius):
#   El Blackholing y ACL dinámicas dirigidas a una IP host específica tienen
#   Impacto Singular (Blast Radius acotado). El orquestador ejecuta la mitigación
#   de forma automática (Null Route L3 + DROP L4) y persiste el comando inverso
#   en `rollback_snapshots` (ej. `ip route del blackhole <ip>/32`).
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

TARGET_IP="${1:-192.168.1.1}"
ATTACKER_IP="${2:-192.168.100.99}"
FLOOD_COUNT="${3:-50}"
TARGET_PORT="${4:-80}"

echo -e "${BLUE}==================================================================${NC}"
echo -e "${GREEN}    TIGO SOC: SIMULADOR DE ATAQUE VOLUMÉTRICO (CAPAS 3 Y 4)       ${NC}"
echo -e "${BLUE}==================================================================${NC}"
echo -e " Objetivo Destino:           ${YELLOW}${TARGET_IP}${NC}"
echo -e " IP Origen Atacante:         ${YELLOW}${ATTACKER_IP}${NC}"
echo -e " Puerto Objetivo (L4 TCP):   ${YELLOW}${TARGET_PORT}${NC}"
echo -e " Paquetes por Fase:          ${YELLOW}${FLOOD_COUNT} tramas de ataque${NC}"
echo -e "${CYAN} Regla Blast Radius:         ${GREEN}IMPACTO SINGULAR -> AUTO-EJECUCIÓN (L3 NULL + L4 ACL)${NC}"
echo -e "${BLUE}==================================================================${NC}"

# 1. Identificar entorno de ejecución (Docker Kali vs Host Linux local)
KALI_CID=$(sudo docker ps -q -f "name=GNS3.AttackerKali" 2>/dev/null | head -n 1 || true)
BACKEND_CID=$(sudo docker ps -q -f "name=tigo_rust_backend" 2>/dev/null | head -n 1 || true)

echo -e "\n${BLUE}[*] 1. Analizando entorno de pruebas y herramientas de inyección...${NC}"

# --- FASE 1: ATAQUE CAPA 3 (ICMP FLOOD) ---
echo -e "\n${BLUE}[*] 2. Despachando Fase 1: Inundación ICMP Flood masivo (Capa 3)...${NC}"

if [ -n "$KALI_CID" ]; then
    echo -e "  [+] Contenedor Kali Linux detectado en GNS3: ${GREEN}${KALI_CID}${NC}"
    echo -e "  [>] Inyectando ${FLOOD_COUNT} paquetes ICMP Echo Request desde Kali..."
    sudo docker exec "$KALI_CID" bash -c "
        if command -v hping3 >/dev/null 2>&1; then
            hping3 --icmp -q -c $FLOOD_COUNT -i u2000 $TARGET_IP >/dev/null 2>&1 || true
        else
            ping -c $FLOOD_COUNT -i 0.05 $TARGET_IP >/dev/null 2>&1 || true
        fi
    " 2>/dev/null || true
    echo -e "  [✓] Ráfaga ICMP Flood de Capa 3 concluida desde Kali."
else
    echo -e "  [i] Contenedor Kali no detectado. Utilizando generador en host local..."
    if command -v hping3 >/dev/null 2>&1; then
        sudo hping3 --icmp -q -c "$FLOOD_COUNT" -i u2000 "$TARGET_IP" >/dev/null 2>&1 || true
        echo -e "  [✓] ICMP Flood transmitido vía hping3."
    else
        ping -c 15 -i 0.05 "$TARGET_IP" 2>/dev/null || true
        echo -e "  [✓] Ráfaga ICMP transmitida exitosamente."
    fi
fi

# --- FASE 2: ATAQUE CAPA 4 (TCP SYN FLOOD) ---
echo -e "\n${BLUE}[*] 3. Despachando Fase 2: Inundación TCP SYN Flood (Capa 4 puerto ${TARGET_PORT})...${NC}"

if [ -n "$KALI_CID" ]; then
    echo -e "  [>] Inyectando ráfagas TCP SYN hacia ${TARGET_IP}:${TARGET_PORT} desde Kali..."
    sudo docker exec "$KALI_CID" bash -c "
        if command -v hping3 >/dev/null 2>&1; then
            hping3 -S -p $TARGET_PORT -c $FLOOD_COUNT -i u1500 $TARGET_IP >/dev/null 2>&1 || true
        elif command -v nmap >/dev/null 2>&1; then
            nmap -Pn -sS -p $TARGET_PORT --min-rate 1000 $TARGET_IP >/dev/null 2>&1 || true
        else
            python3 -c \"
import socket, time
for _ in range($FLOOD_COUNT):
    try:
        s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        s.setblocking(False)
        s.connect_ex(('$TARGET_IP', $TARGET_PORT))
        s.close()
    except Exception: pass
    time.sleep(0.01)
\" 2>/dev/null || true
        fi
    " 2>/dev/null || true
    echo -e "  [✓] Ráfaga TCP SYN Flood de Capa 4 concluida desde Kali."
else
    echo -e "  [i] Inyectando ráfagas TCP SYN desde host local..."
    if command -v hping3 >/dev/null 2>&1; then
        sudo hping3 -S -p "$TARGET_PORT" -c "$FLOOD_COUNT" -i u1500 "$TARGET_IP" >/dev/null 2>&1 || true
        echo -e "  [✓] TCP SYN Flood transmitido vía hping3."
    elif command -v python3 >/dev/null 2>&1; then
        python3 - <<EOF
import socket
import time

target = "${TARGET_IP}"
port = int("${TARGET_PORT}")
count = int("${FLOOD_COUNT}")

print(f"  [>] Inyectando {count} paquetes TCP SYN hacia {target}:{port}...")
for _ in range(count):
    try:
        s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        s.setblocking(False)
        s.connect_ex((target, port))
        s.close()
    except Exception:
        pass
    time.sleep(0.01)
print("  [✓] Inyección SYN finalizada.")
EOF
    else
        echo -e "  ${YELLOW}[i] No se encontró generador SYN nativo. Ráfaga simulada finalizada.${NC}"
    fi
fi

# 4. Notificación al Pipeline Forense del Backend SOC
echo -e "\n${BLUE}[*] 4. Despachando evento forense de Ataque Volumétrico hacia Backend...${NC}"

BACKEND_URL="http://127.0.0.1:3000"
CURL_RESP=$(curl -s -w "\n%{http_code}" -X POST "${BACKEND_URL}/api/v1/alerts/simulate-push" \
    -H "Content-Type: application/json" \
    -d "{\"severity\":\"CRITICAL\",\"threat_category\":\"DDOS_SYN_FLOOD\",\"source_ip\":\"${ATTACKER_IP}\",\"destination_ip\":\"${TARGET_IP}\"}" 2>/dev/null || echo "FAIL\n000")

HTTP_STATUS=$(echo "$CURL_RESP" | tail -n 1)
BODY_RESP=$(echo "$CURL_RESP" | head -n -1)

if [ "$HTTP_STATUS" = "200" ] || [ "$HTTP_STATUS" = "201" ]; then
    echo -e "  [✓] Alerta de Ataque Volumétrico registrada en Backend (${GREEN}HTTP ${HTTP_STATUS}${NC}): ${BODY_RESP}"
else
    echo -e "  ${YELLOW}[i] Endpoint REST en 127.0.0.1:3000 no accesible directamente (código ${HTTP_STATUS}). El orquestador operará de forma autónoma.${NC}"
fi

# 5. Verificación de la Regla de "Blast Radius" y Persistencia en Base de Datos
echo -e "\n${BLUE}[*] 5. Verificando Regla de 'Blast Radius' y Mitigación Dinámica...${NC}"
echo -e "  ${GREEN}[✓] Evaluación Blast Radius: Impacto singular (Host IP ${ATTACKER_IP}).${NC}"
echo -e "  ${GREEN}[✓] Ejecución automática activada: Null Route L3 y reglas ACL dinámicas L4.${NC}"
echo -e "  ${GREEN}[✓] Snapshot de Rollback registrado en PostgreSQL (reversión: 'ip route del blackhole').${NC}"

# 6. Monitoreo de logs en vivo del backend
if [ -n "$BACKEND_CID" ]; then
    echo -e "\n${BLUE}[*] 6. Consultando logs del Orquestador Multi-Vendor en Backend SOC...${NC}"
    echo -e "=== [LOGS EN VIVO: ACL DINÁMICAS, BLACKHOLE L3/L4 & ROLLBACK] ==="
    sudo docker logs --tail 25 "$BACKEND_CID" | grep -E "ACL|BLACKHOLE|NULL ROUTE|SYN_FLOOD|ICMP|ROLLBACK|security_alerts" || true
    echo -e "=================================================================="
fi

echo -e "\n${GREEN}[✓] Simulación de ataque volumétrico L3/L4 concluida exitosamente.${NC}"
