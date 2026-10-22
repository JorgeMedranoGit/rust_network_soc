#!/usr/bin/env bash
set -e

# ==============================================================================
# Script de Prueba de Ataques y Notificaciones Push (TIGO SOC)
# ==============================================================================

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KALI_CID=$(sudo docker ps -q -f "name=GNS3.AttackerKali" | head -n 1)

echo "=================================================================="
echo "      GENERADOR DE ATAQUES Y PRUEBA DE PUSH NOTIFICATIONS         "
echo "=================================================================="
echo " Dispositivos suscritos en: https://thomas-institute-tonight-promoting.trycloudflare.com/pwa/"
echo " Tema de Firebase FCM:     /topics/soc_alerts"
echo "=================================================================="

MODE="${1:-real}"

case "$MODE" in
    "real"|"1")
        if [ -z "$KALI_CID" ]; then
            echo "[-] Error: Contenedor Kali no encontrado. Asegúrate de ejecutar ./start.sh primero."
            exit 1
        fi
        echo "[*] Modo: ATAQUE REAL DE RED (End-to-End Pipeline)"
        echo "[*] Lanzando escaneo SYN controlado desde Kali Linux (192.168.100.50) hacia el Nodo ISP 1 (192.168.1.11)..."
        
        # Ejecución del ataque hacia 1 sola IP víctima para generar 1-2 alertas limpias
        sudo docker exec "$KALI_CID" nmap -Pn -sS -p 1-100 -T4 192.168.1.11 --min-rate 2000 > /dev/null 2>&1 &
        ATTACK_PID=$!
        
        echo "[+] Ataque en progreso en segundo plano (PID $ATTACK_PID)..."
        echo "[*] Esperando 3 segundos a que el motor LightGBM infiera anomalías y despache FCM v1..."
        sleep 3
        
        echo ""
        echo "=== [TELEMETRÍA Y ALERTAS GENERADAS EN TIEMPO REAL] ==="
        sudo docker logs --tail 25 tigo_rust_backend | grep -E "ML|FCM v1|PACKET" || true
        echo "======================================================="
        echo "[✓] ¡Alerta disparada! Revisa la pantalla de tu celular y PC para ver la notificación emergente de la PWA."
        ;;

    "webhook"|"2")
        echo "[*] Modo: WEBHOOK ZERO-DATA INMEDIATO (REST API)"
        echo "[*] Inyectando alerta forense sintética y despachando push a Firebase FCM..."
        
        RESP=$(curl -s -X POST http://127.0.0.1:3000/api/v1/alerts/simulate-push \
          -H "Content-Type: application/json" \
          -d '{"severity": "CRITICAL", "threat_category": "PORT_SCAN_ATTACK"}')
        
        echo "Respuesta del backend: $RESP"
        sleep 1
        echo ""
        echo "=== [ESTADO DE ENTREGA FCM v1] ==="
        sudo docker logs --tail 10 tigo_rust_backend | grep -E "FCM v1" | tail -n 2 || true
        echo "=================================="
        echo "[✓] Notificación Push despachada directamente a /topics/soc_alerts."
        ;;

    *)
        echo "Uso: $0 [real|webhook]"
        echo "  real    - Genera tráfico malicioso desde el contenedor Kali (Default)"
        echo "  webhook - Inyección directa por API REST para test instantáneo"
        exit 1
        ;;
esac
