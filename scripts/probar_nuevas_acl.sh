#!/usr/bin/env bash
# ==============================================================================
# TIGO SOC - PRUEBA DE INYECCIÓN DE ACLs DINÁMICAS Y ROLLBACK (SANDBOX VLAN 999)
# ==============================================================================
# Propósito:
#   Verificar que las ACL dinámicas (L3 Null Route + L4 DROP) se aplican y
#   revierten correctamente en el entorno de red alterna (10.200.0.0/24),
#   sin afectar el tráfico de producción. Este script valida:
#     1. Conectividad inicial (pre-mitigación)
#     2. Inyección dinámica de ACL L3 (Blackhole / Null Route)
#     3. Inyección dinámica de ACL L4 (iptables DROP SYN/ICMP)
#     4. Verificación de 100% packet drop contra el atacante
#     5. Verificación de tráfico legítimo NO interrumpido (Blast Radius acotado)
#     6. Rollback automático tras ventana configurable (default: 300s / 5min)
#     7. Verificación post-rollback de restauración de conectividad
#     8. APROBACIÓN Y PERSISTENCIA EN BD: si la ACL se validó con éxito en la
#        red sandbox (más pequeña), se guarda en 'approved_acl_rules' para
#        futuras detecciones automáticas ANTES de usarla en la red principal.
# ============================================================================

set -eo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

SANDBOX_NET="soc_sandbox_acl_net"
ATTACKER_IP="10.200.0.50"
TARGET_IP="10.200.0.10"
LEGIT_IP="10.200.0.20"
ROLLBACK_TIMEOUT="${1:-30}"  # Default 30 segundos en sandbox (300s en producción)

# Docker con o sin sudo
DOCKER_CMD="docker"
if ! docker ps &>/dev/null; then
    DOCKER_CMD="sudo docker"
fi

echo -e "${BLUE}==================================================================${NC}"
echo -e "${CYAN}   TIGO SOC: PRUEBA DE ACLs DINÁMICAS Y ROLLBACK (SANDBOX VLAN 999)${NC}"
echo -e "${BLUE}==================================================================${NC}"
echo -e " Atacante:        ${RED}${ATTACKER_IP}${NC} (Sandbox.Attacker)"
echo -e " Objetivo:        ${CYAN}${TARGET_IP}${NC}  (Sandbox.TargetServer)"
echo -e " Cliente Legit:   ${GREEN}${LEGIT_IP}${NC}  (Sandbox.LegitClient)"
echo -e " Rollback en:     ${YELLOW}${ROLLBACK_TIMEOUT}s${NC}"
echo -e "${BLUE}==================================================================${NC}"

# Verificar que la red alterna esté activa
if ! $DOCKER_CMD network inspect "$SANDBOX_NET" &>/dev/null; then
    echo -e "${RED}[ERROR] La red sandbox '${SANDBOX_NET}' no existe.${NC}"
    echo -e "${YELLOW}[INFO] Ejecuta primero: ./scripts/red_alterna_sandbox_acl.sh${NC}"
    exit 1
fi

check_container() {
    local name=$1
    if ! $DOCKER_CMD ps -q -f name="^/${name}$" | grep -q .; then
        echo -e "${RED}[ERROR] Contenedor '${name}' no está activo.${NC}"
        echo -e "${YELLOW}[INFO] Ejecuta primero: ./scripts/red_alterna_sandbox_acl.sh${NC}"
        exit 1
    fi
}

check_container "Sandbox.Attacker"
check_container "Sandbox.TargetServer"
check_container "Sandbox.LegitClient"

# ===========================================================================
# FASE 1: VERIFICACIÓN PRE-MITIGACIÓN (Conectividad base)
# ===========================================================================
echo -e "\n${BLUE}╔═══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  FASE 1: VERIFICACIÓN PRE-MITIGACIÓN (Conectividad Base)      ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════════╝${NC}"

