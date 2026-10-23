// * * * COMPONENTE PRINCIPAL DE LA APLICACIÓN SPA (LEPTOS WASM) * * *
// Conexión activa con Backend Axum en :3000 via gloo-net HTTP (polling cada 15s)

use leptos::*;
use gloo_timers::future::IntervalStream;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::components::{
    AlertsTable, BlastRadiusControl, LoginView, MlMetrics, MonitoringChart, RealtimeAlerts,
    SandboxAcl, SystemStatus, Topology3D, UserManagement,
};
use crate::models::{
    AlertItem, GlobalSecurityConfig, MetricDataPoint, PendingMitigation,
    RealtimeStreamEvent, RoleType, Severity, SocUser, SystemMetrics, UserSessionState,
};

// Paleta institucional oficial del manual de mockups (MockupsFiles/Final) —
// ISO/IEC 25010, contraste WCAG 2.2 >= 12:1 en textos y controles.
/// Azul #0284C7 — navegación principal, filtros seleccionados, acción primaria
pub const TIGO_BLUE: &str = "#0284c7";
/// Verde #16A34A — tráfico normal verificado, TLS 1.3, 2FA válido, enlaces activos
pub const TIGO_GREEN: &str = "#16a34a";
/// Rojo #DC2626 — exfiltración crítica, DDoS volumétrico, bloqueo/descarte
pub const TIGO_RED: &str = "#dc2626";
/// Ámbar #D97706 — rate limiting, retención preventiva, umbrales operacionales
pub const TIGO_AMBER: &str = "#d97706";
/// Índigo #4F46E5 — subred de pruebas VLAN 999 (Sandbox ACL), uso exclusivo
pub const TIGO_INDIGO: &str = "#4f46e5";

/// Indicador de estado del backend Axum (Online/Offline) según el último sondeo HTTP
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendStatus {
    Online,
    Offline,
}

/// Usuario vacío inicial — la señal users_list/session se llena con datos REALES
/// vía GET /api/v1/users y POST /api/v1/auth/login (nunca con mocks).
impl Default for SocUser {
    fn default() -> Self {
        SocUser {
            user_id: 0,
            username: String::new(),
            role: RoleType::Auditor,
            created_at: String::new(),
            is_active: true,
        }
    }
}

/// Estado central de datos en vivo obtenidos del backend Axum. Se expone a los
/// componentes hijos mediante señales derivadas (`use_derived`) y permite que el
/// ciclo de polling (un cierre `Copy`) actualice todo desde un único `SignalSet`.
#[derive(Clone, PartialEq)]
pub struct DataState {
    pub metrics: SystemMetrics,
    pub alerts: Vec<AlertItem>,
    pub pending: Vec<PendingMitigation>,
    pub chart: Vec<MetricDataPoint>,
    pub stream: Vec<RealtimeStreamEvent>,
    pub users: Vec<SocUser>,
}

