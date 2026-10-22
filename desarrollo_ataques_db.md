# Contexto: Auditoría, Rollback y Control de Impacto (Blast Radius)
El SOC debe someterse a pruebas de estrés mediante scripts de ataque por capa (L2, L3, L4, L7). Además, la orquestación de mitigaciones debe integrarse con el esquema de base de datos BCNF proporcionado.

## Reglas de Negocio Estrictas:
1. **Registro de Alertas:** Toda anomalía detectada por LightGBM debe insertarse en `security_alerts` con su `anomaly_score` y clasificarse como alerta CRÍTICA.
2. **Evaluación de Radio de Impacto (Blast Radius):** Antes de mitigar, el orquestador debe evaluar a cuántos clientes afectará:
   - **Impacto Singular (Auto):** Si afecta a un solo host (ej. L2 RADIUS MAC aislando un puerto, o L4 aislando un puerto de una IP específica). Pasa directo a ejecución.
   - **Impacto Masivo (Manual):** Si afecta a múltiples clientes (ej. L3 Blackholing de un Gateway/Subred entera, o L7 DNS Sinkholing que afecta a toda la red). La acción NO se ejecuta. Se inserta en `execution_queue` con el estado `PENDING_MANUAL`.
3. **Rollback Obligatorio:** Toda mitigación automática que se ejecute con éxito debe guardar su estado previo y el comando inverso en la tabla `rollback_snapshots` en formato JSON (ej. `{"command": "iptables -D INPUT -s IP -j DROP"}` o `{"radius_action": "Re-Authenticate"}`).
4. **Scripts de Ataque:** Por cada capa, se debe generar un script Bash en la carpeta `scripts/ataques/` que ejecute la ofensiva desde Kali Linux hacia la topología GNS3.