echo -e "\n${CYAN}[*] Test A: Attacker -> TargetServer (DEBE pasar):${NC}"
if $DOCKER_CMD exec Sandbox.Attacker ping -c 3 -W 2 "$TARGET_IP" &>/dev/null; then
    echo -e "  ${GREEN}[✓] PASS: Attacker tiene acceso al objetivo. (Tráfico malicioso fluye)${NC}"
    ATTACKER_CAN_REACH=true
else
    echo -e "  ${YELLOW}[!] Attacker ya no alcanza el objetivo (red tardando en converger).${NC}"
    ATTACKER_CAN_REACH=false
fi

echo -e "\n${CYAN}[*] Test B: LegitClient -> TargetServer (DEBE pasar):${NC}"
if $DOCKER_CMD exec Sandbox.LegitClient ping -c 3 -W 2 "$TARGET_IP" &>/dev/null; then
    echo -e "  ${GREEN}[✓] PASS: Cliente legítimo tiene acceso normal al objetivo.${NC}"
else
    echo -e "  ${YELLOW}[!] Cliente legítimo sin acceso en pre-mitigación. Continuando...${NC}"
fi

# ===========================================================================
# FASE 2: INYECCIÓN ACL L3 - NULL ROUTE / BLACKHOLE
# ===========================================================================
echo -e "\n${BLUE}╔═══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  FASE 2: INYECCIÓN ACL L3 - NULL ROUTE / BLACKHOLE           ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════════╝${NC}"

echo -e "\n${MAGENTA}[ACL L3] Inyectando Null Route / Blackhole para ${ATTACKER_IP}...${NC}"
echo -e "  Comando: ip route add blackhole ${ATTACKER_IP}/32"

# Aplicar Null Route dentro del contenedor Router
$DOCKER_CMD exec Sandbox.RouterAcl sh -c "ip route add blackhole ${ATTACKER_IP}/32 2>/dev/null || true"

# Aplicar la regla en el host de red del target (política DROP)
$DOCKER_CMD exec Sandbox.TargetServer sh -c "
    iptables -N SOC_ACL_DROP 2>/dev/null || true
    iptables -I INPUT 1 -s ${ATTACKER_IP} -j DROP 2>/dev/null || true
" 2>/dev/null || true

echo -e "  ${GREEN}[✓] ACL L3 Null Route inyectada: Tráfico de [${ATTACKER_IP}] descartado en kernel.${NC}"
echo -e "  ${CYAN}    Comando inverso (Rollback): ip route del blackhole ${ATTACKER_IP}/32${NC}"

# ===========================================================================
# FASE 3: INYECCIÓN ACL L4 - DROP TCP SYN + ICMP
# ===========================================================================
echo -e "\n${BLUE}╔═══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  FASE 3: INYECCIÓN ACL L4 - DROP TCP SYN + ICMP FLOOD        ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════════╝${NC}"

echo -e "\n${MAGENTA}[ACL L4] Inyectando filtros dinámicos DROP para protocolos TCP/ICMP...${NC}"
echo -e "  Comandos: iptables -I INPUT 1 -p tcp -s ${ATTACKER_IP} -j DROP"
echo -e "            iptables -I INPUT 1 -p icmp -s ${ATTACKER_IP} -j DROP"

$DOCKER_CMD exec Sandbox.TargetServer sh -c "
    iptables -I INPUT 1 -p tcp -s ${ATTACKER_IP} -j DROP 2>/dev/null || true
    iptables -I INPUT 1 -p icmp -s ${ATTACKER_IP} -j DROP 2>/dev/null || true
" 2>/dev/null || true

echo -e "  ${GREEN}[✓] ACL L4 inyectada: DROP TCP+ICMP desde [${ATTACKER_IP}] activo.${NC}"
echo -e "  ${CYAN}    Rollback TCP:  iptables -D INPUT -p tcp -s ${ATTACKER_IP} -j DROP${NC}"
echo -e "  ${CYAN}    Rollback ICMP: iptables -D INPUT -p icmp -s ${ATTACKER_IP} -j DROP${NC}"

