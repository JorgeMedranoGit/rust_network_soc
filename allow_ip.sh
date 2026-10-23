#!/usr/bin/env bash
# ==============================================================================
# Script de Gestión Rápida de IPs en Tiempo Real (TIGO SOC Zero Trust)
# Permite agregar, quitar y listar IPs autorizadas simultáneamente en:
# 1. Firewall del Kernel (iptables DOCKER-USER)
# 2. Perímetro Zero Trust de la PWA (frontend/app.js e index.html)
# 3. Persistencia de arranque (start.sh)
# ==============================================================================
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACTION="${1:-}"
TARGET_IP="${2:-}"

mostrar_ayuda() {
    echo -e "\e[1;34m=== GESTIÓN DE ACCESO ZERO TRUST (TIGO SOC) ===\e[0m"
    echo "Uso:"
    echo "  $0 add <IP>     -> Autorizar IP en tiempo real (iptables + PWA)"
    echo "  $0 del <IP>     -> Revocar acceso de IP en tiempo real"
    echo "  $0 list         -> Listar IPs actualmente autorizadas"
    echo ""
    echo "Ejemplos:"
    echo "  $0 add 200.58.87.90"
    echo "  $0 del 200.58.87.90"
    echo "  $0 list"
}

if [ -z "$ACTION" ]; then
    mostrar_ayuda
    exit 1
fi

case "$ACTION" in
    "add")
        if [ -z "$TARGET_IP" ]; then
            echo -e "\e[1;31m[ERROR] Debe especificar la IP a autorizar. Ej: $0 add 200.58.87.90\e[0m"
            exit 1
        fi
        
        echo -e "\e[1;34m[*] Autorizando IP [$TARGET_IP] en tiempo real...\e[0m"
        
        # 1. iptables (Capa Kernel)
        if ! sudo iptables -C DOCKER-USER -s "$TARGET_IP" -p tcp --dport 3000 -j ACCEPT 2>/dev/null; then
            sudo iptables -I DOCKER-USER 1 -s "$TARGET_IP" -p tcp --dport 3000 -j ACCEPT
            echo -e "\e[1;32m[✓] iptables: Regla ACCEPT inyectada en DOCKER-USER para puerto 3000.\e[0m"
        else
            echo -e "\e[1;33m[!] iptables: La IP ya estaba autorizada en DOCKER-USER.\e[0m"
        fi

        # Autorización también en la Red GNS3 PRINCIPAL (soc_isp_net, segmento
        # de ataques/tráfico 192.168.100.0/24 + 192.168.1.0/24). La red SANDBOX
        # ACL (soc_sandbox_acl_net) no requiere regla: es segregada por diseño.
        if ! sudo iptables -C DOCKER-USER -s "$TARGET_IP" -d 192.168.0.0/16 -j ACCEPT 2>/dev/null; then
            sudo iptables -I DOCKER-USER 2 -s "$TARGET_IP" -d 192.168.0.0/16 -j ACCEPT
            echo -e "\e[1;32m[✓] iptables: IP autorizada en la red principal soc_isp_net (192.168.0.0/16).\e[0m"
        fi
        
        # 2. Modificar frontend/app.js, frontend/index.html y start.sh mediante Python
        python3 - <<EOF
import re, time

ip = "$TARGET_IP"
ts = int(time.time())

# Modificar app.js
app_path = "$DIR/frontend/app.js"
with open(app_path, "r", encoding="utf-8") as f:
    app_code = f.read()

if ip not in app_code:
    pattern = r"(const ALLOWED_SOC_IPS = \[\n)"
    replacement = r"\g<1>  '" + ip + r"',\n"
    new_app = re.sub(pattern, replacement, app_code, count=1)
    with open(app_path, "w", encoding="utf-8") as f:
        f.write(new_app)
    print(f"[✓] PWA app.js: IP {ip} agregada a ALLOWED_SOC_IPS.")
else:
    print(f"[!] PWA app.js: IP {ip} ya existía en la lista.")

# Modificar index.html (cache buster para forzar actualización en clientes)
index_path = "$DIR/frontend/index.html"
with open(index_path, "r", encoding="utf-8") as f:
    html = f.read()

html = re.sub(r'src="/pwa/app\.js\?v=[^"]+"', f'src="/pwa/app.js?v={ts}"', html)
with open(index_path, "w", encoding="utf-8") as f:
    f.write(html)
print(f"[✓] PWA index.html: Cache buster actualizado (?v={ts}).")

# Modificar start.sh (persistencia)
start_path = "$DIR/start.sh"
with open(start_path, "r", encoding="utf-8") as f:
    start_code = f.read()

m = re.search(r'TRUSTED_SOC_IPS="([^"]+)"', start_code)
if m and ip not in m.group(1):
    current = m.group(1)
    updated = f"{ip},{current}"
    start_code = start_code.replace(f'TRUSTED_SOC_IPS="{current}"', f'TRUSTED_SOC_IPS="{updated}"')
    with open(start_path, "w", encoding="utf-8") as f:
        f.write(start_code)
    print(f"[✓] start.sh: IP {ip} agregada a TRUSTED_SOC_IPS para persistencia.")
