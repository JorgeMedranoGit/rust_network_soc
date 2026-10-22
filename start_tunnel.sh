#!/usr/bin/env bash
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="$DIR/cloudflared.log"

echo "[*] Verificando servicio systemd cloudflared..."

if ! systemctl is-active --quiet cloudflared; then
    echo "[*] Iniciando servicio cloudflared..."
    sudo systemctl restart cloudflared
    sleep 3
fi

# Extract URL from log
URL=""
for i in {1..20}; do
    if [ -f "$LOG_FILE" ]; then
        URL=$(grep -oE 'https://[a-zA-Z0-9.-]+\.trycloudflare\.com' "$LOG_FILE" | grep -v 'api.trycloudflare.com' | tail -n 1 || true)
        if [ -n "$URL" ]; then
            break
        fi
    fi
    sleep 1
done

if [ -z "$URL" ]; then
    echo "[-] Error: No se encontró URL activa en $LOG_FILE"
    exit 1
fi

echo "================================================================"
echo "    CLOUDFLARE TUNNEL ACTIVO (HTTPS SEGURO PARA PWA / PUSH)    "
echo "================================================================"
echo " URL Base:        $URL"
echo " PWA App:         $URL/pwa/"
echo " Service Worker:  $URL/pwa/firebase-messaging-sw.js"
echo " Healthcheck:     $URL/health"
echo "================================================================"

HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" -m 10 "$URL/health" || true)
if [ "$HTTP_CODE" = "200" ]; then
    echo "[✓] Healthcheck exitoso (HTTP $HTTP_CODE) a través de la red global de Cloudflare."
else
    echo "[!] Estado de respuesta: HTTP $HTTP_CODE"
fi