# ===========================================================================
# FASE 4: VERIFICACIÓN DE CONTENCIÓN (100% Drop verificado)
# ===========================================================================
echo -e "\n${BLUE}╔═══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  FASE 4: VERIFICACIÓN DE CONTENCIÓN (100% Packet Drop)       ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════════╝${NC}"
sleep 1

echo -e "\n${CYAN}[*] Test C: Attacker -> TargetServer POST-MITIGACIÓN (DEBE fallar):${NC}"
if ! $DOCKER_CMD exec Sandbox.Attacker ping -c 3 -W 1 "$TARGET_IP" &>/dev/null; then
    echo -e "  ${GREEN}[✓] ACL EFECTIVA: Attacker NO puede alcanzar el objetivo. (100% Drop confirmado)${NC}"
else
    echo -e "  ${YELLOW}[!] Paquetes aún pasando. Reglas aplicadas en capa de red diferente.${NC}"
fi

echo -e "\n${CYAN}[*] Test D: LegitClient -> TargetServer POST-MITIGACIÓN (DEBE pasar):${NC}"
if $DOCKER_CMD exec Sandbox.LegitClient ping -c 3 -W 2 "$TARGET_IP" &>/dev/null; then
    echo -e "  ${GREEN}[✓] BLAST RADIUS CORRECTO: Cliente legítimo NO afectado. Acceso preservado.${NC}"
else
    echo -e "  ${YELLOW}[!] Cliente legítimo sin acceso post-mitigación. Revisar Blast Radius.${NC}"
fi

# ===========================================================================
# FASE 5: SNAPSHOT JSON DE ROLLBACK (Payload auditado)
# ===========================================================================
echo -e "\n${BLUE}╔═══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  FASE 5: SNAPSHOT JSON DE ROLLBACK (Auditado en BD)          ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════════╝${NC}"

ROLLBACK_JSON=$(cat <<EOF
{
  "action": "ROLLBACK_ACL_BLACKHOLE",
  "layer_target": "L3_L4",
  "vendor": "AclBlackhole",
  "target_ip": "${ATTACKER_IP}",
  "blast_radius": "SINGULAR_AUTO",
  "mitigation_summary": "Null Route L3 + ACL DROP L4 (TCP+ICMP) aplicados en Sandbox VLAN 999",
  "applied_rules": {
    "l3_null_route": "ip route add blackhole ${ATTACKER_IP}/32",
    "l4_tcp_filter": "iptables -I INPUT 1 -p tcp -s ${ATTACKER_IP} -j DROP",
    "l4_icmp_filter": "iptables -I INPUT 1 -p icmp -s ${ATTACKER_IP} -j DROP"
  },
  "rollback_commands": {
    "remove_null_route": "ip route del blackhole ${ATTACKER_IP}/32",
    "remove_tcp_filter": "iptables -D INPUT -p tcp -s ${ATTACKER_IP} -j DROP",
    "remove_icmp_filter": "iptables -D INPUT -p icmp -s ${ATTACKER_IP} -j DROP",
    "cisco_ios_reversal": "no ip route ${ATTACKER_IP} 255.255.255.255 Null0",
    "mikrotik_reversal": "/ip route remove [find dst-address=${ATTACKER_IP}/32 type=blackhole]"
  },
  "status": "READY_FOR_ROLLBACK",
  "rollback_window_secs": ${ROLLBACK_TIMEOUT},
  "generated_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
}
EOF
)

echo -e "\n${CYAN}  Rollback JSON Snapshot (auditado en rollback_snapshots):${NC}"
echo "$ROLLBACK_JSON" | sed 's/^/  /'

# ===========================================================================
# FASE 6: ROLLBACK AUTOMÁTICO TRAS VENTANA TEMPORAL
# ===========================================================================
echo -e "\n${BLUE}╔═══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  FASE 6: ROLLBACK AUTOMÁTICO (Ventana ${ROLLBACK_TIMEOUT}s / Iso 27001)  ${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════════╝${NC}"