impl DataState {
    fn initial() -> Self {
        DataState {
            metrics: SystemMetrics::default(),
            alerts: Vec::new(),
            pending: Vec::new(),
            chart: Vec::new(),
            stream: Vec::new(),
            users: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Dashboard,
    Topology,
    MonitoringCharts,
    MlMetrics,
    RealtimeStream,
    Sandbox,
    Alerts,
    BlastRadius,
    UserManagement,
}

#[component]
pub fn App() -> impl IntoView {
    // ── Estado central EN VIVO (SIN MOCKS: arranca vacío y lo llena el polling HTTP) ─
    let (data_state, set_data_state) =
        create_signal::<DataState>(DataState::initial());

    // ── Señales espejo para los componentes hijos (compatibles con sus props
    //    ReadSignal/WriteSignal actuales, sin modificar dichos componentes) ──────
    let (metrics, set_metrics) = create_signal::<SystemMetrics>(SystemMetrics::default());
    let (alerts, _set_alerts_unused) = create_signal::<Vec<AlertItem>>(Vec::new());
    let (pending_items, set_pending_items) =
        create_signal::<Vec<PendingMitigation>>(Vec::new());
    let (chart_data, set_chart_data) =
        create_signal::<Vec<MetricDataPoint>>(Vec::new());
    let (stream_events, set_stream_events) =
        create_signal::<Vec<RealtimeStreamEvent>>(Vec::new());
    let (users_list, set_users_list) = create_signal::<Vec<SocUser>>(Vec::new());

    // Efecto reactivo: propaga cada actualización del estado central (polling HTTP)
    // hacia las señales espejo que consumen SystemStatus, AlertsTable, MonitoringChart,
    // RealtimeAlerts, BlastRadiusControl y UserManagement.
    create_effect(move |_| {
        let d = data_state.get();
        set_metrics.set(d.metrics.clone());
        _set_alerts_unused.set(d.alerts.clone());
        set_pending_items.set(d.pending.clone());
        set_chart_data.set(d.chart.clone());
        set_stream_events.set(d.stream.clone());
        set_users_list.set(d.users.clone());
    });

    let (selected_severity, set_selected_severity) =
        create_signal::<Option<Severity>>(None);
    let (active_tab, set_active_tab) = create_signal::<ActiveTab>(ActiveTab::Dashboard);

    // ── Estados de gestión de sesiones y RBAC ──────────────────────────────
    let (session, set_session) = create_signal::<UserSessionState>(UserSessionState {
        current_user: SocUser::default(),
        session_token: String::new(),
        expires_at: "—".to_string(),
        last_reauth_timestamp: 0.0,
    });
    let (security_config, set_security_config) = create_signal::<GlobalSecurityConfig>(
        GlobalSecurityConfig { critical_action_timeout_secs: 60 },
    );

    // ── Estado de autenticación y salud del backend ────────────────────────
    // is_authenticated: false = pantalla de login, true = dashboard SOC
    let (is_authenticated, set_is_authenticated) = create_signal::<bool>(false);
    // BackendStatus: Online/Offline según el resultado del último sondeo HTTP real
    let (backend_status, set_backend_status) =
        create_signal::<BackendStatus>(BackendStatus::Offline);

    // ════════════════════════════════════════════════════════════════════════
    // CICLO DE POLLING EN VIVO contra el backend Axum (gloo-net + spawn_local)
    //   GET /api/v1/alerts            → alertas enriquecidas + cola Blast Radius + stream
    //   GET /api/v1/metrics           → resumen global (PostgreSQL agregado)
    //   GET /api/v1/telemetry/metrics → ventanas de tráfico para las gráficas SVG
    //   GET /api/v1/users             → directorio RBAC real
    // Cada sondeo actualiza el indicador Online/Offline del header.
    // ════════════════════════════════════════════════════════════════════════
    fn run_poll(
        first_run: bool,
        set_data_state: WriteSignal<DataState>,
        set_security_config: WriteSignal<GlobalSecurityConfig>,
        set_backend_status: WriteSignal<BackendStatus>,
    ) {
        spawn_local(async move {
            let mut ok_any = false;

            // 1) Alertas reales → tabla, cola PENDING_MANUAL (Blast Radius) y flujo en vivo
            match api::fetch_alerts(if first_run { "primera carga" } else { "polling 15s" }).await {
                Ok(raw) => {
                    ok_any = true;
                    let pending: Vec<PendingMitigation> = raw
                        .iter()
                        .filter(|a| {
                            a.status_name
                                .as_deref()
                                .unwrap_or("")
                                .eq_ignore_ascii_case("PENDING_MANUAL")
                        })
                        .map(api::map_pending_mitigation)
                        .collect();
                    let mapped: Vec<AlertItem> =
                        raw.clone().into_iter().map(api::map_enriched_alert).collect();
                    let stream = api::map_alerts_to_stream(raw);
                    set_data_state.update(|d| {
                        d.alerts = mapped;
                        d.pending = pending;
                        d.stream = stream;
                    });
                }
                Err(e) => log::warn!("Polling /api/v1/alerts falló: {}", e),
            }

            // 2) Resumen de métricas del sistema (pending_manual lo fija la cola real abajo)
            if let Ok(summary) = api::fetch_metrics_summary().await {
                ok_any = true;
                let fresh = api::map_metrics_summary(summary, None);
                set_data_state.update(|d| d.metrics = fresh);
            }

            // 3) Ventanas de telemetría → series de las gráficas
            match api::fetch_telemetry_metrics().await {
                Ok(points) => {
                    ok_any = true;
                    let pts = api::map_traffic_metrics_to_points(points);
                    set_data_state.update(|d| d.chart = pts);
                }
                Err(e) => log::warn!("Polling /api/v1/telemetry/metrics falló: {}", e),
            }

            // 4) Directorio de usuarios RBAC
            match api::fetch_users().await {
                Ok(us) => {
                    ok_any = true;
                    let users: Vec<SocUser> = us.into_iter().map(api::map_user_with_role).collect();
                    set_data_state.update(|d| d.users = users);
                }
                Err(e) => log::warn!("Polling /api/v1/users falló: {}", e),
            }

            // 5) Timeout global de acciones críticas (config zero-trust del backend)
            if let Ok(secs) = api::fetch_critical_timeout().await {
                set_security_config.set(GlobalSecurityConfig {
                    critical_action_timeout_secs: secs.max(1) as u64,
                });
            }

            // 6) Indicador Online/Offline: health-check explícito o cualquier sondeo exitoso
            let healthy = ok_any || api::fetch_system_status().await.is_ok();
            set_backend_status.set(if healthy {
                BackendStatus::Online
            } else {
                BackendStatus::Offline
            });
        });
    }

    // Cierre `Copy`: captura solo señales/setters (todos `Copy` en Leptos 0.6), por lo
    // que puede invocarse en cada tick del IntervalStream sin mover ningún estado.
    let poll_once = move |first_run: bool| {
        run_poll(first_run, set_data_state, set_security_config, set_backend_status);
    };

    // Bandera del ciclo de polling: mientras esté activa, el bucle de IntervalStream
    // sigue sondeando; al cerrar sesión se marca en falso y el bucle termina.
    let (poll_active, set_poll_active) = create_signal::<bool>(false);

    // Efecto reactivo: arranca el ciclo solo cuando hay sesión autenticada y
    // lo detiene (deja de sondear) al cerrar sesión.
    create_effect(move |_| {
        if is_authenticated.get() {
            // Primera consulta inmediata al entrar al dashboard
            poll_once(true);

            if !poll_active.get() {
                set_poll_active.set(true);
                let active = poll_active.clone();
                // Polling periódico cada 15 segundos vía IntervalStream (gloo-timers)
                let mut stream = IntervalStream::new(15_000);
                spawn_local(async move {
                    use futures_lite::stream::StreamExt;
                    while stream.next().await.is_some() {
                        if !active.get() {
                            break; // sesión cerrada → fin del ciclo de sondeo
                        }
                        poll_once(false);
                    }
                });
            }
        } else {
            set_poll_active.set(false);
        }
    });

    // Badge dinámico de Blast Radius: sincroniza el tamaño de la cola REAL de
    // mitigaciones PENDING_MANUAL con la tarjeta "Pendientes" del dashboard.
    create_effect(move |_| {
        let count = pending_items.get().len() as u64;
        set_metrics.update(|m| {
            m.pending_manual = count;
        });
    });

    // Callback de login exitoso
    let on_login_success = Callback::new(move |()| {
        set_is_authenticated.set(true);
    });

    // ── Logout REAL: POST /api/v1/auth/logout con el token de sesión ──────
    let handle_logout = move |_| {
        let token = session.get().session_token.clone();
        spawn_local(async move {
            if !token.is_empty() {
                if let Err(e) = api::logout(&token).await {
                    log::warn!("POST /api/v1/auth/logout respondió error: {} (sesión local invalidada)", e);
                }
            }
        });
        set_session.update(|s| {
            s.last_reauth_timestamp = 0.0;
            s.session_token = String::new();
        });
        set_is_authenticated.set(false);
        set_active_tab.set(ActiveTab::Dashboard);
        // Al salir, limpiamos los datos en memoria para no exponer información
        // de la sesión anterior y marcamos el backend como aún no verificado.
        set_data_state.set(DataState::initial());
        set_backend_status.set(BackendStatus::Offline);
    };

    view! {
        {move || {
            if !is_authenticated.get() {
                // ── Pantalla de Login / Registro ──────────────────────────────
                view! {
                    <LoginView
                        users=users_list
                        set_users=set_users_list
                        session=session
                        set_session=set_session
                        on_login_success=on_login_success
                    />
                }.into_view()
            } else {
                // ── Aplicación SOC Principal ──────────────────────────────────
                view! {
                    <div class="soc-app">
                        // ── Barra de Navegación Superior ─────────────────────
                        <header class="soc-header">
                            <div class="soc-brand">
                                <div class="soc-logo-icon">"🛡️"</div>
                                <div>
                                    <h1>"TIGO "<span>"SOC"</span>" MONITOR"</h1>
                                </div>
                            </div>

                            <nav class="soc-nav">
                                <button
                                    class=move || if active_tab.get() == ActiveTab::Dashboard { "nav-btn active" } else { "nav-btn" }
                                    on:click=move |_| set_active_tab.set(ActiveTab::Dashboard)
                                >
                                    "📊 Panel de Control"
                                </button>
                                <button
                                    class=move || if active_tab.get() == ActiveTab::Topology { "nav-btn active" } else { "nav-btn" }
                                    on:click=move |_| set_active_tab.set(ActiveTab::Topology)
                                >
                                    "🌐 Topología 3D"
                                </button>
                                <button
                                    class=move || if active_tab.get() == ActiveTab::MonitoringCharts { "nav-btn active" } else { "nav-btn" }
                                    on:click=move |_| set_active_tab.set(ActiveTab::MonitoringCharts)
                                >
                                    "📈 Gráficas"
                                </button>
                                <button
                                    class=move || if active_tab.get() == ActiveTab::MlMetrics { "nav-btn active" } else { "nav-btn" }
                                    on:click=move |_| set_active_tab.set(ActiveTab::MlMetrics)
                                >
                                    "🤖 Motor ML"
                                </button>
                                <button
                                    class=move || if active_tab.get() == ActiveTab::RealtimeStream { "nav-btn active" } else { "nav-btn" }
                                    on:click=move |_| set_active_tab.set(ActiveTab::RealtimeStream)
                                >
                                    "⚡ Flujo en Vivo"
                                </button>
                                <button
                                    class=move || if active_tab.get() == ActiveTab::Sandbox { "nav-btn active" } else { "nav-btn" }
                                    on:click=move |_| set_active_tab.set(ActiveTab::Sandbox)
                                >
                                    "🧪 Sandbox VLAN 999"
                                </button>
                                <button
                                    class=move || if active_tab.get() == ActiveTab::Alerts { "nav-btn active" } else { "nav-btn" }
                                    on:click=move |_| set_active_tab.set(ActiveTab::Alerts)
                                >
                                    "🚨 Alertas"
                                </button>
                                <button
                                    class=move || if active_tab.get() == ActiveTab::BlastRadius { "nav-btn active" } else { "nav-btn" }
                                    on:click=move |_| set_active_tab.set(ActiveTab::BlastRadius)
                                >
                                    "⚠️ Blast Radius "
                                    <span style="background-color: #d97706; color: #ffffff; padding: 0.1rem 0.4rem; border-radius: 10px; font-size: 0.6875rem; font-weight: 700; margin-left: 0.25rem; font-family: var(--font-mono);">
                                        {move || pending_items.get().len().to_string()}
                                    </span>
                                </button>
                                // Pestaña de Gestión de Usuarios — solo visible para Administrador General
                                {move || {
                                    if session.get().current_user.role == RoleType::Admin {
                                        view! {
                                            <button
                                                class=move || if active_tab.get() == ActiveTab::UserManagement { "nav-btn active" } else { "nav-btn" }
                                                on:click=move |_| set_active_tab.set(ActiveTab::UserManagement)
                                            >
                                                "👥 Usuarios & RBAC"
                                            </button>
                                        }.into_view()
                                    } else {
                                        view! {}.into_view()
                                    }
                                }}
                            </nav>

                            <div class="soc-header-status">
                                // Badge de usuario autenticado
                                <div style="display:flex; align-items:center; gap:0.4rem; font-size:0.78rem; color:var(--ctp-subtext0);">
                                    <span style="color:var(--ctp-green);">"✓"</span>
                                    {move || session.get().current_user.username.clone()}
                                    <span class=move || match session.get().current_user.role {
                                        RoleType::Admin => "badge-role badge-role-admin",
                                        RoleType::SocOperator => "badge-role badge-role-operator",
                                        RoleType::Auditor => "badge-role badge-role-auditor",
                                        RoleType::AnalystL1 => "badge-role badge-role-l1",
                                    }>
                                        {move || session.get().current_user.role.display_name()}
                                    </span>
                                </div>

                                // Indicador REAL de disponibilidad del backend Axum
                                // (Online/Offline según el último sondeo HTTP del ciclo de polling)
                                <div
                                    class=move || match backend_status.get() {
                                        BackendStatus::Online => "status-badge online",
                                        BackendStatus::Offline => "status-badge offline",
                                    }
                                >
                                    <span class="status-dot"></span>
                                    <span>
                                        {move || match backend_status.get() {
                                            BackendStatus::Online => "BACKEND EN LÍNEA",
                                            BackendStatus::Offline => "BACKEND OFFLINE",
                                        }}
                                    </span>
                                </div>

                                // Botón de logout
                                <button
                                    class="btn btn-inspect"
                                    style="padding: 0.35rem 0.7rem; font-size: 0.75rem; border-color: var(--ctp-red); color: var(--ctp-red);"
                                    on:click=handle_logout
                                >
                                    "⏏ Cerrar Sesión"
                                </button>
                            </div>
                        </header>

                        // ── Contenedor Principal de Vistas ────────────────────
                        <main class="soc-main">
                            {move || match active_tab.get() {
                                ActiveTab::Dashboard => view! {
                                    <div>
                                        <SystemStatus metrics=metrics />
                                        <div style="margin-bottom: 1.5rem;">
                                            <MonitoringChart
                                                chart_data=chart_data
                                            />
                                        </div>
                                        <h2 style="font-size: 1.125rem; font-weight: 700; margin-bottom: 0.75rem; color: var(--ctp-text);">
                                            "Últimas Detecciones de Seguridad (Multi-Vendor)"
                                        </h2>
                                        <AlertsTable
                                            alerts=alerts
                                            selected_severity=selected_severity
                                            set_selected_severity=set_selected_severity
                                        />
                                    </div>
                                }.into_view(),

                                ActiveTab::Topology => view! {
                                    <div>
                                        <Topology3D />
                                    </div>
                                }.into_view(),

                                ActiveTab::MonitoringCharts => view! {
                                    <div>
                                        <h2 style="font-size: 1.25rem; font-weight: 700; margin-bottom: 1rem; color: var(--ctp-text);">
                                            "Telemetría en Tiempo Real y Métricas del Motor GBDT"
                                        </h2>
                                        <SystemStatus metrics=metrics />
                                        <MonitoringChart
                                            chart_data=chart_data
                                        />
                                    </div>
                                }.into_view(),

                                // Lámina 05/20 — Métricas ML LightGBM (API real /api/v1/ml/*)
                                ActiveTab::MlMetrics => view! {
                                    <div>
                                        <h2 style="font-size: 1.25rem; font-weight: 700; margin-bottom: 1rem; color: var(--ctp-text);">
                                            "Inferencia Algorítmica LightGBM y Validación K-Fold"
                                        </h2>
                                        <MlMetrics />
                                    </div>
                                }.into_view(),

                                ActiveTab::RealtimeStream => view! {
                                    <div>
                                        <h2 style="font-size: 1.25rem; font-weight: 700; margin-bottom: 1rem; color: var(--ctp-text);">
                                            "Flujo Continuo de Eventos Forenses y Alertas en Streaming"
                                        </h2>
                                        <RealtimeAlerts
                                            events=stream_events
                                            set_events=set_stream_events
                                        />
                                    </div>
                                }.into_view(),

                                // Lámina 10/21 — Sandbox ACL en subred aislada (índigo exclusivo)
                                ActiveTab::Sandbox => view! {
                                    <div>
                                        <h2 style="font-size: 1.25rem; font-weight: 700; margin-bottom: 1rem; color: var(--ctp-text);">
                                            "Ambiente de Pruebas Aislado — Inyección Controlada y Promoción"
                                        </h2>
                                        <SandboxAcl
                                            set_stream_events=set_stream_events
                                            alerts=alerts
                                            session=session
                                        />
                                    </div>
                                }.into_view(),

                                ActiveTab::Alerts => view! {
                                    <div>
                                        <h2 style="font-size: 1.25rem; font-weight: 700; margin-bottom: 1rem; color: var(--ctp-text);">
                                            "Monitoreo Forense de Alertas en Tiempo Real"
                                        </h2>
                                        <AlertsTable
                                            alerts=alerts
                                            selected_severity=selected_severity
                                            set_selected_severity=set_selected_severity
                                        />
                                    </div>
                                }.into_view(),

                                ActiveTab::BlastRadius => view! {
                                    <div>
                                        <h2 style="font-size: 1.25rem; font-weight: 700; margin-bottom: 1rem; color: var(--ctp-text);">
                                            "Mitigaciones Retenidas por Radio de Impacto (PENDING_MANUAL)"
                                        </h2>
                                        <BlastRadiusControl
                                            pending_items=pending_items
                                            set_pending_items=set_pending_items
                                            session=session
                                            set_session=set_session
                                            security_config=security_config
                                        />
                                    </div>
                                }.into_view(),

                                ActiveTab::UserManagement => view! {
                                    <div>
                                        <h2 style="font-size: 1.25rem; font-weight: 700; margin-bottom: 1rem; color: var(--ctp-text);">
                                            "Administración de Usuarios, Control de Acceso (RBAC) y Seguridad Zero-Trust"
                                        </h2>
                                        {move || {
                                            if session.get().current_user.role == RoleType::Admin {
                                                view! {
                                                    <UserManagement
                                                        users=users_list
                                                        set_users=set_users_list
                                                        session=session
                                                        set_session=set_session
                                                        security_config=security_config
                                                        set_security_config=set_security_config
                                                    />
                                                }.into_view()
                                            } else {
                                                view! {
                                                    <div style="
                                                        display: flex;
                                                        flex-direction: column;
                                                        align-items: center;
                                                        justify-content: center;
                                                        padding: 4rem 2rem;
                                                        text-align: center;
                                                    ">
                                                        <div style="font-size: 3.5rem; margin-bottom: 1rem;">"🔒"</div>
                                                        <h3 style="font-size: 1.5rem; font-weight: 700; color: var(--ctp-red); margin-bottom: 0.5rem;">
                                                            "403 — Acceso Denegado"
                                                        </h3>
                                                        <p style="font-size: 0.9rem; color: var(--ctp-subtext0); max-width: 480px; line-height: 1.6;">
                                                            "El módulo de administración de usuarios y control RBAC está restringido "
                                                            "exclusivamente al rol "
                                                            <strong style="color: var(--ctp-peach);">"Administrador General"</strong>
                                                            ". Su perfil actual no posee los permisos requeridos (ISO/IEC 27001 — Principio de Mínimo Privilegio)."
                                                        </p>
                                                        <div style="margin-top: 1.5rem;">
                                                            <span class="badge-role badge-role-admin" style="font-size: 0.875rem; padding: 0.4rem 0.75rem;">
                                                                "Solo Administrador General"
                                                            </span>
                                                        </div>
                                                    </div>
                                                }.into_view()
                                            }
                                        }}
                                    </div>
                                }.into_view(),
                            }}
                        </main>

                        // ── Pie de Página ─────────────────────────────────────
                        <footer class="soc-footer">
                            <p>
                                "Tigo SOC © 2026 | Arquitectura Ciberseguridad Multi-Vendor (Rust Tokio + Leptos WASM + LightGBM + Axum) | Bolivia"
                            </p>
                        </footer>
                    </div>
                }.into_view()
            }
        }}
    }
}