EOF
        echo -e "\e[1;32m[LISTO] IP [$TARGET_IP] 100% ACTIVA Y AUTORIZADA.\e[0m"
        ;;

    "del"|"remove"|"rm")
        if [ -z "$TARGET_IP" ]; then
            echo -e "\e[1;31m[ERROR] Debe especificar la IP a revocar. Ej: $0 del 200.58.87.90\e[0m"
            exit 1
        fi
        
        echo -e "\e[1;34m[*] Revocando acceso a IP [$TARGET_IP] en tiempo real...\e[0m"
        
        # 1. iptables (Capa Kernel)
        while sudo iptables -D DOCKER-USER -s "$TARGET_IP" -p tcp --dport 3000 -j ACCEPT 2>/dev/null; do
            :
        done
        while sudo iptables -D DOCKER-USER -s "$TARGET_IP" -j ACCEPT 2>/dev/null; do
            :
        done
        # Revocar también la autorización de la Red GNS3 PRINCIPAL (soc_isp_net)
        while sudo iptables -D DOCKER-USER -s "$TARGET_IP" -d 192.168.0.0/16 -j ACCEPT 2>/dev/null; do
            :
        done
        echo -e "\e[1;32m[✓] iptables: Reglas de IP [$TARGET_IP] eliminadas de DOCKER-USER (backend + red principal).\e[0m"
        
        # 2. Modificar frontend/app.js, frontend/index.html y start.sh mediante Python
        python3 - <<EOF
import re, time

ip = "$TARGET_IP"
ts = int(time.time())

# Modificar app.js
app_path = "$DIR/frontend/app.js"
with open(app_path, "r", encoding="utf-8") as f:
    lines = f.readlines()

new_lines = [l for l in lines if f"'{ip}'" not in l and f'"{ip}"' not in l]
with open(app_path, "w", encoding="utf-8") as f:
    f.writelines(new_lines)
print(f"[✓] PWA app.js: IP {ip} eliminada de ALLOWED_SOC_IPS.")

# Modificar index.html (cache buster)
index_path = "$DIR/frontend/index.html"
with open(index_path, "r", encoding="utf-8") as f:
    html = f.read()

html = re.sub(r'src="/pwa/app\.js\?v=[^"]+"', f'src="/pwa/app.js?v={ts}"', html)
with open(index_path, "w", encoding="utf-8") as f:
    f.write(html)
print(f"[✓] PWA index.html: Cache buster actualizado (?v={ts}).")

# Modificar start.sh (persistencia)
start_path = "$DIR/start.sh"
with open(start_path, "r", encoding="utf-8") as f:
    start_code = f.read()

m = re.search(r'TRUSTED_SOC_IPS="([^"]+)"', start_code)
if m and ip in m.group(1):
    current = m.group(1).split(",")
    updated = [x for x in current if x != ip]
    start_code = start_code.replace(f'TRUSTED_SOC_IPS="{m.group(1)}"', f'TRUSTED_SOC_IPS="{",".join(updated)}"')
    with open(start_path, "w", encoding="utf-8") as f:
        f.write(start_code)
    print(f"[✓] start.sh: IP {ip} retirada de TRUSTED_SOC_IPS.")
EOF
        echo -e "\e[1;32m[LISTO] IP [$TARGET_IP] REVOCADA Y BLOQUEADA.\e[0m"
        ;;

    "list"|"ls")
        echo -e "\e[1;34m=== ESTADO ACTUAL DEL PERÍMETRO ZERO TRUST ===\e[0m"
        echo -e "\e[1;33m[1. iptables - Cadena DOCKER-USER (Puerto 3000)]:\e[0m"
        sudo iptables -L DOCKER-USER -n -v | grep -E "ACCEPT|DROP|target" || true
        echo ""
        echo -e "\e[1;33m[2. PWA Frontend (frontend/app.js - ALLOWED_SOC_IPS)]:\e[0m"
        grep -A 8 "ALLOWED_SOC_IPS" "$DIR/frontend/app.js"
        echo ""
        echo -e "\e[1;33m[3. Modo Contingencia Inmediata (Bypass Temporal)]:\e[0m"
        echo "Si necesitas dar acceso inmediato sin configurar IP, comparte este enlace:"
        TUNNEL_URL=$(grep -oE 'https://[a-zA-Z0-9.-]+\.trycloudflare\.com' "$DIR/cloudflared.log" | grep -v 'api.trycloudflare.com' | tail -n 1 || true)
        if [ -n "$TUNNEL_URL" ]; then
            echo "-> $TUNNEL_URL/pwa/?bypass_zt=true"
        else
            echo "-> http://localhost:3000/pwa/?bypass_zt=true"
        fi
        ;;

    *)
        mostrar_ayuda
        exit 1
        ;;
esac