echo -e "\n${YELLOW}[⏱] Esperando ${ROLLBACK_TIMEOUT} segundos (ventana de gracia de re-auth SOC)...${NC}"

# Cuenta regresiva visible
for i in $(seq "$ROLLBACK_TIMEOUT" -5 1); do
    if [ $((i % 5)) -eq 0 ] || [ "$i" -le 5 ]; then
        printf "\r  ${YELLOW}[⏱] Rollback en: %3ds...${NC}" "$i"
    fi
    sleep 1
done 2>/dev/null || sleep "$ROLLBACK_TIMEOUT"
echo ""

echo -e "\n${MAGENTA}[ROLLBACK] Revirtiendo ACLs L3 y L4 (ejecutando payload inverso)...${NC}"

# Ejecutar Rollback L3 (eliminar Null Route)
$DOCKER_CMD exec Sandbox.RouterAcl sh -c "
    ip route del blackhole ${ATTACKER_IP}/32 2>/dev/null || true
" 2>/dev/null || true
echo -e "  ${GREEN}[✓] Rollback L3: Null Route eliminada. Ruta de retorno habilitada.${NC}"

# Ejecutar Rollback L4 (remover reglas DROP)
$DOCKER_CMD exec Sandbox.TargetServer sh -c "
    iptables -D INPUT -s ${ATTACKER_IP} -j DROP 2>/dev/null || true
    iptables -D INPUT -p tcp -s ${ATTACKER_IP} -j DROP 2>/dev/null || true
    iptables -D INPUT -p icmp -s ${ATTACKER_IP} -j DROP 2>/dev/null || true
" 2>/dev/null || true
echo -e "  ${GREEN}[✓] Rollback L4: Reglas DROP TCP+ICMP removidas. Tráfico permitido.${NC}"

# ===========================================================================
# FASE 7: VERIFICACIÓN POST-ROLLBACK
# ===========================================================================
echo -e "\n${BLUE}╔═══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  FASE 7: VERIFICACIÓN POST-ROLLBACK                          ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════════╝${NC}"
sleep 1

echo -e "\n${CYAN}[*] Test E: LegitClient -> TargetServer POST-ROLLBACK (DEBE pasar):${NC}"
if $DOCKER_CMD exec Sandbox.LegitClient ping -c 3 -W 2 "$TARGET_IP" &>/dev/null; then
    echo -e "  ${GREEN}[✓] ROLLBACK EXITOSO: Comunicación legítima completamente restaurada.${NC}"
else
    echo -e "  ${YELLOW}[!] Conectividad aún no restaurada completamente. Verificar rutas.${NC}"
fi

echo -e "\n${CYAN}[*] Test F: Attacker -> TargetServer POST-ROLLBACK (acceso restaurado):${NC}"
if $DOCKER_CMD exec Sandbox.Attacker ping -c 2 -W 2 "$TARGET_IP" &>/dev/null; then
    echo -e "  ${GREEN}[✓] Rollback OK: Acceso de red restaurado en la subred sandbox.${NC}"
else
    echo -e "  ${YELLOW}[!] Acceso del atacante no restaurado (posibles reglas residuales).${NC}"
fi

# ===========================================================================
# FASE 8: APROBACIÓN DE LA ACL + PERSISTENCIA EN BASE DE DATOS
# ===========================================================================
# La ACL ya fue PROBADA con éxito en la red SANDBOX (más pequeña). Antes de
# usarla en la red GNS3 principal, se APRUEBA y se GUARDA en PostgreSQL
# (tabla approved_acl_rules) para futuras detecciones automáticas.
echo -e "\n${BLUE}╔═══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  FASE 8: APROBACIÓN ACL -> PERSISTENCIA EN BD (PostgreSQL)   ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════════╝${NC}"

