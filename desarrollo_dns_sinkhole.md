# Contexto del Sistema: Interceptor DNS Sinkholing (Capa 7), Rollback y Pruebas
El SOC requiere mitigar ataques de Capa 7 (ej. exfiltración DNS o balizas C2) interceptando resoluciones hacia dominios maliciosos y redirigiéndolas a un Sinkhole seguro.

## Misión de Desarrollo:
1. **Módulo DNS Sinkhole:** Crea `src/mitigation/dns_sinkhole.rs`. Debe simular la intercepción de peticiones UDP 53 a dominios en lista negra y devolver la IP del Sinkhole.
2. **Patrón Multi-Vendor:** Añade la variante `DnsSinkhole` al enum `Vendor` en `src/mitigation/mod.rs` e intégralo en el orquestador.
3. **Evaluación de Blast Radius (Manual):** Alterar el DNS afecta a toda la red. Ajusta el Orquestador para que, si la mitigación es Capa 7, NO se ejecute automáticamente. Debe registrase en `security_alerts` y encolarse en `execution_queue` con estado "PENDING_MANUAL". Debe generar su JSON de Rollback.
4. **Script de Ataque Capa 7:** Crea `scripts/ataques/simulacion_l7_dns_c2.sh` usando `dig` o `nslookup` en un bucle hacia un dominio simulado (ej. evil-c2.com) para disparar esta alerta específica.

## Restricción Crítica
NO EJECUTES `cargo test` ni intentes compilar el código.
