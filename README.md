# 🛡️ Tigo SOC — Plataforma de Operaciones de Seguridad con ML y Gemelos Digitales

Proyecto de titulación (Jorge Medrano) — **Tesis**: CAPITULO I Y II CORREGIDO - MEDRANO CHACOLLA JORGE ESTEBAN.pdf
Diseño de interfaz de referencia: `MockupsFiles/Final/MOCKUPS - JORGE MEDRANO.pdf` (láminas 1–10, temas claro/oscuro).

Plataforma SOC para una ISP: detección de amenazas con **LightGBM**, mitigación automática
(ACL blackhole / DNS sinkhole / RADIUS CoA), gemelo digital de red en **GNS3** con dos laboratorios,
y dashboard en tiempo real.

## 🏗️ Arquitectura

| Módulo | Tecnología | Descripción |
|---|---|---|
| `tigo_soc_backend/` | Rust + Axum + SQLx + PostgreSQL | API REST DDD (`/api/v1/*`): auth JWT+TOTP, alertas enriquecidas, métricas, inventario, RBAC, ACLs aprobadas |
| `tico_soc_frontend/` → `tigo_soc_frontend/` | Rust + Leptos 0.6 (CSR/WASM) + gloo-net + Three.js | Dashboard SPA: láminas de Alertas, Métricas, Blast Radius, Topología 3D, RBAC, ML Metrics y Sandbox ACL. Sin mocks: todo consume la API vía polling (`create_effect` + `spawn_local`) |
| `ModelTrainedLightGBM/` | Python + LightGBM | Motor NDR: modelos entrenados (`modelo_ndr_v1/v2_high_prec.txt`), reporte k-fold |
| `bd/init.sql` | PostgreSQL | Esquema: usuarios/roles, threat_types, telemetry, `approved_acl_rules` |
| GNS3/Docker lab | Shell scripts | Dos redes (ver abajo) |
| `frontend/` | output servido | Build generado por Trunk (regenerable, no editar a mano) |

## 🖧 Laboratorio GNS3 — dos redes

1. **Red principal** (`build_topology.sh` → proyecto `Tigo_SOC_ISP_Expanded`, fallback Docker `soc_isp_net`):
   topología ISP expandida (Kali 192.168.100.50 + víctimas 192.168.1.x). Aquí se lanzan los ataques
   (`lanzar_ataque.sh`) y se captura **todo el tráfico** que alimenta el pipeline ML/SOC.
2. **Red sandbox ACL** (proyecto `Tigo_SOC_Sandbox_ACL` / `scripts/red_alterna_sandbox_acl.sh`, VLAN 999 `10.200.0.0/24`):
   más pequeña (RouterAcl + TargetServer + LegitClient + Attacker). Diseñada para **probar ACLs antes
   de aplicarlas en la red principal**. Una ACL validada al 100% (contención + blast radius + rollback)
   se **aprueba y persiste en PostgreSQL** (`approved_acl_rules`, ver `scripts/probar_nuevas_acl.sh` FASE 8)
   para futuras detecciones automáticas.

## 🚀 Puesta en marcha

```bash
cp .env.example .env            # ajusta credenciales (Postgres, JWT, Firebase si aplica)
./startAll.sh                   # orquestador one-shot: BD -> backend -> build trunk -> lab GNS3/Docker
# o por partes:
./start.sh                      # mismo flujo con opciones interactivas
./build_topology.sh             # despliega AMBAS redes GNS3 (principal + sandbox)
./scripts/probar_nuevas_acl.sh  # ciclo completo: proponer ACL -> validar en sandbox -> aprobar -> guardar en BD
./allow_ip.sh add <IP>          # zero-trust: autorizar acceso a la red principal
```

Frontend dev: `cd tigo_soc_frontend && trunk serve` (puerto 8080; el backend debe habilitar CORS hacia ese origen).
Backend: `cd tigo_soc_backend && cargo run` (puerto 3000). Verificación WASM: `cargo check --target wasm32-unknown-unknown`.

## 🔌 Endpoints principales (`/api/v1`)

`POST /auth/login` (username, password, totpCode) · `POST /auth/logout` · `POST /auth/reauth` ·
`GET /alerts` · `GET /metrics` · `GET /telemetry/metrics` · `GET /inventory/nodes` ·
`GET /catalogs/device-types` · `GET /users` (CRUD RBAC) · `GET /ml/model-info` · `GET/POST /acls/*` (sandbox y aprobadas)

## 🎨 UI

Paleta institucional según mockups: azules (#1E5AA8), verdes, rojos, ámbar e índigo; tema oscuro/claro.
Láminas implementadas: 1 Login · 2 Dashboard · 3 Tabla de alertas · 4 Topología 3D · 5 Métricas ML ·
6 Blast Radius · 7 Gráficas de tráfico · 8 RBAC · 9 Flujo en vivo · 10 Sandbox ACL.

## ⚠️ Notas de seguridad

- `.gitignore` excluye `.env`, secretos, `target/` y artefactos WASM/JS de build.
- La apiKey de Firebase en `frontend/` es pública por diseño (web key); restrínzela en Firebase Console.
- Credenciales por defecto de Postgres (`postgres:postgres`) solo para laboratorio local; cámbialas en producción.

## 📄 Licencia
Ver `LICENSE`.
