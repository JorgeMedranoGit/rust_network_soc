# Contexto del Sistema: ACLs Dinámicas y Blackholing (Capas 3 y 4), Rollback y Pruebas
El SOC requiere implementar contención perimetral e interna mediante la inyección dinámica de reglas ACL y técnicas de Blackholing en enrutadores para mitigar ataques en Capas 3 y 4 (ej. SYN Flood, ICMP Flood, ataques volumétricos).

## Misión de Desarrollo:
1. **Módulo ACL y Blackhole:** Crea `src/mitigation/acl_blackhole.rs`. Debe gestionar la aplicación de reglas de filtrado IP, control de puertos y redirección a Null Routes (Blackholing).
2. **Patrón Multi-Vendor:** Añade la variante `AdvancedACL` (o `AclBlackhole`) al enum `Vendor` en `src/mitigation/mod.rs` e intégralo en el orquestador implementando el trait correspondiente.
3. **Evaluación de Blast Radius y Rollback:** 
   - El blackholing de una IP de host específico tiene impacto singular (ejecución automática).
   - Registra el evento en `security_alerts` antes de mitigar.
   - Genera el JSON de Rollback con el comando inverso exacto (ej. eliminar la ruta nula o regla de firewall).
4. **Script de Ataque Capas 3 y 4:** Crea `scripts/ataques/simulacion_l3_l4_flood.sh` utilizando herramientas como `hping3`, `nmap` o generadores de tráfico para simular un SYN Flood (Capa 4) y un ICMP Flood masivo (Capa 3).

## Restricción Crítica
NO EJECUTES `cargo test` ni intentes compilar el código bajo ninguna circunstancia.