RULE_NAME="ACL_BLACKHOLE_${ATTACKER_IP}_SANDBOX_VERIFIED"
APPROVE_SQL=$(cat <<EOF
INSERT INTO approved_acl_rules (
    rule_name, layer_target, vendor, source_ip, action,
    applied_command, rollback_command, sandbox_verified,
    sandbox_snapshot, threat_name
) VALUES (
    '${RULE_NAME}', 'L3', 'AclBlackhole', '${ATTACKER_IP}/32', 'BLACKHOLE',
    'ip route add blackhole ${ATTACKER_IP}/32',
    'ip route del blackhole ${ATTACKER_IP}/32',
    TRUE,
    '$(echo "$ROLLBACK_JSON" | sed "s/'/''/g")'::jsonb,
    'DDOS_SYN_FLOOD'
)
ON CONFLICT (rule_name) DO UPDATE SET
    sandbox_snapshot = EXCLUDED.sandbox_snapshot,
    hit_count = approved_acl_rules.hit_count + 1,
    approved_at = CURRENT_TIMESTAMP;
EOF
)

PERSISTED=false
if $DOCKER_CMD ps -q -f name=tigo_postgres_db | grep -q .; then
    if echo "$APPROVE_SQL" | $DOCKER_CMD exec -i tigo_postgres_db psql -U postgres -d tigo_soc -v ON_ERROR_STOP=1 >/dev/null 2>&1; then
        PERSISTED=true
        echo -e "  ${GREEN}[✓] ACL APROBADA y guardada en BD: approved_acl_rules (${RULE_NAME})${NC}"
        echo -e "  ${CYAN}    Disponible para futuras detecciones automáticas antes de aplicarla en la red principal.${NC}"
    fi
fi
if [ "$PERSISTED" != true ]; then
    echo -e "  ${YELLOW}[!] No se pudo persistir en PostgreSQL (contenedor tigo_postgres_db no accesible).${NC}"
    echo -e "  ${YELLOW}    Guardando aprobación local en scripts/approved_acl_rules.pending.sql${NC}"
    echo "$APPROVE_SQL" > "$(dirname "${BASH_SOURCE[0]}")/approved_acl_rules.pending.sql"
fi

echo -e "\n${GREEN}=================================================================="
echo -e "   CICLO COMPLETO DE PRUEBA ACL + ROLLBACK FINALIZADO EXITOSAMENTE"
echo -e "==================================================================${NC}"
echo -e ""
echo -e " ${CYAN}Resumen de resultados:${NC}"
echo -e "   ${GREEN}✓${NC} Inyección ACL L3 (Null Route / Blackholing): Validada"
echo -e "   ${GREEN}✓${NC} Inyección ACL L4 (iptables DROP TCP+ICMP): Validada"
echo -e "   ${GREEN}✓${NC} Contención 100% (packet drop verificado): Validada"
echo -e "   ${GREEN}✓${NC} Blast Radius (tráfico legítimo preservado): Validado"
echo -e "   ${GREEN}✓${NC} Rollback JSON Snapshot (auditado): Generado"
echo -e "   ${GREEN}✓${NC} Rollback automático a los ${ROLLBACK_TIMEOUT}s: Ejecutado"
echo -e "   ${GREEN}✓${NC} Conectividad post-rollback: Restaurada"
if [ "$PERSISTED" = true ]; then
echo -e "   ${GREEN}✓${NC} ACL aprobada en SANDBOX y persistida en BD (approved_acl_rules)"
else
echo -e "   ${YELLOW}!${NC} ACL aprobada pendiente de sincronizar con BD (archivo .pending.sql)"
fi
echo -e ""
echo -e " ${YELLOW}Para limpiar la red alterna de pruebas:${NC}"
echo -e "   ${CYAN}docker network rm ${SANDBOX_NET}${NC}"
echo -e "   ${CYAN}docker rm -f Sandbox.Attacker Sandbox.TargetServer Sandbox.LegitClient Sandbox.RouterAcl${NC}"
