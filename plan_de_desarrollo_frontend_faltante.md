# Plan de Desarrollo Frontend y Consolidación SOC (Tigo Business Bolivia)

Este documento detalla los módulos de interfaz de usuario (Frontend) implementados para garantizar que la plataforma web responda de manera absoluta a los requerimientos de la tesis y ofrezca una experiencia de operador SOC de nivel industrial (End-to-End).

---

## 1. Módulos Implementados en el Frontend

Para asegurar la coherencia completa entre el backend en Rust (Axum + PostgreSQL BCNF) y la interfaz visual, se implementaron las siguientes vistas interactivas en Rust Leptos (WASM):

### A. Vista de Autenticación y Login (`LoginView`) — `src/components/login_view.rs`
* **Propósito:** Proveer un formulario de acceso seguro con estética cyberpunk/Catppuccin Mocha que consuma las rutas de autenticación del backend (`/api/v1/auth/login`).
* **Características Implementadas:**
  * Campos interactivos para usuario y contraseña con feedback visual reactivo.
  * Formulario integrado para **Creación de Administrador Inicial** si no existen cuentas previas en el sistema.
  * Emisión y gestión del token de sesión (`session_token`), expiración de 24h y marcas de tiempo para políticas de re-autenticación.
  * Modal integrado para **Re-autenticación por Contraseña** cuando el timeout de inactividad (`critical_action_timeout_secs`) expira.
  * Background SVG interactivo con gradientes orbitales y animación de radar/escudo SOC.

### B. Módulo de Topología Gráfica 3D de Dispositivos (`Topology3D`) — `src/components/topology_3d.rs`
* **Propósito:** Visualizar de forma gráfica en 3D la topología de la red y los nodos descubiertos pasivamente por el motor OUI (`asset_discovery.rs`) en el laboratorio GNS3 / Tigo Business Bolivia.
* **Características Implementadas:**
  * Motor WebGL 3D basado en Three.js inyectado limpiamente vía DOM (`web-sys` + `wasm-bindgen`).
  * 12 nodos modelados: *pfSense_Gateway*, *Router_Core_L3*, *AttackerKali*, y 9 estaciones de trabajo *VictimeAlpine1* a *VictimeAlpine9*.
  * Geometrías semánticas diferenciadas por dispositivo: Octaedro (Firewall), Dodecaedro (Router L3), Cubo (Workstations Víctimas), Cono con pulso luminoso (Atacante Kali).
  * Enlaces y aristas de topología interconectadas con el Gateway central.
  * OrbitControls orbitales con rotación por arrastre de ratón y zoom por rueda.
  * Raycasting 3D interactivo para selección de nodo al hacer clic con iluminación en anillo (Halo Ring) y panel lateral desplegable con métricas forenses detalladas: IP, MAC, Fabricante/Vendor, Tipo de dispositivo, Capa OSI, Protocolo de gestión y notas forenses.
  * Filtros dinámicos por estado de nodo: *Todos*, *Atacante*, *Comprometido*, *Seguro*.
  * Acciones rápidas de aislamiento inmediato (RADIUS CoA) y visualización de alertas asociadas al nodo.

### C. Panel Avanzado de Blast Radius y Mitigación Manual (`BlastRadiusControl`) — `src/components/blast_radius_control.rs`
* **Propósito:** Consola de toma de decisiones para ataques de Capa 7 (DNS Sinkholing) o acciones con impacto masivo que se encuentran retenidas en la cola con estado `PENDING_MANUAL`.
* **Características Implementadas:**
  * Tarjetas de incidentes que muestran claramente el **Radio de Impacto (Blast Radius)**: *Singular (Auto-ejecutable)* vs. *Masivo (Requiere aprobación del operador)*.
  * Visor modal integrado del **Payload de Rollback JSON Snapshot** (comandos inversos para restaurar la red en caso de falso positivo).
  * Botones de **Aprobar y Ejecutar Mitigación**, **Rechazar / Descartar** y **Revertir Estado (Ejecutar Rollback JSON)**.
  * Flujo estricto de **Re-autenticación por Contraseña** que bloquea la ejecución de acciones críticas si el timeout de inactividad ha expirado, respetando el control de acceso RBAC (bloqueo automático para usuarios con rol `Auditor`).

---

## 2. Integración y Enrutamiento SPA (`App`) — `src/app.rs`
* Pestañas de navegación reactivas:
  * `Panel de Control` (`Dashboard`): Métricas de sistema, gráfica de telemetría y tabla de alertas.
  * `Topología 3D` (`Topology`): Renderizado WebGL Three.js del grafo de red GNS3 / Tigo Business.
  * `Gráficas` (`MonitoringCharts`): Telemetría en tiempo real de anomalías y motor LightGBM.
  * `Flujo en Vivo` (`RealtimeStream`): Streaming continuo de eventos forenses clasificados por capas L2-L7.
  * `Alertas` (`Alerts`): Tabla forense de auditoría con filtrado por severidad.
  * `Blast Radius` (`BlastRadius`): Control manual de mitigaciones masivas y reversiones snapshot.
  * `Usuarios & RBAC` (`UserManagement`): Administración de perfiles, permisos Zero-Trust y configuración de timeouts.
* Flujo de autenticación global: Si no hay sesión iniciada, la aplicación muestra la vista `LoginView`; tras validar credenciales, se accede al panel SOC con selector rápido de perfil y botón de cierre de sesión seguro.

---

## 3. Estado de Verificación
* Compilación: `cargo check` verificado con `0 errores` y `0 advertencias`.
* Compatibilidad WASM: Paquete compatible con compilación web `wasm32-unknown-unknown` vía Trunk / wasm-bindgen.